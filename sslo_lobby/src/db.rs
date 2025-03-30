pub mod members;

use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use sslo_lib::error::SsloError;
use members::{MembersDbData, MembersDbInterface};

struct DatabaseManagerData {
    db_members: Arc<RwLock<MembersDbData>>,
}

impl DatabaseManagerData {
    pub async fn new(database_directory: &Path) -> Result<Arc<RwLock<Self>>, SsloError> {

        // set up tables
        let db_members = MembersDbData::new(Some(database_directory.join("members.db").as_path())).await?;

        // create the manager
        Ok(Arc::new(RwLock::new( Self {
            db_members,
        })))
    }
}

#[derive(Clone)]
pub struct DatabaseManager(Arc<RwLock<DatabaseManagerData>>);

impl DatabaseManager {
    pub async fn new(database_directory: &Path) -> Result<Self, SsloError> {
        let data = DatabaseManagerData::new(database_directory).await?;
        Ok(Self(data))
    }

    pub async fn db_members(&self) -> MembersDbInterface {
        MembersDbInterface::new(self.0.read().await.db_members.clone())
    }
}