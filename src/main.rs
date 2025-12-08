use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use config::Config;
use dotenv::dotenv;
use log::{error, info};
use mysql::Row;
use reqwest::blocking::{Client, ClientBuilder};
use serde::Deserialize;
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, TermLogger, TerminalMode};

use crate::{database::Database, scrapers::{perseverance::PerseveranceScraper, Scraper}};

mod database;
mod scrapers;

#[derive(Debug, Deserialize)]
struct ScraperConfig {
    db_user: String,
    db_pass: String,
    db_host: String,
    db_port: u32,
    db_schema: String,

    log_level: Option<String>,
    max_sols: Option<u32>
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
    let mut config: ScraperConfig = Config::builder()
        .add_source(config::Environment::with_prefix("SCRAPER"))
        .build()
        .expect("Unable to build configuration")
        .try_deserialize::<ScraperConfig>()
        .expect("Unable to deserialize configuration");
    // optional defaults
    if config.log_level.is_none() {
        config.log_level = Some("information".to_string());
    }
    if config.max_sols.is_none() {
        config.max_sols = Some(15);
    }

    let filter: LevelFilter = match config.log_level.clone().unwrap().to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info
    };

    // Logging 
    CombinedLogger::init(vec![
        TermLogger::new(filter, simplelog::Config::default(), TerminalMode::Mixed, ColorChoice::Auto)
    ]).expect("Failed to create logger.");

    // Connect to our DB
    info!("Connecting to the database...");
    let mut db: Database = Database::new(&config).expect("Failed to connect to database");
    info!("Checking/seeding tables as needed");
    db.check_tables().expect("Failed to check database tables");
    db.seed_tables().expect("Failed to seed database tables");

    // Create our request client 
    let request_client: Client = ClientBuilder::new()
        .user_agent("Interstellar Mars Photo Scraper")
        .timeout(Duration::from_secs(120)) // NASA's API is fuckin slowwwwwwww
        .build()
        .expect("Failed to create HTTP Client");

    // Scraping time 
    let perseverance_time: Instant = Instant::now();
    info!("Running Perseverance Scraper...");
    if let Err(e) = PerseveranceScraper::scrape(&mut db, &request_client, &config) {
        error!("Perseverance Scraper failed to scrape: {e}");
    }
    info!("Perseverance Scraper finished in {:?}", perseverance_time.elapsed())
}
