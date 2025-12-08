use std::{cmp::min, io::{self, Read}, str::FromStr};

use chrono::{DateTime, Utc};
use log::{debug, error, trace};
use reqwest::blocking::Client;
use serde_json::{Map, Value};

use crate::{database::Database, scrapers::Scraper, Rover, RoverImage, ScraperConfig, CURIOSITY_ID};

const API_BASE_URL: &str = "https://mars.nasa.gov/api/v1/raw_image_items/";

pub struct CuriosityScraper;

impl Scraper for CuriosityScraper {
    fn scrape(db: &mut Database, client: &Client, config: &ScraperConfig) -> Result<(), String> {
        let rover: Rover = match db.get_rover(CURIOSITY_ID) {
            Ok(r) => match r {
                Some(r) => r,
                None => return Err(String::from("Failed to find rover.")),
            }
            Err(e) => return Err(e.to_string()),
        };
        trace!("Found rover for Curiosity");

        // find the latest sol
        let mut latest_sol_request = match client.get(format!("{API_BASE_URL}?order=sol desc&condition_1=msl:mission&per_page=1")).send() {
            Ok(r) => r,
            Err(e) => return Err(e.to_string())
        };
        if !latest_sol_request.status().is_success() {
            return Err(String::from("Latest Sol image request failed."));
        }
        trace!("Made request for latest SOL image");
        let mut latest_sol_body: String = String::new();
        let latest_sol_read_result: Result<usize, io::Error> = latest_sol_request.read_to_string(&mut latest_sol_body);
        if let Err(err) = latest_sol_read_result {
            return Err(err.to_string())
        }
        trace!("{latest_sol_body}");
        // parsing
        let latest_sol_body_parsed: Value = match serde_json::from_str(&latest_sol_body) {
            Ok(r) => r,
            Err(e) => return Err(e.to_string())
        };
        let latest_sol: u64 = match latest_sol_body_parsed["items"][0]["sol"].as_u64() {
            Some(r) => r,
            None => return Err(String::from("Can't find latest SOL")),
        };
        debug!("Latest Remote Curiosity Sol: {latest_sol}");

        // get *our* last processed
        let last_processed: u64 = rover.last_sol_processed;
        debug!("Latest Processed Curiosity Sol: {last_processed}");
        if last_processed >= latest_sol {
            debug!("Latest processed is >= remote, nothing to do");
            return Ok(()); // nothin to do
        }

        let mut current_sol: u64 = last_processed;
        let end: u64 = min(last_processed + (config.max_sols.unwrap() as u64), latest_sol);
        debug!("Processing from sol {current_sol} to sol {end}");
        while current_sol < end {
            current_sol += 1;

            // request all the images for the current sol
            trace!("Running sol {current_sol}");
            let mut request = match client.get(format!("{API_BASE_URL}?order=sol desc&condition_1=msl:mission&condition_2={current_sol}:sol")).send() {
                Ok(r) => r,
                Err(e) => return Err(e.to_string())
            };
            if !request.status().is_success() {
                return Err(String::from("Sol request failed."));
            }
            trace!("Made request successfully");
            let mut body: String = String::new();
            let read_result: Result<usize, io::Error> = request.read_to_string(&mut body);
            if let Err(err) = read_result {
                return Err(err.to_string())
            }
            trace!("{body}");

            // parse it
            let body_parsed: Value = match serde_json::from_str(&body) {
                Ok(r) => r,
                Err(e) => return Err(e.to_string())
            };
            let images: &Vec<Value> = match body_parsed["items"].as_array() {
                Some(r) => r,
                None => continue
            };
            trace!("Got {} images", images.len());
            for image_value in images {
                let image_object: &Map<String, Value> = match image_value.as_object() {
                    Some(r) => r,
                    None => continue
                };

                let nasa_id: String = image_object["id"].to_string();
                let date_to_parse: String = match image_object["date_taken"].as_str() {
                    Some(r) => r.to_string(),
                    None => {
                        error!("Failed to take date for {}", &nasa_id);
                        error!("Skipping Image...");
                        continue;
                    },
                };
                // this stupid dumb function needs a timezone :/
                let date: DateTime<Utc> = match DateTime::parse_from_str(&format!("{date_to_parse} +00:00"), "%Y-%m-%dT%H:%M:%S%.3fZ %z") {
                    Ok(r) => r.to_utc(),
                    Err(e) => {
                        error!("Failed to parse date for {} ({date_to_parse}) -> {e}", &nasa_id);
                        error!("Skipping Image...");
                        continue;
                    },
                };

                let image: RoverImage = RoverImage { 
                    nasa_id: nasa_id.clone(),
                    rover_id: rover.id.to_string(),
                    instrument_name: image_object["instrument"].to_string(),
                    image_url: image_object["url"].to_string(),
                    caption: image_object["description"].as_str().map(|caption| caption.to_string()),
                    date,
                    sol: current_sol,
                    title: image_object["title"].to_string(),
                    credit: image_object["image_credit"].as_str().map(|credit| credit.to_string())
                };

                let save_result: Result<(), mysql::Error> = db.save_image(image);
                trace!("Image ID: {nasa_id} -> {save_result:?}");
                if let Err(e) = save_result {
                    error!("Failed to save image {nasa_id}: {e}");
                }
            }

            if let Err(e) = db.update_sol_processed(&rover.id, current_sol) {
                return Err(e.to_string());
            }
            debug!("Completed SOL {current_sol}");
        }

        Ok(())
    }
}
