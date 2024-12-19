use sqlx::SqlitePool;

pub trait PoolPassing {
    fn pool(&self) -> SqlitePool;
}
