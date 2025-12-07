use mysql::{params, prelude::Queryable, Pool, PooledConn, Row, Statement};

use crate::{Rover, RoverImage, ScraperConfig};

pub struct Database {
    connection: PooledConn
}

impl Database {
    pub fn new(settings: &ScraperConfig) -> Result<Self, mysql::Error> {
        let url: String = format!("mysql://{}:{}@{}:{}/{}", settings.db_user, settings.db_pass, settings.db_host, settings.db_port, settings.db_schema);
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
        self.connection.query_drop(r"INSERT IGNORE INTO `rovers` VALUES('perseverance', 'Perseverance', 0)")?;

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
