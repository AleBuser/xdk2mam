use std::sync::Mutex;

use crate::security::keystore::calculate_hash;
use crate::security::keystore::KeyManager;

pub struct ApiKeyAuthor(String);

/// Returns true if `key` is a valid API key string.
fn is_valid(key: &str, hash: String) -> bool {
    calculate_hash(key.to_string()) == hash
}

#[derive(Debug)]
pub enum ApiKeyError {
    BadCount,
    Missing,
    Invalid,
}
