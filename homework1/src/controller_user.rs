use crate::{db_client, db_user, password::hash_password, schema};
use axum::{extract, http::StatusCode, response::IntoResponse, Json};
use chrono::Datelike;
use std::sync::Arc;

pub async fn create_user(
    extract::State(db): extract::State<Arc<db_client::DB>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    match validate_create_user_request(&payload) {
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()),
        Ok(user) => {
            let user: db_user::User = user.into();
            match user.insert_to_db(&db).await {
                Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
                Ok(id) => {
                    tracing::info!("a new user registered with ID {}", id);
                    (StatusCode::CREATED, id)
                }
            }
        }
    }
}

pub async fn get_user(
    extract::State(db): extract::State<Arc<db_client::DB>>,
    extract::Path(id): extract::Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match db_user::User::from_db(id, &db).await {
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
        Ok(user) => match user {
            Some(user) => Ok((StatusCode::OK, Json::<GetUser>(user.into()))),
            None => Err((StatusCode::NOT_FOUND, "not found".into())),
        },
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
struct CreateUser {
    first_name: String,
    second_name: String,
    biography: String,
    birthdate: String,
    city: String,
    password: String,
}

fn validate_create_user_request(payload: &serde_json::Value) -> anyhow::Result<CreateUser> {
    schema::validate(payload, &schema::USER_REGISTER)?;
    let user: CreateUser = serde_json::from_value(payload.clone()).unwrap();
    user.validate()?;
    Ok(user)
}

impl CreateUser {
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
                "you must be 18 year old or over: but now {} > {}",
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

impl From<CreateUser> for db_user::User {
    fn from(user: CreateUser) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            first_name: user.first_name,
            second_name: user.second_name,
            birthdate: user.birthdate,
            biography: user.biography,
            city: user.city,
            password_hash: hash_password(user.password.as_str()),
        }
    }
}
