use crate::{controller, db_client, db_user, schema};
use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, serde::Deserialize, Clone)]
struct Login {
    #[serde(rename = "id")]
    user_id: String,
    #[serde(rename = "password")]
    user_password: String,
}

#[derive(Debug, serde::Serialize, Clone)]
struct Token {
    token: String,
}

impl Token {
    #[allow(clippy::wrong_self_convention)]
    fn to_value(self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

fn validate_login_request(payload: &serde_json::Value) -> anyhow::Result<Login> {
    schema::validate(payload, &schema::LOGIN)?;
    let login: Login = serde_json::from_value(payload.clone()).unwrap();
    Ok(login)
}

pub async fn login(
    extract::State(db): extract::State<Arc<RwLock<db_client::DB>>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    use db_user::UpdatedTokenResult;

    let response: (StatusCode, Json<serde_json::Value>) = match validate_login_request(&payload) {
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            controller::Error::new(err.to_string()).to_value().into(),
        ),
        Ok(login) => {
            match db_user::User::update_token(
                &login.user_id,
                &login.user_password,
                &mut db.write().await.client,
            )
            .await
            {
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    controller::Error::new(err.to_string()).to_value().into(),
                ),
                Ok(result) => match result {
                    UpdatedTokenResult::Ok(user) => (
                        StatusCode::OK,
                        Token { token: user.token }.to_value().into(),
                    ),
                    UpdatedTokenResult::UserNotFound => (
                        StatusCode::NOT_FOUND,
                        controller::Error::new("user not found".into())
                            .to_value()
                            .into(),
                    ),
                    UpdatedTokenResult::WrongPassword => (
                        StatusCode::BAD_REQUEST,
                        controller::Error::new("invalid data".into())
                            .to_value()
                            .into(),
                    ),
                },
            }
        }
    };
    response
}
