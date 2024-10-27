use std::error::Error;
use rand::RngCore;

pub struct Token {
    pub decrypted: String,
    pub encrypted: String,
}

impl Token {

    pub fn new(decrypted: String, encrypted: String) -> Token {
        Self{ decrypted, encrypted }
    }

    pub fn generate(argon2_config: Option<argon2::Config>) -> Result<Self, Box<dyn Error>> {

        // generate config
        let cfg: argon2::Config = match argon2_config {
            Some(cfg) => cfg,
            None => argon2::Config::default(),
        };

        // create a random token
        let mut token: [u8; 64] = [0; 64];
        rand::thread_rng().fill_bytes(&mut token);
        let decrypted: String = hex::encode(token);

        // create a salt
        let mut salt: Vec<u8> = vec![0u8; 64];
        rand::thread_rng().fill_bytes(&mut salt);

        // encrypt plain token
        let encrypted = argon2::hash_encoded(&token, &salt, &cfg).or_else(|e| {
            log::error!("{}", e);
           Err(e)
        })?;

        Ok(Self { decrypted, encrypted })
    }

    pub fn verify(self) -> bool {
        if self.decrypted.len() == 0 || self.encrypted.len() == 0 { return false; }
        let token: Vec<u8> = hex::decode(self.decrypted).unwrap_or(Vec::new());
        argon2::verify_encoded(&self.encrypted, token.as_slice()).unwrap_or(false)
    }
}


#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;

    #[test]
    fn normal_function() {

        // create a token
        let token = super::Token::generate(None).unwrap();

        // check that plain token is a hex string
        assert_eq!(token.decrypted.len(), 128);

        // verify token
        assert!(token.verify());
    }

    #[test]
    fn manipulation() {
        let mut token = super::Token::generate(None).unwrap();
        token.decrypted += "ab";
        assert!(!token.verify());
    }

    #[test]
    fn invalid() {
        let token = super::Token::new("".to_string(), "".to_string());
        assert!(!token.verify());
    }

    #[test]
    fn time() {

        // strong encryption
        let token = super::Token::generate(None).unwrap();
        let start = std::time::Instant::now();
        assert!(token.verify());
        let duration_strong = start.elapsed();
        let duration_strong = duration_strong.as_millis();

        // weak encryption
        let mut token_cfg = argon2::Config::default();
        token_cfg.mem_cost = 128;
        token_cfg.time_cost = 1;
        // token_cfg.variant = argon2::Variant::Argon2d;
        let token = super::Token::generate(Some(token_cfg)).unwrap();
        let start = std::time::Instant::now();
        assert!(token.verify());
        let duration_weak = start.elapsed();
        let duration_weak = duration_weak.as_millis();

        // evaluate
        println!("Duration(strong={}ms)", duration_strong);
        println!("Duration(weak={}ms)", duration_weak);
        assert!((1000*duration_weak) < duration_strong);
    }
}
