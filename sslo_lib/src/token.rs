use std::error::Error;
use rand::RngCore;


pub enum TokenType {

    /// eg. for passwords
    Strong,

    /// eg. for random generated, temporary tokens
    Quick,
}


impl TokenType {

    pub fn get_config(&self) -> argon2::Config {
        match self {
            Self::Strong => {
                argon2::Config::default()
            },
            Self::Quick => {
                let mut cfg = argon2::Config::default();
                cfg.mem_cost = 128;
                cfg.time_cost = 1;
                cfg.variant = argon2::Variant::Argon2d;
                cfg
            }
        }
    }
}


pub struct Token {
    pub decrypted: String,
    pub encrypted: String,
}


impl Token {

    pub fn new(decrypted: String, encrypted: String) -> Self {
        Self{ decrypted, encrypted }
    }

    pub fn generate(token_type: TokenType) -> Result<Self, Box<dyn Error>> {

        // create a random token
        let mut token: [u8; 64] = [0; 64];
        rand::thread_rng().fill_bytes(&mut token);
        let decrypted: String = hex::encode(token);

        // create a salt
        let mut salt: Vec<u8> = vec![0u8; 64];
        rand::thread_rng().fill_bytes(&mut salt);

        // encrypt plain token
        let encrypted = argon2::hash_encoded(&token, &salt, &token_type.get_config()).or_else(|e| {
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
    use super::*;

    #[test]
    fn normal_function() {

        // create a token
        let token = Token::generate(TokenType::Strong).unwrap();

        // check that plain token is a hex string
        assert_eq!(token.decrypted.len(), 128);

        // verify token
        assert!(token.verify());
    }

    #[test]
    fn manipulation() {
        let mut token = Token::generate(TokenType::Strong).unwrap();
        token.decrypted += "ab";
        assert!(!token.verify());
    }

    #[test]
    fn invalid() {
        let token = Token::new("".to_string(), "".to_string());
        assert!(!token.verify());
    }

    #[test]
    fn time() {

        // strong encryption
        let token = Token::generate(TokenType::Strong).unwrap();
        let start = std::time::Instant::now();
        assert!(token.verify());
        let duration_strong = start.elapsed();
        let duration_strong = duration_strong.as_millis();

        // weak encryption
        let token = Token::generate(TokenType::Quick).unwrap();
        let start = std::time::Instant::now();
        assert!(token.verify());
        let duration_weak = start.elapsed();
        let duration_weak = duration_weak.as_millis();

        // evaluate
        println!("Duration(strong={}ms)", duration_strong);
        println!("Duration(weak={}ms)", duration_weak);
        assert!((10*duration_weak) < duration_strong);  // the quick encryption should be much faster
    }
}
