use core::fmt;
use redis::Client;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct AppState {
    pub start_time: Instant,
    pub total_requests: Arc<AtomicU64>,
    pub total_bytes: Arc<AtomicU64>,
    pub max_body_bytes: usize,
    pub max_concurrent_requests: usize,
    pool: MySqlPool,
    redis_client: Client,
}

impl fmt::Display for AppState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Uptime: {} micros, Total Requests: {}, Total Bytes: {}",
            self.uptime_micros(),
            self.total_requests.load(Ordering::Relaxed),
            self.total_bytes.load(Ordering::Relaxed)
        )
    }
}

impl AppState {
    pub async fn new_with_db(
        max_body_bytes: usize,
        max_concurrent_requests: usize,
    ) -> anyhow::Result<Self> {
        let db_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL not set in .env"))?;
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        let redis_url =
            std::env::var("REDIS_URL").map_err(|_| anyhow::anyhow!("REDIS_URL not set in .env"))?;
        let redis_client = redis::Client::open(redis_url)?;
        Ok(AppState {
            start_time: Instant::now(),
            total_requests: Arc::new(AtomicU64::new(0)),
            total_bytes: Arc::new(AtomicU64::new(0)),
            max_body_bytes,
            max_concurrent_requests,
            pool,
            redis_client,
        })
    }

    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }

    pub fn redis_client(&self) -> &Client {
        &self.redis_client
    }

    pub fn increment_requests(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_bytes(&self, bytes: u64) {
        self.total_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn uptime_micros(&self) -> u128 {
        self.start_time.elapsed().as_micros()
    }

    // pub fn get_total_requests(&self) -> u64 {
    //     self.total_requests.load(Ordering::Relaxed)
    // }

    // pub fn get_total_bytes(&self) -> u64 {
    //     self.total_bytes.load(Ordering::Relaxed)
    // }

    pub fn get_max_concurrent_requests(&self) -> usize {
        self.max_concurrent_requests
    }

    pub fn get_max_body_bytes(&self) -> usize {
        self.max_body_bytes
    }
}
