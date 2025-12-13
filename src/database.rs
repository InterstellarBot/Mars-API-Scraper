use std::time::Instant;

use log::{error, info, trace, warn};
use mysql::{params, prelude::Queryable, Pool, PooledConn, Row, Statement};

use crate::{Rover, RoverImage, ScraperConfig, CURIOSITY_ID, PERSEVERANCE_ID};

pub struct Database {
    connection: PooledConn
}

impl Database {
    pub fn new(config: &ScraperConfig) -> Result<Self, mysql::Error> {
        let url: String = format!("mysql://{}:{}@{}:{}/{}", config.db_user, config.db_pass, config.db_host, config.db_port, config.db_schema);
        let pool: Pool = Pool::new(url.as_str())?;
        let connection: PooledConn = pool.get_conn()?;

        Ok(Self {connection})
    }

    pub fn check_tables(&mut self) -> Result<(), mysql::Error> {
        self.connection.query_drop(r"CREATE TABLE IF NOT EXISTS `rovers`(
            `id` VARCHAR(255) NOT NULL PRIMARY KEY,
            `name` VARCHAR(255),
            `last_sol_processed` INT UNSIGNED
        )")?;
        // Camera ID is just "{rover_id}-{instrument_name}"
        // It's to keep our website's eloquent happy - It used to just be a compound between the two
        self.connection.query_drop(r"CREATE TABLE IF NOT EXISTS `rover_cameras`(
            `camera_id` VARCHAR(255) NOT NULL PRIMARY KEY,
            `rover_id` VARCHAR(255) NOT NULL,
            `instrument_name` VARCHAR(255) NOT NULL,
            `name` VARCHAR(255)
        );")?;
        self.connection.query_drop(r"CREATE TABLE IF NOT EXISTS `rover_images`(
            `nasa_id` VARCHAR(255) NOT NULL PRIMARY KEY,
            `rover_id` VARCHAR(255) NOT NULL,
            `camera_instrument_name` VARCHAR(255) NOT NULL,
            `image_url` VARCHAR(1024) NOT NULL,
            `caption` VARCHAR(8196),
            `timestamp` INT UNSIGNED NOT NULL,
            `sol` INT UNSIGNED NOT NULL,
            `title` VARCHAR(1024) NOT NULL,
            `credit` VARCHAR(1024)
        );")?;

        // indexes
        self.connection.query_drop("CREATE INDEX image_rover_id ON `rover_images`(`rover_id`)")?;
        self.connection.query_drop("CREATE INDEX image_camera ON `rover_images`(`camera_id`)")?;
        self.connection.query_drop("CREATE INDEX image_sol ON `rover_images`(`sol`)")?;

        Ok(())
    }

    // "Migrations" lol
    pub fn migrate_tables(&mut self) -> Result<(), mysql::Error> {
        // Add camera_id to the images table and alter all images to 
        self.connection.query_drop("ALTER TABLE `rover_images` ADD COLUMN IF NOT EXISTS `camera_id` VARCHAR(255) NOT NULL;")?;
        let rows_to_fix: Vec<Row> = self.connection.query("SELECT `nasa_id`, `rover_id`, `camera_instrument_name` FROM `rover_images` WHERE `camera_id` = '';")?;
        let statement: Statement = self.connection.prep("UPDATE `rover_images` SET `camera_id` = :camera_id WHERE `nasa_id` = :nasa_id;")?;
        if !rows_to_fix.is_empty() {
            warn!("Detected {} rows without Camera ID's - Migrating, this may take a while...", rows_to_fix.len());
            let time: Instant = Instant::now();

            self.start_transaction()?;

            for row in rows_to_fix {
                // this is a pk, it should be here lol
                let nasa_id: String = row.get(0).expect("can't find row's NASA ID... which is meant to be the primary key. what the fuck did you do?");
                trace!("Fixing camera ID for {nasa_id}");

                let rover_id: String = match row.get(1) {
                    Some(r) => r,
                    None => {
                        // bruh
                        error!("Row has no Rover ID??? Skipping???????????");
                        continue;
                    },
                };
                let camera_instrument_name: String = match row.get(2) {
                    Some(r) => r,
                    None => {
                        // bruh
                        error!("Row has no Camera Instrument Name??? Skipping???????????");
                        continue;
                    },
                };
                let camera_id = format!("{}-{}", rover_id.to_lowercase(), camera_instrument_name.to_lowercase());
                self.connection.exec_drop(&statement, params!{
                    "camera_id" => camera_id,
                    "nasa_id" => nasa_id
                })?;
            }

            self.end_transaction()?;

            info!("Migrated in {:?}", time.elapsed());
        }

        Ok(())
    }

