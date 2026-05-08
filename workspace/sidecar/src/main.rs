// main.rs
use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse, middleware};
use actix_ws::Message;
use futures_util::{SinkExt, StreamExt};
use std::env;

async fn validate_token(token: &str, cluster_id: &str) -> bool {
    let backend = env::var("HOST").unwrap();

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/api/workspaces/validatetoken/{}/{}", backend, cluster_id, token))
        .send()
        .await;

    matches!(res, Ok(r) if r.status().is_success())
}

async fn terminal(req: HttpRequest, stream: web::Payload) -> HttpResponse {
    let cluster_id = env::var("CLUSTER_ID").unwrap();

    // grab token from query string
    let token = match web::Query::<std::collections::HashMap<String, String>>::from_query(
        req.query_string()
    ) {
        Ok(q) => match q.get("token") {
            Some(t) => t.clone(),
            None => return HttpResponse::Unauthorized().finish(),
        },
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    if !validate_token(&token, &cluster_id).await {
        return HttpResponse::Unauthorized().finish();
    }

    let (res, mut session, mut client_stream) = match actix_ws::handle(&req, stream) {
        Ok(v) => v,
        Err(e) => return e.error_response(),
    };

    let ttyd_url = env::var("TTYD_URL").unwrap_or("ws://localhost:7681/ws".into());

    actix_web::rt::spawn(async move {
        let Ok((ttyd_ws, _)) = tokio_tungstenite::connect_async(&ttyd_url).await else {
            session.close(None).await.ok();
            return;
        };

        let (mut ttyd_tx, mut ttyd_rx) = ttyd_ws.split();

        // client → ttyd
        let mut session_clone = session.clone();
        let t1 = actix_web::rt::spawn(async move {
            while let Some(Ok(msg)) = client_stream.next().await {
                let out = match msg {
                    Message::Binary(b) => tokio_tungstenite::tungstenite::Message::Binary(b.into()),
                    Message::Text(t) => tokio_tungstenite::tungstenite::Message::Text(t.to_string()),
                    Message::Close(_) => break,
                    Message::Ping(b) => tokio_tungstenite::tungstenite::Message::Ping(b.into()),
                    _ => continue,
                };
                if ttyd_tx.send(out).await.is_err() { break; }
            }
        });

        // ttyd → client
        let t2 = actix_web::rt::spawn(async move {
            while let Some(Ok(msg)) = ttyd_rx.next().await {
                let res = match msg {
                    tokio_tungstenite::tungstenite::Message::Binary(b) => {
                        session_clone.binary(b).await
                    },
                    tokio_tungstenite::tungstenite::Message::Text(t) => {
                        session_clone.text(t).await
                    },
                    tokio_tungstenite::tungstenite::Message::Ping(b) => {
                        session_clone.ping(&b).await
                    },
                    _ => continue,
                };
                if res.is_err() { break; }
            }
        });

        tokio::select! {
            _ = t1 => {},
            _ = t2 => {},
        }
    });

    res
}

// yeet all connections to ttyd
async fn proxy_http(req: HttpRequest) -> HttpResponse {
    let ttyd = env::var("TTYD_HTTP_URL").unwrap_or("http://localhost:7681".into());
    let path = req.uri().path_and_query().map(|p| p.as_str()).unwrap_or("/");

    let client = reqwest::Client::new();
    match client.get(format!("{}{}", ttyd, path)).send().await {
        Ok(res) => {
            let status = actix_web::http::StatusCode::from_u16(
                res.status().as_u16()
            ).unwrap();
            let body = res.bytes().await.unwrap_or_default();
            HttpResponse::build(status).body(body)
        },
        Err(_) => HttpResponse::BadGateway().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/ws", web::get().to(terminal))
            .default_service(web::to(proxy_http))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
