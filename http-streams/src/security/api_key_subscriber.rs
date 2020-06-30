use std::sync::Mutex;

extern crate serde_json;

use crate::security::keystore::calculate_hash;
use crate::security::keystore::KeyManager;

pub struct ApiKeySubscriber(String);

/// Returns true if `key` is a valid API key string.
fn is_valid(key: &str, hashes: Vec<String>) -> bool {
    for hash in hashes {
        if calculate_hash(key.to_string()) == hash {
            return true;
        }
    }
    false
}

#[derive(Debug)]
pub enum ApiKeyError {
    BadCount,
    Missing,
    Invalid,
}
