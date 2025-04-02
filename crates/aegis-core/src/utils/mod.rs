//! Utility functions and helpers for the Aegis framework
//!
//! This module contains common helpers and utility functions that are used
//! throughout the framework for common tasks like string manipulation,
//! serialization, cryptography and more.

use crate::error::{AegisError, AegisResult};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Generate a random UUID v4
pub fn generate_uuid() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

/// Get the current timestamp as an ISO8601 formatted string
pub fn current_timestamp() -> String {
    use chrono::Utc;
    Utc::now().to_rfc3339()
}

/// Get the current timestamp as milliseconds since the Unix epoch
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis() as u64
}

/// Serialize a struct to JSON string
pub fn to_json<T: serde::Serialize>(value: &T) -> AegisResult<String> {
    serde_json::to_string(value).map_err(|e| AegisError::Serialization(e.to_string()))
}

/// Deserialize a JSON string to a struct
pub fn from_json<T: serde::de::DeserializeOwned>(json: &str) -> AegisResult<T> {
    serde_json::from_str(json).map_err(|e| AegisError::Serialization(e.to_string()))
}

/// Serialize a struct to YAML string
pub fn to_yaml<T: serde::Serialize>(value: &T) -> AegisResult<String> {
    serde_yaml::to_string(value).map_err(|e| AegisError::Serialization(e.to_string()))
}

/// Deserialize a YAML string to a struct
pub fn from_yaml<T: serde::de::DeserializeOwned>(yaml: &str) -> AegisResult<T> {
    serde_yaml::from_str(yaml).map_err(|e| AegisError::Serialization(e.to_string()))
}

/// Calculate SHA-256 hash of a byte slice
pub fn sha256_hash(data: &[u8]) -> Vec<u8> {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Format a byte slice as a hexadecimal string
pub fn format_hex(bytes: &[u8]) -> String {
    use base16ct::{lower::encode_string};
    encode_string(bytes)
}

/// Parse a hexadecimal string to a byte vector
pub fn parse_hex(hex: &str) -> AegisResult<Vec<u8>> {
    use base16ct::{mixed::decode_vec};
    decode_vec(hex).map_err(|e| AegisError::Serialization(format!("Invalid hex string: {}", e)))
}

/// Encrypt data using AES-GCM
pub fn encrypt_aes_gcm(data: &[u8], key: &[u8], nonce: &[u8]) -> AegisResult<Vec<u8>> {
    use aes_gcm::{
        aead::{Aead, KeyInit, Payload},
        Aes256Gcm, Nonce,
    };

    if key.len() != 32 {
        return Err(AegisError::Security("Invalid key length for AES-256-GCM".into()));
    }

    if nonce.len() != 12 {
        return Err(AegisError::Security("Invalid nonce length for AES-256-GCM".into()));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AegisError::Security(format!("Failed to initialize cipher: {}", e)))?;

    let nonce = Nonce::from_slice(nonce);
    let payload = Payload {
        msg: data,
        aad: b"",
    };

    cipher
        .encrypt(nonce, payload)
        .map_err(|e| AegisError::Security(format!("Encryption failed: {}", e)))
}

/// Decrypt data using AES-GCM
pub fn decrypt_aes_gcm(ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> AegisResult<Vec<u8>> {
    use aes_gcm::{
        aead::{Aead, KeyInit, Payload},
        Aes256Gcm, Nonce,
    };

    if key.len() != 32 {
        return Err(AegisError::Security("Invalid key length for AES-256-GCM".into()));
    }

    if nonce.len() != 12 {
        return Err(AegisError::Security("Invalid nonce length for AES-256-GCM".into()));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AegisError::Security(format!("Failed to initialize cipher: {}", e)))?;

    let nonce = Nonce::from_slice(nonce);
    let payload = Payload {
        msg: ciphertext,
        aad: b"",
    };

    cipher
        .decrypt(nonce, payload)
        .map_err(|e| AegisError::Security(format!("Decryption failed: {}", e)))
}

/// Create a cryptographically secure random byte array
pub fn random_bytes(length: usize) -> Vec<u8> {
    use rand::{RngCore, rngs::OsRng};
    
    let mut bytes = vec![0u8; length];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

/// Calculate file extension from path
pub fn file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

/// Get the MIME type from a file extension
pub fn mime_type_from_extension(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "yaml" | "yml" => "application/yaml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        _ => "application/octet-stream",
    }
}

/// Trim whitespace and trailing slash from a URL or path
pub fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.ends_with('/') || trimmed.ends_with('\\') {
        trimmed[0..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

/// Join URL or file path segments safely
pub fn join_paths(base: &str, path: &str) -> String {
    let base = normalize_path(base);
    let path = path.trim_start_matches('/').trim_start_matches('\\');
    
    if base.is_empty() {
        return path.to_string();
    }
    
    format!("{}/{}", base, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_uuid() {
        let uuid = generate_uuid();
        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.chars().filter(|&c| c == '-').count(), 4);
    }

    #[test]
    fn test_current_timestamp() {
        let timestamp = current_timestamp();
        assert!(timestamp.len() > 20);
        assert!(timestamp.contains('T'));
        assert!(timestamp.contains('Z') || timestamp.contains('+'));
    }

    #[test]
    fn test_json_serialization() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Test {
            name: String,
            value: i32,
        }

        let test = Test {
            name: "test".to_string(),
            value: 42,
        };

        let json = to_json(&test).unwrap();
        let parsed: Test = from_json(&json).unwrap();

        assert_eq!(test, parsed);
    }

    #[test]
    fn test_sha256_and_hex() {
        let data = b"test data";
        let hash = sha256_hash(data);
        let hex = format_hex(&hash);
        
        assert_eq!(hash.len(), 32); // SHA-256 is 32 bytes
        assert_eq!(hex.len(), 64);  // Hex string is 64 chars
        
        let parsed = parse_hex(&hex).unwrap();
        assert_eq!(hash, parsed);
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("/test/path/"), "/test/path");
        assert_eq!(normalize_path("test/path/"), "test/path");
        assert_eq!(normalize_path("test/path"), "test/path");
        assert_eq!(normalize_path("  /test/  "), "/test");
    }

    #[test]
    fn test_join_paths() {
        assert_eq!(join_paths("/base/path", "/sub/path"), "/base/path/sub/path");
        assert_eq!(join_paths("/base/path/", "sub/path"), "/base/path/sub/path");
        assert_eq!(join_paths("", "/sub/path"), "sub/path");
        assert_eq!(join_paths("/base/path", ""), "/base/path");
    }
} 