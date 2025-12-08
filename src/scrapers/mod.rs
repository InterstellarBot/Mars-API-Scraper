use reqwest::blocking::Client;

use crate::{database::Database, ScraperConfig};

pub mod perseverance;
pub mod curiosity;

pub trait Scraper {
    fn scrape(db: &mut Database, client: &Client, config: &ScraperConfig) -> Result<(), String>;
}
