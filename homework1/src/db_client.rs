pub struct DB {
    pub client: tokio_postgres::Client,
}

const DBNAME: &str = "highload_alexander_bubnov";
const TABLE_USERS: &str = "users";
impl DB {
    pub async fn new(conn_string: &String) -> anyhow::Result<Self> {
        let need_create_db = !conn_string.to_lowercase().contains("dbname=");
        let mut client = DB::create_client(conn_string).await?;

        let dbname = if need_create_db {
            tracing::info!("database has not been specifed so let's create it");
            if DB::is_db_exists(&client).await? {
                tracing::info!(
                    "db with name '{}' already exists, so let's using it",
                    DBNAME
                );
            } else {
                DB::create_db(&client).await?;
                tracing::info!("db with name '{}' created", DBNAME);
            }
            let conn_string = conn_string.to_owned() + " dbname=" + DBNAME;
            client = DB::create_client(&conn_string).await?;
            DBNAME.to_string()
        } else {
            conn_string
                .split_whitespace()
                .find(|param| param.to_lowercase().starts_with("dbname="))
                .unwrap()
                .split('=')
                .last()
                .unwrap()
                .to_string()
        };
        tracing::info!("connected to DB with name '{}'", dbname);
        DB::apply_migrations(&client).await?;

        Ok(Self { client })
    }

    async fn create_client(conn_string: &str) -> anyhow::Result<tokio_postgres::Client> {
        let (client, connection) = tokio_postgres::connect(conn_string, tokio_postgres::NoTls)
            .await
            .map_err(|e| anyhow::anyhow!("failed to connect with params {}: {}", conn_string, e))?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(client)
    }

    async fn create_db(client: &tokio_postgres::Client) -> anyhow::Result<()> {
        client
            .query(&format!("create database {DBNAME}"), &[])
            .await
            .map_err(|e| anyhow::anyhow!("failed to create db with name {}: {}", DBNAME, e))?;

        Ok(())
    }

    async fn is_db_exists(client: &tokio_postgres::Client) -> anyhow::Result<bool> {
        let db_exists_query = format!(
            "SELECT datname FROM pg_catalog.pg_database WHERE datname = '{}'",
            DBNAME
        );

        let exists = !client
            .query(db_exists_query.as_str(), &[])
            .await
            .map_err(|e| {
                anyhow::anyhow!("failed to check if db exists \"{}\" {}", db_exists_query, e)
            })?
            .is_empty();
        Ok(exists)
    }

    async fn apply_migrations(client: &tokio_postgres::Client) -> anyhow::Result<()> {
        client
            .query(&format!("create table if not exists {TABLE_USERS} (id text PRIMARY KEY, first_name text, second_name text, birthdate text, biography text, city text, password_hash text)"), &[])
            .await
            .map_err(|e| anyhow::anyhow!("failed to create table '{}': {}", TABLE_USERS, e))?;
        tracing::info!("migrations applied");
        Ok(())
    }
}
