use crate::{db::DB, password};

#[derive(serde::Serialize)]
pub struct User {
    pub id: String,
    pub first_name: String,
    pub second_name: String,
    pub biography: String,
    pub birthdate: String,
    pub city: String,
    pub password_hash: String,
    pub token: String,
}

impl User {
    pub async fn insert_to_db(self, db: &DB) -> anyhow::Result<String> {
        let statement = "INSERT INTO users (id, first_name, second_name, birthdate, biography, city, password_hash, token) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";
        db.get()
            .await?
            .execute(
                statement,
                &[
                    &self.id,
                    &self.first_name,
                    &self.second_name,
                    &self.birthdate,
                    &self.biography,
                    &self.city,
                    &self.password_hash,
                    &"",
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("insert failed: {}", e))?;
        Ok(self.id)
    }

    fn from_row(row: &tokio_postgres::Row) -> Self {
        Self {
            id: row.get(0),
            first_name: row.get(1),
            second_name: row.get(2),
            birthdate: row.get(3),
            biography: row.get(4),
            city: row.get(5),
            password_hash: row.get(6),
            token: row.get(7),
        }
    }

    pub async fn from_id(
        id: &String,
        client: &tokio_postgres::Client,
    ) -> anyhow::Result<Option<Self>> {
        let statement = "select * from users where id = $1";
        let rows = client
            .query(statement, &[&id])
            .await
            .map_err(|e| anyhow::anyhow!("failed to read user: {}", e))?;
        if rows.is_empty() {
            Ok(None)
        } else {
            let row = rows.iter().last().unwrap();
            Ok(Some(User::from_row(row)))
        }
    }

    pub async fn update_token(
        user_id: &String,
        password: &str,
        pg_pool: &mut deadpool_postgres::Object,
    ) -> anyhow::Result<UpdatedTokenResult> {
        let token = uuid::Uuid::new_v4().to_string();
        let transaction = pg_pool
            .transaction()
            .await
            .map_err(|e| anyhow::anyhow!("failed to update token: transaction error: {}", e))?;
        let update_token_statement = "UPDATE users SET token = $1 WHERE id = $2";
        let updated = transaction
            .execute(update_token_statement, &[&token, user_id])
            .await
            .map_err(|e| anyhow::anyhow!("failed to updated: update error: {}", e))?;
        if updated != 1 {
            if updated > 1 {
                tracing::error!(
                    "updated more then one row for user: ID = {}, token = {}",
                    user_id,
                    token
                );
            }
            let _ = transaction.rollback().await;
            return Ok(UpdatedTokenResult::UserNotFound);
        }

        let user = User::from_id(user_id, transaction.client()).await?.unwrap();
        if !password::verify_password(password, &user.password_hash) {
            let _ = transaction.rollback().await;
            return Ok(UpdatedTokenResult::WrongPassword);
        }

        transaction
            .commit()
            .await
            .map_err(|e| anyhow::anyhow!("failed to update token: commit error:{}", e))?;

        Ok(UpdatedTokenResult::Ok(user))
    }
}

pub enum UpdatedTokenResult {
    UserNotFound,
    WrongPassword,
    Ok(User),
}
