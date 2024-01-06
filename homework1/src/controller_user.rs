use crate::{controller, db, db_user, password::hash_password, schema};
use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use chrono::Datelike;
use std::sync::Arc;
use tokio_postgres::GenericClient;

pub async fn create_user(
    extract::State(db): extract::State<Arc<db::DB>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let response: (StatusCode, Json<serde_json::Value>) =
        match validate_create_user_request(&payload) {
            Err(err) => (
                StatusCode::BAD_REQUEST,
                serde_json::Value::from(controller::Error {
                    message: err.to_string(),
                })
                .into(),
            ),
            Ok(user) => {
                let user: db_user::User = user.into();
                match user.insert_to_db(&db).await {
                    Err(err) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        serde_json::Value::from(controller::Error {
                            message: err.to_string(),
                        })
                        .into(),
                    ),
                    Ok(user_id) => {
                        tracing::info!("a new user registered with ID {}", user_id);
                        (
                            StatusCode::CREATED,
                            serde_json::Value::from(CreateUserResponse { user_id }).into(),
                        )
                    }
                }
            }
        };

    response
}

pub async fn get_user(
    extract::State(db): extract::State<Arc<db::DB>>,
    extract::Path(id): extract::Path<String>,
) -> impl IntoResponse {
    let pg_pool = match db.get().await {
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

    let response: (StatusCode, Json<serde_json::Value>) =
        match db_user::User::from_id(&id, pg_pool.client()).await {
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::Value::from(controller::Error {
                    message: err.to_string(),
                })
                .into(),
            ),
            Ok(user) => match user {
                Some(dbuser) => (
                    StatusCode::OK,
                    serde_json::Value::from(GetUser::from(dbuser)).into(),
                ),
                None => (StatusCode::NOT_FOUND, serde_json::json!({}).into()),
            },
        };

    response
}

#[derive(Debug, serde::Deserialize, Clone)]
struct CreateUserRequest {
    first_name: String,
    second_name: String,
    biography: String,
    birthdate: String,
    city: String,
    password: String,
}

#[derive(Debug, serde::Serialize, Clone)]
struct CreateUserResponse {
    user_id: String,
}

fn validate_create_user_request(payload: &serde_json::Value) -> anyhow::Result<CreateUserRequest> {
    schema::validate(payload, &schema::USER_REGISTER)?;
    let user: CreateUserRequest = serde_json::from_value(payload.clone()).unwrap();
    user.validate()?;
    Ok(user)
}

impl CreateUserRequest {
    fn validate(&self) -> anyhow::Result<()> {
        let date_18_old = chrono::Local::now() - chrono::Months::new(12 * 18);
        let date_18_old = format!(
            "{:0>4}-{:0>2}-{:0>2}",
            date_18_old.year(),
            date_18_old.month(),
            date_18_old.day()
        );

        let birthdate = self.birthdate.as_str();
        if birthdate > date_18_old.as_str() {
            anyhow::bail!(
                "you must be 18 year old or over: but {} > {}",
                birthdate,
                date_18_old
            )
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct GetUser {
    id: String,
    first_name: String,
    second_name: String,
    biography: String,
    birthdate: String,
    city: String,
}

impl From<db_user::User> for GetUser {
    fn from(user: db_user::User) -> Self {
        Self {
            id: user.id,
            first_name: user.first_name,
            second_name: user.second_name,
            birthdate: user.birthdate,
            biography: user.biography,
            city: user.city,
        }
    }
}

impl From<CreateUserResponse> for serde_json::Value {
    fn from(user: CreateUserResponse) -> Self {
        serde_json::to_value(user).unwrap()
    }
}

impl From<GetUser> for serde_json::Value {
    fn from(user: GetUser) -> Self {
        serde_json::to_value(user).unwrap()
    }
}

impl From<CreateUserRequest> for db_user::User {
    fn from(user: CreateUserRequest) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            first_name: user.first_name,
            second_name: user.second_name,
            birthdate: user.birthdate,
            biography: user.biography,
            city: user.city,
            password_hash: hash_password(user.password.as_str()),
            token: String::new(),
        }
    }
}
