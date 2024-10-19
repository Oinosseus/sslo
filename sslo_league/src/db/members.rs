use std::error::Error;
use axum::http::StatusCode;
use sqlx::sqlite::SqlitePool;

#[derive(Clone)]
pub struct Database {
    db_pool: SqlitePool,
}


impl Database {

    pub fn new(db_pool: SqlitePool) -> Self {
        Self {db_pool}
    }


    /// Create a new row into NewEmailUser table.
    /// There is no protection of inserting same email multiple times.
    /// Consumes the Email for the new user and returns the generated, unique, token as String
    pub async fn new_email_user(&self, email: &str) -> Result<String, Box<dyn Error>> {

        // try ten times to generate a unique token
        for i in 0..11 {

            // create token
            let new_token = "12345".to_string();

            // count existence of same token in database
            let new_token_db_count = sqlx::query("SELECT Id FROM NewEmailUser WHERE Token = $1 LIMIT 1;")
                .bind(&new_token)
                .fetch_all(&self.db_pool).await
                .or(Err("Failed to read NewEmailUser table!"))?.len();

            // create new row and return token
            if new_token_db_count == 0 {
                match sqlx::query("INSERT INTO NewEmailUser (Email, Token) VALUES ($1, $2) RETURNING Id;")
                    .bind(&email)
                    .bind(&new_token)
                    .fetch_all(&self.db_pool).await {
                    Err(e) => {
                        log::error!("Failed to request DB.members.NewEmailUser!");
                        log::error!("{}", e);
                        return Err("Failed to insert into NewEmailUser table")?;
                    },
                    Ok(res) => {
                        return Ok(new_token);
                    }
                }
            }
        }

        // failed to create a unique token
        log::error!("Could not generate a unique token for NewEmailUser table");
        Err("Could not generate unique token for NewEmailUser table")?
    }
}


impl super::Database for Database {

    fn pool(&self) -> &SqlitePool {
        &self.db_pool
    }

    async fn init(&mut self) -> Result<(), Box<dyn Error>> {
        sqlx::migrate!("../rsc/db_migrations/league_members").run(&self.db_pool).await?;
        Ok(())
    }

}
