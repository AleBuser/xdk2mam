use crypto::digest::Digest;
use crypto::sha1::Sha1;
use serde::{Deserialize, Serialize};

use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
pub struct Keystore {
    pub api_key_subscribers: Vec<String>,
    pub api_key_author: String,
}

#[derive(Debug)]
pub struct KeyManager {
    pub keystore: Keystore,
}

impl KeyManager {
    pub fn new(new_key_aut: String, new_key_subscriber: Vec<String>) -> KeyManager {
        KeyManager {
            keystore: Keystore {
                api_key_subscribers: new_key_subscriber,
                api_key_author: new_key_aut,
            },
        }
    }

    pub fn restore() -> KeyManager {
        let rec: Keystore = serde_json::from_reader(File::open("keystore.json").unwrap()).unwrap();
        KeyManager { keystore: rec }
    }

    pub fn add_subscriber(&mut self, new_key_subscriber: String) -> () {
        self.keystore
            .api_key_subscribers
            .push(calculate_hash(new_key_subscriber));
        store_keystore(&self.keystore)
    }
}

pub fn store_keystore(k: &Keystore) -> () {
    serde_json::to_writer(&File::create("keystore.json").unwrap(), k).unwrap();
}

pub fn calculate_hash(t: String) -> String {
    let mut hasher = Sha1::new();
    hasher.input_str(&t);
    let hex = hasher.result_str();
    hex
}
