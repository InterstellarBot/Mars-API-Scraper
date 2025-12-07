use std::{cmp::min, io::{self, Read}};

use chrono::Utc;
use reqwest::blocking::Client;
use serde_json::{Map, Value};

use crate::{database::Database, scrapers::Scraper, Rover, RoverImage, PERSEVERANCE_ID};

const RSS_BASE_URL: &str = "https://mars.nasa.gov/rss/api/";

pub struct PerseveranceScraper;

impl Scraper for PerseveranceScraper {
    fn scrape(db: &mut Database, client: &Client) -> Result<(), String> {
        let rover: Rover = match db.get_rover(PERSEVERANCE_ID) {
            Ok(r) => match r {
                Some(r) => r,
                None => return Err(String::from("Failed to find rover.")),
            }
            Err(e) => return Err(e.to_string()),
        };

        // find the latest sol
        let mut sol_count_request = match client.get(format!("{RSS_BASE_URL}?feed=raw_images&category=mars2020&feedtype=json&latest=true")).send() {
            Ok(r) => r,
            Err(e) => return Err(e.to_string())
        };
        if !sol_count_request.status().is_success() {
            return Err(String::from("Sol count request failed."));
        }
        let mut sol_count_body: String = String::new();
        let sol_read_result: Result<usize, io::Error> = sol_count_request.read_to_string(&mut sol_count_body);
        if let Err(err) = sol_read_result {
            return Err(err.to_string())
        }
        // parsing
        let sol_body_parsed: Value = match serde_json::from_str(&sol_count_body) {
            Ok(r) => r,
            Err(e) => return Err(e.to_string())
        };
        let latest_sol: u64 = match sol_body_parsed["latest_sol"].as_u64() {
            Some(r) => r,
            None => return Err(String::from("Can't find latest SOL")),
        };

        // get *our* last processed
        let last_processed: u64 = rover.last_sol_processed;
        if last_processed >= latest_sol {
            return Ok(()); // nothin to do
        }

        let mut current_sol: u64 = last_processed;
        let end: u64 = min(last_processed + 10, latest_sol);
        while current_sol < end {
            current_sol += 1;
            // request all the images for the current sol
            let mut request = match client.get(format!("{RSS_BASE_URL}?feed=raw_images&category=mars2020&feedtype=json&sol={current_sol}")).send() {
                Ok(r) => r,
                Err(e) => return Err(e.to_string())
            };
            if !request.status().is_success() {
                return Err(String::from("Sol count request failed."));
            }
            let mut body: String = String::new();
            let read_result: Result<usize, io::Error> = request.read_to_string(&mut body);
            if let Err(err) = read_result {
                return Err(err.to_string())
            }

            // parse it
            let body_parsed: Value = match serde_json::from_str(&body) {
                Ok(r) => r,
                Err(e) => return Err(e.to_string())
            };
            let images: &Vec<Value> = match body_parsed["images"].as_array() {
                Some(r) => r,
                None => continue
            };
            for image_value in images {
                let image_object: &Map<String, Value> = match image_value.as_object() {
                    Some(r) => r,
                    None => continue
                };

                let image: RoverImage = RoverImage { 
                    nasa_id: image_object["imageid"].to_string(),
                    rover_id: rover.id.to_string(),
                    instrument_name: image_object["camera"]["instrument"].to_string(),
                    caption: image_object["caption"].as_str().map(|caption| caption.to_string()),
                    date: Utc::now(),
                    sol: current_sol,
                    title: image_object["title"].to_string(),
                    credit: image_object["credit"].as_str().map(|credit| credit.to_string())
                };
                println!("{:?}", db.save_image(image))
            }

            if let Err(e) = db.update_sol_processed(&rover.id, current_sol) {
                return Err(e.to_string());
            }
        }

        Ok(())
    }
}
