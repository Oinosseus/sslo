use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use sslo_lib::db::DatabaseError;
use crate::db2::members::{MembersDbData, MembersDbInterface};

mod members;

struct DatabaseManagerData {
    db_members: Arc<RwLock<MembersDbData>>,
}

impl DatabaseManagerData {
    pub async fn new(database_directory: &Path) -> Result<Arc<RwLock<Self>>, DatabaseError> {

        // set up tables
        let db_members = MembersDbData::new(Some(database_directory.join("members.db").as_path())).await?;

        // create the manager
        Ok(Arc::new(RwLock::new( Self {
           db_members,
        })))
    }
}

struct DatabaseManager(Arc<RwLock<DatabaseManagerData>>);

impl DatabaseManager {
    pub async fn new(database_directory: &Path) -> Result<Self, DatabaseError> {
        let data = DatabaseManagerData::new(database_directory).await?;
        Ok(Self(data))
    }

    // pub async fn clone(&self) -> Self {
    //     Self(self.0.clone())
    // }

    pub async fn db_members(&self) -> MembersDbInterface {
        MembersDbInterface::new(self.0.read().await.db_members.clone())
    }
}