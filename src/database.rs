use mysql::{prelude::Queryable, Pool, PooledConn};

use crate::ScraperConfig;

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
            `name` VARCHAR(255)
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
            `caption` VARCHAR(255),
            `date` DATETIME NOT NULL,
            `sol` INT UNSIGNED NOT NULL,
            `title` VARCHAR(255) NOT NULL,
            `credit` VARCHAR(255)
        );")?;

        Ok(())
    }
}
