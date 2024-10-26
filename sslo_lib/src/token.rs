use std::error::Error;
use rand::RngCore;

pub struct Token {
    pub decrypted: String,
    pub encrypted: String,
}

impl Token {

    pub fn new(plain: String, crypted: String) -> Token {
        Self{ decrypted: plain, encrypted: crypted }
    }

    pub fn generate() -> Result<Self, Box<dyn Error>> {

        // create a random token
        let mut token: [u8; 64] = [0; 64];
        rand::thread_rng().fill_bytes(&mut token);
        let decrypted: String = hex::encode(token);

        // create a salt
        let mut salt: Vec<u8> = vec![0u8; 64];
        rand::thread_rng().fill_bytes(&mut salt);

        // encrypt plain token
        let encrypted = argon2::hash_encoded(&token, &salt, &argon2::Config::default()).or_else(|e| {
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
    #[test]
    fn normal_function() {

        // create a token
        let token = super::Token::generate().unwrap();

        // check that plain token is a hex string
        assert_eq!(token.decrypted.len(), 128);

        // verify token
        assert!(token.verify());
    }

    #[test]
    fn manipulation() {
        let mut token = super::Token::generate().unwrap();
        token.decrypted += "ab";
        assert!(!token.verify());
    }

    #[test]
    fn invalid() {
        let token = super::Token::new("".to_string(), "".to_string());
        assert!(!token.verify());
    }
}
