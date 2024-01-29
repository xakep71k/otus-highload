use crate::{db::DB, password};

#[derive(serde::Serialize, Debug)]
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

#[derive(serde::Serialize, Debug)]
pub struct SearchResult {
    pub id: String,
    pub first_name: String,
    pub second_name: String,
    pub biography: String,
    pub birthdate: String,
    pub city: String,
}

impl From<User> for SearchResult {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            first_name: user.first_name,
            second_name: user.second_name,
            biography: user.biography,
            birthdate: user.birthdate,
            city: user.city,
        }
    }
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

    pub async fn search(
        client: &tokio_postgres::Client,
        first_name: Option<&String>,
        second_name: Option<&String>,
    ) -> anyhow::Result<Vec<SearchResult>> {
        let stmt_both = "select * from users where first_name LIKE '$1%' and second_name LIKE $2";
        let stmt_first_name = "select * from users where first_name LIKE $1";
        let stmt_second_name = "select * from users where second_name LIKE $1";
        let rows = if first_name.is_some() && second_name.is_some() {
            #[allow(clippy::unnecessary_unwrap)]
            let first_name = first_name.unwrap().to_owned() + "%";
            #[allow(clippy::unnecessary_unwrap)]
            let second_name = second_name.unwrap().to_owned() + "%";
            client
                .query(stmt_both, &[&first_name, &second_name])
                .await
        } else if let Some(first_name) = first_name {
            let first_name = first_name.to_owned() + "%";
            client.query(stmt_first_name, &[&first_name]).await
        } else if let Some(second_name) = second_name {
            let second_name = second_name.to_owned() + "%";
            client.query(stmt_second_name, &[&second_name]).await
        } else {
            return Ok(vec![]);
        }
        .map_err(|e| anyhow::anyhow!("failed to search users: first_name={first_name:?} second_name={second_name:?}: {e:?}"))?;

        let mut result: Vec<SearchResult> = Vec::with_capacity(rows.len());
        for row in rows {
            let user = User::from_row(&row);
            result.push(user.into());
        }

        Ok(result)
    }

    fn from_row(row: &tokio_postgres::Row) -> Self {
        Self {
            id: row.get(0),
            first_name: row.get(1),
            second_name: row.get(2),
            birthdate: row.get(3),
            biography: row.try_get(4).unwrap_or("".into()),
            city: row.get(5),
            password_hash: row.try_get(6).unwrap_or("".into()),
            token: row.try_get(7).unwrap_or("".into()),
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
