use reqwest::blocking::Client;

use crate::{ScraperConfig, database::Database};

pub mod curiosity;
pub mod perseverance;

pub trait Scraper {
    fn scrape(db: &mut Database, client: &Client, config: &ScraperConfig) -> Result<(), String>;
}
