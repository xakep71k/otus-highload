use crate::{controller_auth, controller_user, db};
use axum::{routing, Router};
use std::sync::Arc;

pub struct App {}
impl App {
    pub async fn run(
        conn_string: &str,
        bind_string: &str,
        pg_conn_size: usize,
    ) -> anyhow::Result<()> {
        let db = Arc::new(db::DB::new(conn_string, pg_conn_size).await?);
        let app = Router::new()
            .route("/login", routing::post(controller_auth::login))
            .route(
                "/user/register",
                routing::post(controller_user::create_user),
            )
            .route("/user/get/:id", routing::get(controller_user::get_user))
            .with_state(db);
        let listener = tokio::net::TcpListener::bind(bind_string).await.unwrap();
        axum::serve(listener, app)
            .await
            .map_err(|e| anyhow::anyhow!("failed to start app: {}", e))?;
        Ok(())
    }
}
