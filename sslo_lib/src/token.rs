use std::error::Error;
use rand::RngCore;

pub struct Token {
    pub plain: String,
    pub crypted: String,
}

impl Token {

    pub fn new(plain: String, crypted: String) -> Token {
        Self{ plain, crypted }
    }

    pub fn generate() -> Result<Self, Box<dyn Error>> {

        // create a random token
        let mut token: [u8; 32] = [0; 32];
        rand::thread_rng().fill_bytes(&mut token);
        let plain: String = hex::encode(token);

        // create a salt
        let mut salt: Vec<u8> = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);

        // encrypt plain token
        let crypted = argon2::hash_encoded(&token, &salt, &argon2::Config::default()).or_else(|e| {
            log::error!("{}", e);
           Err(e)
        })?;

        Ok(Self { plain, crypted })
    }

    pub fn verify(self) -> bool {
        if self.plain.len() == 0 || self.crypted.len() == 0 { return false; }
        let token: Vec<u8> = hex::decode(self.plain).unwrap_or(Vec::new());
        argon2::verify_encoded(&self.crypted, token.as_slice()).unwrap_or(false)
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn normal_function() {

        // create a token
        let token = super::Token::generate().unwrap();

        // check that plain token is a hex string
        assert_eq!(token.plain.len(), 64);

        // verify token
        assert!(token.verify());
    }

    #[test]
    fn manipulation() {
        let mut token = super::Token::generate().unwrap();
        token.plain += "ab";
        assert!(!token.verify());
    }

    #[test]
    fn invalid() {
        let token = super::Token::new("".to_string(), "".to_string());
        assert!(!token.verify());
    }
}
