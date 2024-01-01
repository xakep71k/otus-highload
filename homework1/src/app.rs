use crate::{db, user_controller};
use axum::{routing, Router};
use std::sync::Arc;

pub struct App {}
impl App {
    pub async fn run(conn_string: &String, bind_string: &String) -> anyhow::Result<()> {
        let db = Arc::new(db::DB::new(conn_string).await?);
        let app = Router::new()
            .route(
                "/user/register",
                routing::post(user_controller::create_user),
            )
            .route("/user/get/:id", routing::get(user_controller::get_user))
            .with_state(db);
        let listener = tokio::net::TcpListener::bind(bind_string).await.unwrap();
        axum::serve(listener, app)
            .await
            .map_err(|e| anyhow::anyhow!("failed to start app: {}", e))?;
        Ok(())
    }
}
