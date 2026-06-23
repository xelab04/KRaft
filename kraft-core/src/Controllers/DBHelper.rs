pub mod clusters {
    use crate::Models::Cluster::Cluster;
    use actix_web::web;
    use sqlx::PgPool;

    pub async fn list(
        pool: &web::Data<PgPool>,
        user_id_int: &i32,
    ) -> Result<Vec<Cluster>, sqlx::Error> {
        let clusters: Vec<Cluster> = sqlx::query_as::<_, Cluster>("SELECT cluster_id as id, cluster_name as name, cluster_endpoint as endpoint FROM clusters WHERE user_id=($1)")
            .bind(user_id_int)
            .fetch_all(pool.get_ref())
            .await?;

        Ok(clusters)
    }

    pub async fn same_name(
        pool: &web::Data<PgPool>,
        cluster_name: &str,
    ) -> Result<bool, sqlx::Error> {
        let count_same_name: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM clusters WHERE cluster_name = $1)")
                .bind(cluster_name)
                .fetch_one(pool.get_ref())
                .await?;
        Ok(count_same_name)
    }

    pub async fn name_belongs_to(
        pool: &web::Data<PgPool>,
        user_id: &i32,
        cluster_name: &str,
    ) -> Result<bool, sqlx::Error> {
        let r: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM clusters WHERE user_id = $1 AND cluster_name = $2)",
        )
        .bind(user_id)
        .bind(cluster_name)
        .fetch_one(pool.get_ref())
        .await?;
        Ok(r)
    }

    pub async fn id_belongs_to(
        pool: &web::Data<PgPool>,
        user_id: &i32,
        cluster_id: &i32,
    ) -> Result<bool, sqlx::Error> {
        let r: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM clusters WHERE user_id = $1 AND cluster_id = $2)",
        )
        .bind(user_id)
        .bind(cluster_id)
        .fetch_one(pool.get_ref())
        .await?;
        Ok(r)
    }

    pub async fn delete(
        pool: &web::Data<PgPool>,
        int_user_id: &i32,
        raw_cluster_name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM clusters WHERE user_id = $1 AND cluster_name = $2")
            .bind(int_user_id)
            .bind(raw_cluster_name)
            .execute(pool.get_ref())
            .await?;
        Ok(())
    }

    pub async fn cluster_id(
        pool: &web::Data<PgPool>,
        user_id: &i32,
        cluster_name: &str,
    ) -> Result<i32, sqlx::Error> {
        let int_cluster_id: i32 = sqlx::query_scalar(
            "SELECT cluster_id FROM clusters WHERE user_id = $1 AND cluster_name = $2",
        )
        .bind(user_id)
        .bind(cluster_name)
        .fetch_one(pool.get_ref())
        .await?;

        Ok(int_cluster_id)
    }
}

pub mod password {
    use actix_web::web;
    use sqlx::PgPool;

    pub async fn update(
        pool: &web::Data<PgPool>,
        new_hashed_password: &str,
        user_id: &i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET password = ($1) WHERE user_id = ($2)")
            .bind(new_hashed_password)
            .bind(user_id)
            .execute(pool.get_ref())
            .await?;
        Ok(())
    }
}

pub mod user {
    use crate::{Controllers::DBHelper::user, Models::User::User};
    use actix_web::web;
    use sqlx::PgPool;

    pub async fn get_details(pool: &web::Data<PgPool>, user_id: &i32) -> Result<User, sqlx::Error> {
        let user_data = sqlx::query_as::<_, User>(
        "SELECT user_id, username, email, password as user_password, betacode, uuid FROM users WHERE user_id = ($1)"
        )
        .bind(user_id)
        .fetch_one(pool.get_ref())
        .await?;

        Ok(user_data)
    }

    pub async fn get_details_from_email(
        pool: &web::Data<PgPool>,
        email: &str,
    ) -> Result<User, sqlx::Error> {
        let user_data = sqlx::query_as::<_, User>(
        "SELECT user_id, username, email, password as user_password, betacode, uuid FROM users WHERE email = ($1)"
        )
        .bind(email)
        .fetch_one(pool.get_ref())
        .await?;

        Ok(user_data)
    }

    pub async fn get_id_from_uuid(pool: &web::Data<PgPool>, uuid: &str) -> Result<i32, sqlx::Error> {
        let user_id: i32 = sqlx::query_scalar("SELECT user_id FROM users WHERE uuid=($1)")
            .bind(uuid)
            .fetch_one(pool.as_ref())
            .await?;

        Ok(user_id)
    }

    /// used by the admin to list all users
    pub async fn list_users(pool: &web::Data<PgPool>) -> Result<Vec<User>, sqlx::Error> {
        let user_list = sqlx::query_as::<_, User>(
            "SELECT user_id, username, email, 'password' as user_password, betacode, uuid FROM users",
        )
        .fetch_all(pool.as_ref())
        .await?;

        Ok(user_list)
    }

    pub async fn get_role(pool: &web::Data<PgPool>, user_id: &i32) -> Result<String, sqlx::Error> {
        let role = sqlx::query_scalar("SELECT")
            .bind(user_id)
            .fetch_one(pool.get_ref())
            .await?;

        Ok(role)
    }

