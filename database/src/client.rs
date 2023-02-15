use std::{str::FromStr, sync::Arc, time::Duration};

use log::error;
use mobc::Pool as MobcPool;
use mobc_postgres::{
    tokio_postgres::{types::ToSql, Config, NoTls, Row, ToStatement},
    PgConnectionManager,
};
use tokio_postgres::Statement;

use crate::environment;
use crate::prelude::{DatabaseConnection, DatabaseError, DatabasePool};

#[derive(Clone)]
struct Pool {
    pool: DatabasePool,
}

impl Pool {
    pub async fn new() -> Self {
        let env = environment::Env::new().await;

        let max_open: u64 = 32;
        let max_idle: u64 = 8;
        let timeout_seconds: u64 = 15;
        let config = Config::from_str(
            format!(
                "host={} port={} user={} password={}",
                env.postgres_host, env.postgres_port, env.postgres_user, env.postgres_password,
            )
            .as_str(),
        )
        .map_err(|e| error!("{:?}", e))
        .unwrap_or_default();

        let manager = PgConnectionManager::new(config, NoTls);

        let pool = MobcPool::builder()
            .max_open(max_open)
            .max_idle(max_idle)
            .get_timeout(Some(Duration::from_secs(timeout_seconds)))
            .build(manager);

        Pool { pool }
    }

    pub async fn connection(&self) -> Result<DatabaseConnection, DatabaseError> {
        self.pool.get().await.map_err(DatabaseError::DBPoolError)
    }
}

#[derive(Clone)]
pub struct Client {
    pool: Pool,
}

impl Client {
    pub async fn new() -> Self {
        let pool = Pool::new().await;

        Client { pool }
    }

    // db is considered healthy if:
    // a) connection can be made from the pool
    // b) all of the expected tables exist
    pub async fn healthy(&self) -> bool {
        // casting via ::regclass will raise an error if the table does not exist
        let query = "SELECT
            'job'::regclass,
            'guilds'::regclass,
            'users'::regclass,
            'user_guild'::regclass,
            'proof'::regclass,
            'lists'::regclass,
            'tasks'::regclass,
            'accountability_requests'::regclass";
        self.query_one(query, &[])
            .await
            .map_err(|e| error!("{:?}", e))
            .is_ok()
    }

    // batches must use the same connection from the pool, see: https://www.postgresql.org/docs/current/sql-prepare.html
    // note: these should be order independent, i.e. this should not be used in place of a transaction
    pub async fn batch_prepare(
        &self,
        conn: Arc<DatabaseConnection>,
        queries: Vec<&str>,
    ) -> Result<Vec<Statement>, tokio_postgres::Error> {
        futures::future::try_join_all(
            queries
                .iter()
                .map(|query| async { conn.prepare(query).await }),
        )
        .await
    }

    pub async fn batch_execute_shared_stmt(
        &self,
        conn: Arc<DatabaseConnection>,
        statement: Statement,
        params: Vec<&[&(dyn ToSql + Sync)]>,
    ) -> Result<Vec<u64>, tokio_postgres::Error> {
        futures::future::try_join_all(
            params
                .iter()
                .map(|param| async { conn.execute(&statement, param).await }),
        )
        .await
    }

    pub async fn batch_execute(
        &self,
        conn: Arc<DatabaseConnection>,
        statements: Vec<Statement>,
        params: Vec<&[&(dyn ToSql + Sync)]>,
    ) -> Result<Vec<u64>, tokio_postgres::Error> {
        futures::future::try_join_all(
            statements
                .iter()
                .zip(params)
                .map(|group| async { conn.execute(group.0, group.1).await }),
        )
        .await
    }

    pub async fn prepare<T>(
        &self,
        conn: Arc<DatabaseConnection>,
        query: &str,
    ) -> Result<Statement, tokio_postgres::Error> {
        conn.prepare(query).await
    }

    pub async fn query_one<T>(
        &self,
        query: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Row, DatabaseError>
    where
        T: ?Sized + ToStatement,
    {
        let conn = self.pool.connection().await?;

        conn.query_one(query, params)
            .await
            .map_err(DatabaseError::DBQueryError)
    }

    pub async fn query_opt<T>(
        &self,
        query: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<Row>, DatabaseError>
    where
        T: ?Sized + ToStatement,
    {
        let conn = self.pool.connection().await?;

        conn.query_opt(query, params)
            .await
            .map_err(DatabaseError::DBQueryError)
    }

    pub async fn query<T>(
        &self,
        query: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, DatabaseError>
    where
        T: ?Sized + ToStatement,
    {
        let conn = self.pool.connection().await?;

        conn.query(query, params)
            .await
            .map_err(DatabaseError::DBQueryError)
    }
}
