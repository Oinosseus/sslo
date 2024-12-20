use std::sync::{Arc, Weak};
use sslo_lib::db::PoolPassing;

pub struct Item {
    pool_ref_2parent: Weak<dyn PoolPassing>,
    row: super::row::Row,
}

impl Item {

    pub fn id(&self) -> i64 { self.row.rowid}

    /// Set up an object from a data row (assumed to be clean retrieved from db)
    pub(super) fn from_row(pool_ref: Weak<dyn PoolPassing>, row: super::row::Row) -> Arc<Self> {
        Arc::new(Self { pool_ref_2parent: pool_ref, row } )
    }

}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use sqlx::SqlitePool;
    use super::*;

    struct ItemTestTable {
        pub pool: SqlitePool,
        pub pool_ref_2me: Weak<dyn PoolPassing>,
    }
    impl ItemTestTable {
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
            sqlx::query(concat!("INSERT INTO ", super::super::tablename!(), " (name, email) VALUES ($1, $2);"))
                .bind("username")
                .bind("user@email.tld")
                .execute(&self.pool)
                .await.unwrap();
        }
    }
    impl PoolPassing for ItemTestTable {
        fn pool(&self) -> Option<SqlitePool> {Some(self.pool.clone())}
    }

    #[tokio::test]
    async fn test_item() {

        // create test table
        let tbl = ItemTestTable::new();
        tbl.init().await;

        // test failed retrieval
        // let i = Item::from_db_by_id(tbl.pool_ref_2me.clone(), 999).await;
        // assert!(i.is_none());

        // test retrieval
        // let i = Item::from_db_by_id(tbl.pool_ref_2me.clone(), 1).await.unwrap();
        // assert_eq!(i.id(), 1);
    }
}