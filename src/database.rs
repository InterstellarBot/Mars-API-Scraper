use mysql::{prelude::Queryable, Pool, PooledConn};

use crate::ScraperConfig;

pub struct Database {
    connected: bool,
    pool: Pool,
    connection: PooledConn
}

impl Database {
    pub fn new(settings: &ScraperConfig) -> Result<Self, mysql::Error> {
        let url: String = format!("mysql://{}:{}@{}:{}/{}", settings.db_user, settings.db_pass, settings.db_host, settings.db_port, settings.db_schema);
        let pool: Pool = Pool::new(url.as_str())?;
        let connection: PooledConn = pool.get_conn()?;

        Ok(Self {
            connected: true,
            pool, 
            connection
        })
    }

    pub fn check_tables(&mut self) -> Result<(), mysql::Error> {
        self.connection.query_drop(r"CREATE TABLE IF NOT EXISTS `test`(`test` varchar(255));")?;
        Ok(())
    }
}
