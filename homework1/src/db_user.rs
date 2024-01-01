use crate::db_client::DB;

#[derive(serde::Serialize)]
pub struct User {
    pub id: String,
    pub first_name: String,
    pub second_name: String,
    pub biography: String,
    pub birthdate: String,
    pub city: String,
    pub password_hash: String,
}

impl User {
    pub async fn insert_to_db(self, db: &DB) -> anyhow::Result<String> {
        let statement = "INSERT INTO users (id, first_name, second_name, birthdate, biography, city, password_hash) VALUES ($1, $2, $3, $4, $5, $6, $7)";
        db.client
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
            password_hash: String::new(),
        }
    }

    pub async fn from_db(id: String, db: &DB) -> anyhow::Result<Option<Self>> {
        let statement = "select * from users where id = $1";
        let rows = db
            .client
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
}