    pub async fn same_username(
        pool: &web::Data<PgPool>,
        username: &str,
    ) -> Result<bool, sqlx::Error> {
        let same_users: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
                .bind(username)
                .fetch_one(pool.get_ref())
                .await?;
        Ok(same_users)
    }

    pub async fn same_email(
        pool: &web::Data<PgPool>,
        email: &str,
    ) -> Result<bool, sqlx::Error> {
        let same_users: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
                .bind(email)
                .fetch_one(pool.get_ref())
                .await?;
        Ok(same_users)
    }

    pub async fn is_first_user(
        pool: &web::Data<PgPool>
    ) -> Result<bool, sqlx::Error> {
        let same_users: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users LIMIT 1)")
                .fetch_one(pool.get_ref())
                .await?;
        Ok(same_users)
    }

    pub async fn validate(pool: &web::Data<PgPool>, db_token: &str) -> Result<(), sqlx::Error> {
        let _r =
            sqlx::query("UPDATE users SET verified_email = true WHERE verification_code = ($1)")
                .bind(db_token)
                .execute(pool.as_ref())
                .await?;
        Ok(())
    }

    pub async fn get_validation_token(
        pool: &web::Data<PgPool>,
        user_id: &i32,
    ) -> Result<String, sqlx::Error> {
        let possible_stored_user_token =
            sqlx::query_scalar("SELECT verification_code FROM users WHERE user_id = ($1)")
                .bind(user_id)
                .fetch_one(pool.as_ref())
                .await?;

        Ok(possible_stored_user_token)
    }

    pub async fn is_admin(pool: &web::Data<PgPool>, user_id: &i32) -> Result<bool, sqlx::Error> {
        let is_admin: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE user_id = $1 and admin = true)",
        )
        .bind(user_id)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(is_admin)
    }

    pub async fn delete(pool: &web::Data<PgPool>, user_id: &i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM users WHERE user_id = $1")
            .bind(user_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }
}

pub mod workspaces {
    use actix_web::web;
    use chrono::DateTime;
    use chrono::Utc;
    use sqlx::PgPool;

    pub async fn token_delete(pool: &web::Data<PgPool>, user_id: &i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM workspace_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn delete(pool: &web::Data<PgPool>, user_id: &i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM workspaces WHERE user_id = $1")
            .bind(user_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn exists(
        pool: &web::Data<PgPool>,
        user_id: &i32,
        cluster_name: &str,
    ) -> Result<bool, sqlx::Error> {
        let cluster_workspace_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM workspaces WHERE user_id = $1 AND cluster_name = $2)",
        )
        .bind(user_id)
        .bind(cluster_name)
        .fetch_one(pool.get_ref())
        .await?;

        Ok(cluster_workspace_exists)
    }

    pub async fn create(
        pool: &web::Data<PgPool>,
        workspace_name: &str,
        cluster_name: &str,
        user_id: &i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO workspaces (workspace_name, cluster_name, user_id) VALUES ($1, $2, $3)",
        )
        .bind(workspace_name)
        .bind(cluster_name)
        .bind(user_id)
        .execute(pool.get_ref())
        .await?;

        Ok(())
    }

    pub async fn token_create(
        pool: &web::Data<PgPool>,
        token: &str,
        user_id: &i32,
        cluster_id: &i32,
        created_at: &DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        let _r = sqlx::query("INSERT INTO workspace_tokens (token, user_id, cluster_id, created_at, used) VALUES ($1, $2, $3, $4, $5)")
        .bind(token)
        .bind(user_id)
        .bind(cluster_id)
        .bind(created_at)
        .bind(false)
        .execute(pool.get_ref())
        .await
        .unwrap();

        Ok(())
    }
}

pub mod betacode {
    use crate::Models::Betacode::Betacode;
    use actix_web::web;
    use sqlx;
    use sqlx::PgPool;

    pub async fn list(pool: &web::Data<PgPool>) -> Result<Vec<Betacode>, sqlx::Error> {
        let betacodes: Vec<Betacode> =
            sqlx::query_as::<_, Betacode>("SELECT betacode, enabled FROM betacode")
                .fetch_all(pool.as_ref())
                .await?;

        Ok(betacodes)
    }

    pub async fn update(pool: &web::Data<PgPool>, betacode: &Betacode) -> Result<(), sqlx::Error> {
        let _r = sqlx::query("UPDATE betacode SET enabled = ($1) WHERE betacode = ($2)")
            .bind(&betacode.enabled)
            .bind(&betacode.betacode)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn create(pool: &web::Data<PgPool>, betacode: &Betacode) -> Result<(), sqlx::Error> {
        let _r = sqlx::query("INSERT INTO betacode (betacode, enabled) VALUES ($1, $2)")
            .bind(&betacode.betacode)
            .bind(&betacode.enabled)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn delete(pool: &web::Data<PgPool>, betacode: &Betacode) -> Result<(), sqlx::Error> {
        let _r = sqlx::query("DELETE FROM betacode WHERE betacode = ($1)")
            .bind(&betacode.betacode)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn verify(pool: &web::Data<PgPool>, betacode: &str) -> Result<bool, sqlx::Error> {
        let matches: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM betacode WHERE enabled = TRUE AND betacode = ($1))")
            .bind(betacode)
            .fetch_one(pool.get_ref())
            .await?;

        Ok(matches)
    }
}
