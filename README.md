# Mars API Scraper
Interstellar's scraper for NASA's Mars Rover Photos, based heavily off the work of [Chris Cerami's now archived API](https://github.com/corincerami/mars-photo-api).

Note this is just the scraper - The actual web API code will be avaiable later... once it's actually written.

## Running 
The scraper uses environment variables for configuration just for ease of setting up. You can either use a .env file if your just looking to get it quickly working, or if you intend to run this as a scraper full-time you should make a systemd service that has these variables set or something. A list of all the variables you can set are in `.env.example`.

A database is required. The scraper will automatically set it up and seed it, you just need to give it credentials with CREATE, INSERT, UPDATE and SELECT priviledges. I've tested this with MariaDB, however your free to try something else.

The scraper is intended to run at a fixed interval. To prevent it hammering NASA's API (it's slow enough as it is, we dont wanna make it worse) the scraper will only fetch a maximum of `SCRAPER_MAX_SOLS` sols before moving on. If your just starting to scrape, you may want to run this scraper every ~15m or so and go to every few hours once your caught up.

For the sake's of NASA's underfunded servers, please don't disable this limit or make it too high. Let them breathe once in a while.

## Where's Spirit and Opportunity?
NASA took their images down a while ago, the original source no longer exists. If anyone has an idea on where to find them, please make an issue tyvm. :)

## Why rewrite the Ruby version?
Because I don't like Ruby, and wanted something a little more lightweight to run. 

## Credits
- Chris Cerami - The owner of the previously heavily used [Mars Photo API](https://github.com/corincerami/mars-photo-api) - A lot of this work is based off his code.
