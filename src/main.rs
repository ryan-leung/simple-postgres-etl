use dotenv::dotenv;
use postgres::{Client, NoTls};
use serde_json::Value;
use std::{error, thread, time};
use ureq::{Agent, AgentBuilder, Error};

fn main() -> Result<(), Box<dyn error::Error>> {
    dotenv().ok();
    // Config
    let db_url = std::env::var("DATABASE_URL").expect("Unable to read DATABASE_URL env var");
    let data_refresh: u64 = std::env::var("DATA_REFRESH")
        .unwrap()
        .parse::<u64>()
        .unwrap();
    // DB Client

    let mut client = Client::connect(&db_url, NoTls)?;

    client.batch_execute(
        "
        CREATE TABLE IF NOT EXISTS test_data (
            id      SERIAL PRIMARY KEY,
            data    JSONB,
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    ",
    )?;

    // HTTP Agent
    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(time::Duration::from_secs(5))
        .timeout_write(time::Duration::from_secs(5))
        .build();

    loop {
        // Fetch Data
        let body: String = agent
            .get("https://postman-echo.com/get")
            .set("Example-Header", "header value")
            .query("hello", "world")
            .call()?
            .into_string()?;

        // Prepare payload to database
        let payload: Value = serde_json::from_str(&body).unwrap();

        // Execute query
        let row = client.execute(
            "INSERT INTO test_data (data) VALUES ($1) returning id",
            &[&payload],
        )?;
        println!("{}", row);

        // Next Run
        thread::sleep(time::Duration::from_secs(data_refresh as u64))
    }
    Ok(())
}
