mod app;
mod controller;
mod controller_auth;
mod controller_user;
mod db;
mod db_user;
mod password;
mod schema;

use app::App;
use clap::{arg, Parser};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() {
    init_logger();
    tracing::info!("PID {}", std::process::id());
    let args = Cli::parse().init_not_specified_opts();
    tracing::info!("opts: {:?}", args);
    if let Err(err) = App::run(
        &args.postgres_conn_string,
        &args.bind_string.unwrap(),
        args.conn_pool_size.unwrap(),
    )
    .await
    {
        eprintln!("{}", err);
    }
}

fn init_logger() {
    const LOG_LEVEL_ENV: &str = "LOG_LEVEL";
    if std::env::var(LOG_LEVEL_ENV).is_err() {
        std::env::set_var(LOG_LEVEL_ENV, "info");
    }
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env(LOG_LEVEL_ENV))
        .init();
}

#[derive(Parser, Debug, Clone)]
#[command(
    version = env!("CARGO_PKG_VERSION"),
    author = "Aleksandr Bubnov\n",
    about = "*** OTUS Highload Architect course homework ***",
    help_template = "{about}\nAuthor: {author}\nUse LOG_LEVEL=debug to see debug logging\n{usage-heading} {usage}\n{all-args}{after-help}"
)]
pub struct Cli {
    #[arg(
        long = "postgres-conn-string",
        value_name = "string",
        help = "for example, \"host=localhost user=postgres\", to specify your own DB use dbname=your_db_name otherwise DB will be created"
    )]
    postgres_conn_string: String,
    #[arg(
        long = "bind-string",
        value_name = "host:port",
        help = "optional, default value is \"127.0.0.1:8080\""
    )]
    bind_string: Option<String>,
    #[arg(
        long = "pg-pool-size",
        value_name = "<size>",
        help = "postgres connection pool size, optional, default value is 16"
    )]
    conn_pool_size: Option<usize>,
}

impl Cli {
    fn init_not_specified_opts(mut self) -> Self {
        if self.bind_string.is_none() {
            self.bind_string = Some("127.0.0.1:8080".into());
        }

        if self.conn_pool_size.is_none() {
            self.conn_pool_size = Some(16);
        }

        self
    }
}
