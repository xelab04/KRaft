use actix_web::web::{Json, Path};
use actix_web::{HttpRequest, HttpResponse};
use serde::{Serialize};

use crate::jwt;

#[derive(Serialize)]
pub struct Cluster {
    name: String
}

#[get("/get/clusters")]
pub async fn list(req: HttpRequest) -> HttpResponse {

    let jwt = jwt::extract_user_id_from_jwt(&req);
    match jwt {
        Ok(user_id) => {
            let user_id = user_id;
        }
        Err(e) => {
            println!("Error: {:?}", e);
            // return HttpResponse::Unauthorized().json("Unauthorized")
        }
    };

    // use id to get from postgres

    let clusters = vec![
            Cluster {
                name: "Cluster 1".to_string(),
            },
            Cluster {
                name: "Cluster 2".to_string(),
            },
        ];

    HttpResponse::Ok()
        .content_type("application/json")
        .json(clusters)

}
