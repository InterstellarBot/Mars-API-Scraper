use reqwest::blocking::Client;

use crate::database::Database;

pub mod perseverance;

pub trait Scraper {
    fn scrape(db: &mut Database, client: &Client) -> Result<(), String>;
}