    pub fn seed_tables(&mut self) -> Result<(), mysql::Error> {
        self.connection.query_drop(format!("INSERT IGNORE INTO `rovers` VALUES('{PERSEVERANCE_ID}', 'Perseverance', 0)"))?;
        // Primarially taken from https://liquidgalaxylab.github.io/LG-Space-Visualizations/utils_constants/cameras.html
        // Checked over (and placed in-order with) with https://github.com/corincerami/mars-photo-api?tab=readme-ov-file#perseverance-rover
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('perseverance-edl_rucam', '{PERSEVERANCE_ID}', 'EDL_RUCAM', 'Rover Up-Look Camera')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('perseverance-edl_rdcam', '{PERSEVERANCE_ID}', 'EDL_RDCAM', 'Rover Down-Look Camera')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('perseverance-edl_ddcam', '{PERSEVERANCE_ID}', 'EDL_DDCAM', 'Descent Stage Down-Look Camera')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('perseverance-edl_pucam1', '{PERSEVERANCE_ID}', 'EDL_PUCAM1', 'Parachute Up-Look Camera A')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('perseverance-edl_pucam2', '{PERSEVERANCE_ID}', 'EDL_PUCAM2', 'Parachute Up-Look Camera B')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('perseverance-navcam_left', '{PERSEVERANCE_ID}', 'NAVCAM_LEFT', 'Navigation Camera - Left')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('perseverance-navcam_right', '{PERSEVERANCE_ID}', 'NAVCAM_RIGHT', 'Navigation Camera - Right')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('perseverance-mcz_right', '{PERSEVERANCE_ID}', 'MCZ_RIGHT', 'Mast Camera Zoom - Right')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('perseverance-mcz_left', '{PERSEVERANCE_ID}', 'MCZ_LEFT', 'Mast Camera Zoom - Left')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('perseverance-front_hazcam_left_a', '{PERSEVERANCE_ID}', 'FRONT_HAZCAM_LEFT_A', 'Front Hazard Camera - Left')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('perseverance-front_hazman_right_a', '{PERSEVERANCE_ID}', 'FRONT_HAZCAM_RIGHT_A', 'Front Hazard Camera - Right')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('perseverance-rear_hazcam_left', '{PERSEVERANCE_ID}', 'REAR_HAZCAM_LEFT', 'Rear Hazard Camera - Left')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('perseverance-rear_hazcam_right', '{PERSEVERANCE_ID}', 'REAR_HAZCAM_RIGHT', 'Rear Hazard Camera - Right')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('perseverance-skycam', '{PERSEVERANCE_ID}', 'SKYCAM', 'MEDA Skycam')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('perseverance-sherloc_watson', '{PERSEVERANCE_ID}', 'SHERLOC_WATSON', 'Sherloc Watson Camera')"))?;
        // This is in the Liquid Galaxy Lab but not Chris's Camera list - Commented out for now
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('perseverance-supercam_rmi', '{PERSEVERANCE_ID}', 'SUPERCAM_RMI', 'SuperCam Micro Imager')"))?;

        self.connection.query_drop(format!("INSERT IGNORE INTO `rovers` VALUES('{CURIOSITY_ID}', 'Curiosity', 0)"))?;
        // https://github.com/corincerami/mars-photo-api?tab=readme-ov-file#other-rovers
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('curiosity-fhaz', '{CURIOSITY_ID}', 'FHAZ', 'Front Hazard Avoidance Camera')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('curiosity-rhaz', '{CURIOSITY_ID}', 'RHAZ', 'Rear Hazard Avoidance Camera')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('curiosity-mast_left', '{CURIOSITY_ID}', 'MAST_LEFT', 'Mast Camera Left')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('curiosity-mast_right', '{CURIOSITY_ID}', 'MAST_RIGHT', 'Mast Camera Right')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('curiosity-chemcam', '{CURIOSITY_ID}', 'CHEMCAM', 'Chemistry and Camera Complex')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('curiosity-mahli', '{CURIOSITY_ID}', 'MAHLI', 'Mars Hand Lens Imager')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('curiosity-mardi', '{CURIOSITY_ID}', 'MARDI', 'Mars Descent Imager')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('curiosity-navcam', '{CURIOSITY_ID}', 'NAVCAM', 'Navigation Camera')"))?;

        Ok(())
    }

    pub fn get_rover(&mut self, rover_id: &str) -> Result<Option<Rover>, mysql::Error> {
        let statement: Statement = self.connection.prep("SELECT * FROM `rovers` WHERE `id` = :rover_id")?;
        let result: Option<Row> = self.connection.exec_first(&statement, params!{ "rover_id" => rover_id })?;
        if let Some(mut row) = result {
            return Ok(Some(Rover::from_row(&mut row)));
        }

        Ok(None)
    }
    pub fn update_sol_processed(&mut self, rover_id: &str, sol: u64) -> Result<(), mysql::Error>  {
        let statement: Statement = self.connection.prep("UPDATE `rovers` SET `last_sol_processed` = :sol WHERE `id` = :rover_id")?;
        self.connection.exec_drop(&statement, params!{ "sol" => sol, "rover_id" => rover_id })?;

        Ok(())
    }

    pub fn save_image(&mut self, rover_image: RoverImage) -> Result<(), mysql::Error> {
        let camera_id = format!("{}-{}", rover_image.rover_id.to_lowercase(), rover_image.instrument_name.to_lowercase());
        let statement: Statement = self.connection.prep("INSERT INTO `rover_images` VALUES(:nasa_id, :rover_id, :instrument_name, :image_url, :caption, :date, :sol, :title, :credit, :camera_id)")?;
        self.connection.exec_drop(&statement, params!{ 
            "nasa_id" => rover_image.nasa_id,
            "rover_id" => rover_image.rover_id,
            "instrument_name" => rover_image.instrument_name,
            "image_url" => rover_image.image_url,
            "caption" => rover_image.caption,
            "date" => rover_image.date.timestamp(),
            "sol" => rover_image.sol,
            "title" => rover_image.title,
            "credit" => rover_image.credit,
            "camera_id" => camera_id,
        })?;

        Ok(())
    }

    pub fn start_transaction(&mut self) -> Result<(), mysql::Error> {
        self.connection.query_drop("START TRANSACTION;")?;
        Ok(())
    }
    pub fn end_transaction(&mut self) -> Result<(), mysql::Error> {
        self.connection.query_drop("COMMIT;")?;
        Ok(())
    }
}
