use sqlx::SqlitePool;

pub trait PoolPassing {
    fn pool(&self) -> Option<SqlitePool>;
}
