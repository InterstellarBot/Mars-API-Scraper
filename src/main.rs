use config::Config;
use dotenv::dotenv;
use serde::Deserialize;

use crate::database::Database;

mod database;

#[derive(Debug, Deserialize)]
struct ScraperConfig {
    db_user: String,
    db_pass: String,
    db_host: String,
    db_port: u32,
    db_schema: String
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
}
