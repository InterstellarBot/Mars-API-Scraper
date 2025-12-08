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
        self.connection.query_drop(r"CREATE TABLE IF NOT EXISTS `rover_cameras`(
            `rover_id` VARCHAR(255) NOT NULL,
            `instrument_name` VARCHAR(255) NOT NULL,
            `name` VARCHAR(255),
            PRIMARY KEY(`rover_id`, `instrument_name`)
        );")?;
        self.connection.query_drop(r"CREATE TABLE IF NOT EXISTS `rover_images`(
            `nasa_id` VARCHAR(255) NOT NULL PRIMARY KEY,
            `rover_id` VARCHAR(255) NOT NULL,
            `camera_instrument_name` VARCHAR(255) NOT NULL,
            `caption` VARCHAR(1024),
            `timestamp` INT UNSIGNED NOT NULL,
            `sol` INT UNSIGNED NOT NULL,
            `title` VARCHAR(1024) NOT NULL,
            `credit` VARCHAR(1024)
        );")?;

        Ok(())
    }

    pub fn seed_tables(&mut self) -> Result<(), mysql::Error> {
        self.connection.query_drop(format!("INSERT IGNORE INTO `rovers` VALUES('{PERSEVERANCE_ID}', 'Perseverance', 0)"))?;
        // Primarially taken from https://liquidgalaxylab.github.io/LG-Space-Visualizations/utils_constants/cameras.html
        // Checked over (and placed in-order with) with https://github.com/corincerami/mars-photo-api?tab=readme-ov-file#perseverance-rover
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{PERSEVERANCE_ID}', 'EDL_RUCAM', 'Rover Up-Look Camera')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{PERSEVERANCE_ID}', 'EDL_RDCAM', 'Rover Down-Look Camera')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{PERSEVERANCE_ID}', 'EDL_DDCAM', 'Descent Stage Down-Look Camera')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{PERSEVERANCE_ID}', 'EDL_PUCAM1', 'Parachute Up-Look Camera A')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{PERSEVERANCE_ID}', 'EDL_PUCAM2', 'Parachute Up-Look Camera B')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{PERSEVERANCE_ID}', 'NAVCAM_LEFT', 'Navigation Camera - Left')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{PERSEVERANCE_ID}', 'NAVCAM_RIGHT', 'Navigation Camera - Right')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{PERSEVERANCE_ID}', 'MCZ_RIGHT', 'Mast Camera Zoom - Right')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{PERSEVERANCE_ID}', 'MCZ_LEFT', 'Mast Camera Zoom - Left')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{PERSEVERANCE_ID}', 'FRONT_HAZCAM_LEFT_A', 'Front Hazard Camera - Left')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{PERSEVERANCE_ID}', 'FRONT_HAZCAM_RIGHT_A', 'Front Hazard Camera - Right')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{PERSEVERANCE_ID}', 'REAR_HAZCAM_LEFT', 'Rear Hazard Camera - Left')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{PERSEVERANCE_ID}', 'REAR_HAZCAM_RIGHT', 'Rear Hazard Camera - Right')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{PERSEVERANCE_ID}', 'SKYCAM', 'MEDA Skycam')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{PERSEVERANCE_ID}', 'SHERLOC_WATSON', 'Sherloc Watson Camera')"))?;
        // This is in the Liquid Galaxy Lab but not Chris's Camera list - Commented out for now
        // self.connection.query_drop(r"INSERT IGNORE INTO `rover_cameras` VALUES('perseverance', 'SUPERCAM_RMI', 'SuperCam Micro Imager')")?;

        self.connection.query_drop(format!("INSERT IGNORE INTO `rovers` VALUES('{CURIOSITY_ID}', 'Curiosity', 0)"))?;
        // https://github.com/corincerami/mars-photo-api?tab=readme-ov-file#other-rovers
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{CURIOSITY_ID}', 'FHAZ', 'Front Hazard Avoidance Camera')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{CURIOSITY_ID}', 'RHAZ', 'Rear Hazard Avoidance Camera')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{CURIOSITY_ID}', 'MAST', 'Mast Camera')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{CURIOSITY_ID}', 'CHEMCAM', 'Chemistry and Camera Complex')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{CURIOSITY_ID}', 'MAHLI', 'Mars Hand Lens Imager')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{CURIOSITY_ID}', 'MARDI', 'Mars Descent Imager')"))?;
        self.connection.query_drop(format!("INSERT IGNORE INTO `rover_cameras` VALUES('{CURIOSITY_ID}', 'NAVCAM', 'Navigation Camera')"))?;

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
        let statement: Statement = self.connection.prep("INSERT INTO `rover_images` VALUES(:nasa_id, :rover_id, :instrument_name, :caption, :date, :sol, :title, :credit)")?;
        self.connection.exec_drop(&statement, params!{ 
            "nasa_id" => rover_image.nasa_id,
            "rover_id" => rover_image.rover_id,
            "instrument_name" => rover_image.instrument_name,
            "caption" => rover_image.caption,
            "date" => rover_image.date.timestamp(),
            "sol" => rover_image.sol,
            "title" => rover_image.title,
            "credit" => rover_image.credit
        })?;

        Ok(())
    }
}
