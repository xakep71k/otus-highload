use crate::{controller, db, db_user, schema};
use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

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

impl From<Token> for serde_json::Value {
    fn from(token: Token) -> Self {
        serde_json::to_value(token).unwrap()
    }
}

fn validate_login_request(payload: &serde_json::Value) -> anyhow::Result<Login> {
    schema::validate(payload, &schema::LOGIN)?;
    let login: Login = serde_json::from_value(payload.clone()).unwrap();
    Ok(login)
}

pub async fn login(
    extract::State(db): extract::State<Arc<db::DB>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let response: (StatusCode, Json<serde_json::Value>) = match validate_login_request(&payload) {
        Err(err) => (
            StatusCode::BAD_REQUEST,
            serde_json::Value::from(controller::Error {
                message: err.to_string(),
            })
            .into(),
        ),
        Ok(login) => {
            let mut pg_pool = match db.get().await {
                Err(err) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json::<serde_json::Value>(serde_json::Value::from(controller::Error {
                            message: err.to_string(),
                        })),
                    )
                }
                Ok(object) => object,
            };

            match db_user::User::update_token(&login.user_id, &login.user_password, &mut pg_pool)
                .await
            {
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    serde_json::Value::from(controller::Error {
                        message: err.to_string(),
                    })
                    .into(),
                ),
                Ok(result) => update_token_result_to_resonse(&login.user_id, result),
            }
        }
    };
    response
}

fn update_token_result_to_resonse(
    user_id: &str,
    result: db_user::UpdatedTokenResult,
) -> (StatusCode, Json<serde_json::Value>) {
    use db_user::UpdatedTokenResult;
    match result {
        UpdatedTokenResult::Ok(user) => {
            tracing::info!("user with id '{}' has been login successfuly", user_id,);
            (
                StatusCode::OK,
                serde_json::Value::from(Token { token: user.token }).into(),
            )
        }
        UpdatedTokenResult::UserNotFound => {
            tracing::error!("user with id '{}' not found", user_id);
            (StatusCode::NOT_FOUND, serde_json::json!({}).into())
        }
        UpdatedTokenResult::WrongPassword => {
            tracing::error!(
                "user with id '{}' has been tried to login with wrong password",
                user_id,
            );
            (
                StatusCode::BAD_REQUEST,
                serde_json::Value::from(controller::Error {
                    message: "wrong password".into(),
                })
                .into(),
            )
        }
    }
}
