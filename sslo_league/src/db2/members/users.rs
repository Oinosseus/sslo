macro_rules! tablename {
    () => { "users" };
}
pub(self) use tablename;

mod row;
mod item;
mod table;


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use sqlx::SqlitePool;
    use super::*;
    use sslo_lib::db::PoolPassing;
    use std::sync::{Arc, Weak};

    struct TestPoolPasser {
        pub pool: SqlitePool,
        pub pool_ref_2me: Weak<dyn PoolPassing>,
    }
    impl TestPoolPasser {
        fn new() -> Arc<Self> {
            let sqlite_opts = SqliteConnectOptions::from_str(":memory:").unwrap();
            let pool = SqlitePoolOptions::new()
                .min_connections(1)
                .max_connections(1)  // default is 10
                .idle_timeout(None)
                .max_lifetime(None)
                .connect_lazy_with(sqlite_opts);
            Arc::new_cyclic(|me: &Weak<Self>| {
                Self{pool, pool_ref_2me: me.clone()}
            })
        }
        async fn init(&self) {
            sqlx::migrate!("../rsc/db_migrations/league_members").run(&self.pool).await.unwrap();
            sqlx::query(concat!("INSERT INTO ", tablename!(), " (name, email) VALUES ($1, $2);"))
                .bind("username")
                .bind("user@email.tld")
                .execute(&self.pool)
                .await.unwrap();
        }
    }
    impl PoolPassing for TestPoolPasser {
        fn pool(&self) -> Option<SqlitePool> {Some(self.pool.clone())}
    }

    #[tokio::test]
    async fn test_item() {

        // create test table
        let pool_passer = TestPoolPasser::new();
        pool_passer.init().await;
        let ref_pool_passer: Arc<dyn PoolPassing> = pool_passer.clone();
        let tbl = table::Table::new(Arc::downgrade(&ref_pool_passer));

        // test failed retrieval
        let i = tbl.item_by_id(999).await;
        assert!(i.is_none());
        let i = tbl.item_by_email("a.b@c.de").await;
        assert!(i.is_none());

        // test retrieval by id
        let i = tbl.item_by_id(1).await.unwrap();
        assert_eq!(i.id(), 1);

        // test retrieval by email
        let i = tbl.item_by_email("user@email.tld").await.unwrap();
        assert_eq!(i.id(), 1);

        // test case-insensitive retrieval
        let i = tbl.item_by_email("user@Email.tld").await.unwrap();
        assert_eq!(i.id(), 1);
    }
}