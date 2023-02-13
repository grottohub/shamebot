use std::{str::FromStr, time::Duration};

use log::error;
use mobc::Pool as MobcPool;
use mobc_postgres::{tokio_postgres::{Config, NoTls, types::ToSql, ToStatement, Row}, PgConnectionManager};

use crate::environment;
use crate::prelude::{DatabaseConnection, DatabaseError, DatabasePool};

struct Pool {
    pool: DatabasePool,
}

impl Pool {
    pub async fn new() -> Self {
        let env = environment::Env::new().await;

        let max_open: u64 = 32;
        let max_idle: u64 = 8;
        let timeout_seconds: u64 = 15;
        let config = Config::from_str(format!(
            "host={} port={} user={} password={}",
            env.postgres_host,
            env.postgres_port,
            env.postgres_user,
            env.postgres_password,
        ).as_str())
        .map_err(|e| error!("{:?}", e))
        .unwrap_or_default();
        
        let manager = PgConnectionManager::new(config, NoTls);

        let pool = MobcPool::builder()
            .max_open(max_open)
            .max_idle(max_idle)
            .get_timeout(Some(Duration::from_secs(timeout_seconds)))
            .build(manager);

        Pool {
            pool,
        }
    }

    pub async fn connection(&self) -> Result<DatabaseConnection, DatabaseError> {
        self.pool
            .get()
            .await
            .map_err(DatabaseError::DBPoolError)
    }
}

pub struct Client {
    pool: Pool,
}

impl Client {
    pub async fn new() -> Self {
        let pool = Pool::new().await;

        Client {
            pool,
        }
    }

    pub async fn query_one<T>(&mut self, query: &T, params: &[&(dyn ToSql + Sync)]) -> Result<Row, DatabaseError>
    where
        T: ?Sized + ToStatement,
    {
        let conn = self
            .pool
            .connection()
            .await?;
        
        conn
            .query_one(query, params)
            .await
            .map_err(DatabaseError::DBQueryError)
    }

    pub async fn query_opt<T>(&mut self, query: &T, params: &[&(dyn ToSql + Sync)]) -> Result<Option<Row>, DatabaseError>
    where
        T: ?Sized + ToStatement,
    {
        let conn = self
            .pool
            .connection()
            .await?;
        
        conn
            .query_opt(query, params)
            .await
            .map_err(DatabaseError::DBQueryError)
    }

    pub async fn query<T>(&mut self, query: &T, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>, DatabaseError>
    where
        T: ?Sized + ToStatement,
    {
        let conn = self
            .pool
            .connection()
            .await?;
        
        conn
            .query(query, params)
            .await
            .map_err(DatabaseError::DBQueryError)
    }
}
