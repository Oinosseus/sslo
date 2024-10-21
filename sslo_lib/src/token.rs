use std::error::Error;
use rand::RngCore;

pub struct Token {
    pub plain: String,
    pub crypted: String,
}

impl Token {
    pub fn new() -> Result<Self, Box<dyn Error>> {

        // create a random token
        let mut token: [u8; 32] = [0; 32];
        rand::thread_rng().fill_bytes(&mut token);
        let plain = format!("{:x?}", &token);

        // create a salt
        let mut salt: Vec<u8> = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);

        // encrypt plain token
        let crypted = argon2::hash_encoded(&token, &salt, &argon2::Config::default())?;

        Ok(Self { plain, crypted })
    }
}

