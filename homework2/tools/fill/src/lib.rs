use chrono::Datelike;
use std::fs::File;
use std::io::{self, BufRead};

pub async fn run(filename: &str, conn_string: &str) -> anyhow::Result<()> {
    let start_time = std::time::Instant::now();
    let lines = read_lines(filename)
        .map_err(|e| anyhow::anyhow!("failed to open file {}: {:?}", filename, e))?;
    let now = chrono::Local::now();
    let mut count = 1;
    let (client, connection) = tokio_postgres::connect(conn_string, tokio_postgres::NoTls)
        .await
        .map_err(|e| anyhow::anyhow!("failed to connect with params {}: {}", conn_string, e))?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    for line in lines.flatten() {
        let items = line.split(',').collect::<Vec<_>>();
        if items.len() != 3 {
            anyhow::bail!(
                "wrong number of items on line {}: {} != 3",
                count,
                items.len()
            );
        }

        let age = items[1].parse::<u32>().map_err(|e| {
            anyhow::anyhow!("failed to convert item1 '{}' to usize: {:?}", items[1], e)
        })?;
        let birthdate = now - chrono::Months::new(12 * age);
        let birthdate = format!(
            "{:04}-{:02}-{:02}",
            birthdate.year(),
            birthdate.month(),
            birthdate.day()
        );

        let mut fullname = items[0].split(' ');
        let first_name = fullname.next().unwrap_or("<first name was not specified>");
        let second_name = fullname.next().unwrap_or("<second name was not specified>");

        if let Err(err) = insert_to_db(&client, first_name, second_name, &birthdate).await {
            tracing::error!(
                "failed to insert ({count}) to {conn_string}, {first_name}, {second_name}, {birthdate}, err = {err:?}"
            );
        }
        count += 1;
    }

    tracing::info!(
        "time spent = {:?}, lines = {}",
        start_time.elapsed(),
        count - 1
    );
    Ok(())
}

pub async fn insert_to_db(
    client: &tokio_postgres::Client,
    first_name: &str,
    second_name: &str,
    birthdate: &str,
) -> anyhow::Result<()> {
    let statement = "INSERT INTO users (id, first_name, second_name, birthdate, biography, city, password_hash, token) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";
    let id = uuid::Uuid::new_v4().to_string();
    client
        .execute(
            statement,
            &[
                &id,
                &first_name,
                &second_name,
                &birthdate,
                &"",
                &"Moscow",
                &"",
                &"",
            ],
        )
        .await
        .map_err(|e| anyhow::anyhow!("insert failed: {}", e))?;
    Ok(())
}

fn read_lines(filename: &str) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::with_capacity(1024 * 1024 * 500, file).lines())
}
