use std::time::Duration;

use chrono::{DateTime, Utc};
use config::Config;
use dotenv::dotenv;
use mysql::Row;
use reqwest::blocking::{Client, ClientBuilder};
use serde::Deserialize;

use crate::{database::Database, scrapers::{perseverance::PerseveranceScraper, Scraper}};

mod database;
mod scrapers;

#[derive(Debug, Deserialize)]
struct ScraperConfig {
    db_user: String,
    db_pass: String,
    db_host: String,
    db_port: u32,
    db_schema: String
}

// Rover ID constants, used as the db pk's
const PERSEVERANCE_ID: &str = "perseverance";

// Rover Struct
#[derive(Debug)]
#[allow(dead_code)]
pub struct Rover {
    id: String,
    name: String,
    last_sol_processed: u64
}
impl Rover {
    fn from_row(row: &mut Row) -> Self {
        Self {
            id: row.take(0).expect("Failed to take column, have you updated the schema but not the code?"),
            name: row.take(1).expect("Failed to take column, have you updated the schema but not the code?"),
            last_sol_processed: row.take(2).expect("Failed to take column, have you updated the schema but not the code?"),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct RoverCamera {
    instrument_name: String,
    name: String,
    rover_id: String
}
#[derive(Debug)]
#[allow(dead_code)]
pub struct RoverImage {
    nasa_id: String,
    rover_id: String,
    instrument_name: String,
    caption: Option<String>,
    date: DateTime<Utc>,
    sol: u64,
    title: String,
    credit: Option<String>,
}

fn main() {
    // Load settings 
    dotenv().ok();
    let settings: ScraperConfig = Config::builder()
        .add_source(config::Environment::with_prefix("SCRAPER"))
        .build()
        .expect("Unable to build configuration")
        .try_deserialize::<ScraperConfig>()
        .expect("Unable to deserialize configuration");

    // Connect to our DB
    let mut db: Database = Database::new(&settings).expect("Failed to connect to database");
    db.check_tables().expect("Failed to check database tables");
    db.seed_tables().expect("Failed to seed database tables");

    // Create our request client 
    let request_client: Client = ClientBuilder::new()
        .user_agent("Interstellar Mars Photo Scraper")
        .timeout(Duration::from_secs(120)) // NASA's API is fuckin slowwwwwwww
        .build()
        .expect("Failed to create HTTP Client");

    // Scraping time 
    if let Err(e) = PerseveranceScraper::scrape(&mut db, &request_client) {
        println!("Perseverance Scraper failed to scrape: {e}");
    }
}
