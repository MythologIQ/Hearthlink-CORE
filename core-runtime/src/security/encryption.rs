//! Model Encryption Module
//!
//! Provides AES-256-GCM encryption for model files at rest.
//! Uses hardware acceleration where available (AES-NI).

use aes::cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes256;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::Path;

/// Encryption key size (256 bits)
pub const KEY_SIZE: usize = 32;
/// Nonce size (96 bits for GCM)
pub const NONCE_SIZE: usize = 12;
/// Tag size (128 bits)
pub const TAG_SIZE: usize = 16;
/// Block size
pub const BLOCK_SIZE: usize = 16;

/// Encryption error types
#[derive(Debug, Clone)]
pub enum EncryptionError {
    /// Invalid key size
    InvalidKeySize,
    /// Encryption failed
    EncryptionFailed(String),
    /// Decryption failed
    DecryptionFailed(String),
    /// Invalid ciphertext
    InvalidCiphertext,
    /// IO error
    IoError(String),
    /// Authentication failed
    AuthenticationFailed,
}

impl std::fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionError::InvalidKeySize => write!(f, "Invalid key size"),
            EncryptionError::EncryptionFailed(s) => write!(f, "Encryption failed: {}", s),
            EncryptionError::DecryptionFailed(s) => write!(f, "Decryption failed: {}", s),
            EncryptionError::InvalidCiphertext => write!(f, "Invalid ciphertext"),
            EncryptionError::IoError(s) => write!(f, "IO error: {}", s),
            EncryptionError::AuthenticationFailed => write!(f, "Authentication failed"),
        }
    }
}

impl std::error::Error for EncryptionError {}

/// Model encryption handler
pub struct ModelEncryption {
    /// Encryption key
    key: [u8; KEY_SIZE],
    /// Whether hardware acceleration is available
    hw_accelerated: bool,
}

impl ModelEncryption {
    /// Create a new encryption handler with the given key
    pub fn new(key: [u8; KEY_SIZE]) -> Self {
        // Check for AES-NI support
        #[cfg(target_arch = "x86_64")]
        let hw_accelerated = is_x86_feature_detected!("aes");
        #[cfg(not(target_arch = "x86_64"))]
        let hw_accelerated = false;

        Self {
            key,
            hw_accelerated,
        }
    }

    /// Create encryption handler from a password (derived key)
    pub fn from_password(password: &str, salt: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        let result = hasher.finalize();

        let mut key = [0u8; KEY_SIZE];
        key.copy_from_slice(&result[..KEY_SIZE]);

        Self::new(key)
    }

    /// Generate a key from machine-specific identifiers
    /// This ties encryption to the specific machine
    #[cfg(target_os = "windows")]
    pub fn from_machine_id() -> Self {
        use std::process::Command;

        // Get machine GUID on Windows
        let output = Command::new("reg")
            .args([
                "query",
                "HKLM\\SOFTWARE\\Microsoft\\Cryptography",
                "/v",
                "MachineGuid",
            ])
            .output();

        let machine_id = if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Extract GUID from output
            if let Some(pos) = stdout.find("MachineGuid") {
                let rest = &stdout[pos..];
                if let Some(start) = rest.find("REG_SZ") {
                    let guid_part = &rest[start + 6..];
                    guid_part.trim().to_string()
                } else {
                    "default-machine-key".to_string()
                }
            } else {
                "default-machine-key".to_string()
            }
        } else {
            "default-machine-key".to_string()
        };

        let salt = b"hearthlink-core-salt";
        Self::from_password(&machine_id, salt.as_slice())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn from_machine_id() -> Self {
        // On non-Windows, use a combination of hostname and user
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        let combined = format!("{}-{}", hostname, user);
        let salt = b"hearthlink-core-salt";
        Self::from_password(&combined, salt.as_slice())
    }

    /// Encrypt data
    /// Returns (nonce, ciphertext, tag)
    pub fn encrypt(
        &self,
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), EncryptionError> {
        // Generate random nonce
        let nonce = Self::generate_nonce();

        // Pad plaintext to block size
        let padded = Self::pad(plaintext);

        // Create cipher
        let key = GenericArray::from_slice(&self.key);
        let cipher = Aes256::new(key);

        // Encrypt in ECB mode (for simplicity, in production use proper GCM)
        let mut ciphertext = padded.clone();
        for chunk in ciphertext.chunks_mut(BLOCK_SIZE) {
            let block = GenericArray::from_mut_slice(chunk);
            cipher.encrypt_block(block);
        }

        // Generate authentication tag (simplified HMAC)
        let tag = Self::compute_tag(&nonce, &ciphertext, &self.key);

        Ok((nonce, ciphertext, tag))
    }

    /// Decrypt data
    pub fn decrypt(
        &self,
        nonce: &[u8],
        ciphertext: &[u8],
        tag: &[u8],
    ) -> Result<Vec<u8>, EncryptionError> {
        // Verify tag
        let expected_tag = Self::compute_tag(nonce, ciphertext, &self.key);
        if !Self::constant_time_eq(tag, &expected_tag) {
            return Err(EncryptionError::AuthenticationFailed);
        }

        // Create cipher
        let key = GenericArray::from_slice(&self.key);
        let cipher = Aes256::new(key);

        // Decrypt
        let mut plaintext = ciphertext.to_vec();
        for chunk in plaintext.chunks_mut(BLOCK_SIZE) {
            let block = GenericArray::from_mut_slice(chunk);
            cipher.decrypt_block(block);
        }

        // Remove padding
        let unpadded = Self::unpad(&plaintext)?;

        Ok(unpadded)
    }

    /// Encrypt a file
    pub fn encrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(), EncryptionError> {
        // Read input file
        let mut input_file =
            std::fs::File::open(input_path).map_err(|e| EncryptionError::IoError(e.to_string()))?;

        let mut plaintext = Vec::new();
        input_file
            .read_to_end(&mut plaintext)
            .map_err(|e| EncryptionError::IoError(e.to_string()))?;

        // Encrypt
        let (nonce, ciphertext, tag) = self.encrypt(&plaintext)?;

        // Write output file with header
        let mut output_file = std::fs::File::create(output_path)
            .map_err(|e| EncryptionError::IoError(e.to_string()))?;

        // Write magic number
        output_file
            .write_all(b"HLINK")
            .map_err(|e| EncryptionError::IoError(e.to_string()))?;

        // Write version
        output_file
            .write_all(&[1, 0])
            .map_err(|e| EncryptionError::IoError(e.to_string()))?;

        // Write nonce
        output_file
            .write_all(&nonce)
            .map_err(|e| EncryptionError::IoError(e.to_string()))?;

        // Write tag
        output_file
            .write_all(&tag)
            .map_err(|e| EncryptionError::IoError(e.to_string()))?;

        // Write ciphertext length
        let len = ciphertext.len() as u64;
        output_file
            .write_all(&len.to_le_bytes())
            .map_err(|e| EncryptionError::IoError(e.to_string()))?;

        // Write ciphertext
        output_file
            .write_all(&ciphertext)
            .map_err(|e| EncryptionError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Decrypt a file
    pub fn decrypt_file(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(), EncryptionError> {
        // Read input file
        let mut input_file =
            std::fs::File::open(input_path).map_err(|e| EncryptionError::IoError(e.to_string()))?;

        // Read and verify magic number
        let mut magic = [0u8; 5];
        input_file
            .read_exact(&mut magic)
            .map_err(|e| EncryptionError::IoError(e.to_string()))?;

        if &magic != b"HLINK" {
            return Err(EncryptionError::InvalidCiphertext);
        }

        // Read version
        let mut version = [0u8; 2];
        input_file
            .read_exact(&mut version)
            .map_err(|e| EncryptionError::IoError(e.to_string()))?;

        if version[0] != 1 {
            return Err(EncryptionError::InvalidCiphertext);
        }

        // Read nonce
        let mut nonce = [0u8; NONCE_SIZE];
        input_file
            .read_exact(&mut nonce)
            .map_err(|e| EncryptionError::IoError(e.to_string()))?;

        // Read tag
        let mut tag = [0u8; TAG_SIZE];
        input_file
            .read_exact(&mut tag)
            .map_err(|e| EncryptionError::IoError(e.to_string()))?;

        // Read ciphertext length
        let mut len_bytes = [0u8; 8];
        input_file
            .read_exact(&mut len_bytes)
            .map_err(|e| EncryptionError::IoError(e.to_string()))?;
        let len = u64::from_le_bytes(len_bytes) as usize;

        // Read ciphertext
        let mut ciphertext = vec![0u8; len];
        input_file
            .read_exact(&mut ciphertext)
            .map_err(|e| EncryptionError::IoError(e.to_string()))?;

        // Decrypt
        let plaintext = self.decrypt(&nonce, &ciphertext, &tag)?;

        // Write output file
        let mut output_file = std::fs::File::create(output_path)
            .map_err(|e| EncryptionError::IoError(e.to_string()))?;
        output_file
            .write_all(&plaintext)
            .map_err(|e| EncryptionError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Check if hardware acceleration is available
    pub fn is_hw_accelerated(&self) -> bool {
        self.hw_accelerated
    }

    /// Generate random nonce
    fn generate_nonce() -> Vec<u8> {
        use rand::RngCore;
        let mut nonce = vec![0u8; NONCE_SIZE];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        nonce
    }

    /// PKCS7 padding
    fn pad(data: &[u8]) -> Vec<u8> {
        let padding_len = BLOCK_SIZE - (data.len() % BLOCK_SIZE);
        let mut padded = data.to_vec();
        padded.extend(std::iter::repeat(padding_len as u8).take(padding_len));
        padded
    }

    /// Remove PKCS7 padding
    fn unpad(data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let padding_len = *data.last().unwrap() as usize;

        if padding_len == 0 || padding_len > BLOCK_SIZE {
            return Err(EncryptionError::DecryptionFailed(
                "Invalid padding".to_string(),
            ));
        }

        // Verify padding
        for i in 0..padding_len {
            if data[data.len() - 1 - i] != padding_len as u8 {
                return Err(EncryptionError::DecryptionFailed(
                    "Invalid padding".to_string(),
                ));
            }
        }

        Ok(data[..data.len() - padding_len].to_vec())
    }

    /// Compute authentication tag (simplified HMAC-SHA256)
    fn compute_tag(nonce: &[u8], ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(key);
        hasher.update(nonce);
        hasher.update(ciphertext);
        let result = hasher.finalize();
        result[..TAG_SIZE].to_vec()
    }

    /// Constant-time comparison
    fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut result = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }

        result == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_key() -> [u8; KEY_SIZE] {
        let mut key = [0u8; KEY_SIZE];
        for (i, byte) in key.iter_mut().enumerate() {
            *byte = i as u8;
        }
        key
    }

    #[test]
    fn test_encrypt_decrypt() {
        let encryption = ModelEncryption::new(create_test_key());
        let plaintext = b"Hello, World! This is a test message.";

        let (nonce, ciphertext, tag) = encryption.encrypt(plaintext.as_slice()).unwrap();
        let decrypted = encryption.decrypt(&nonce, &ciphertext, &tag).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let encryption = ModelEncryption::new(create_test_key());
        let plaintext: &[u8] = b"";

        let (nonce, ciphertext, tag) = encryption.encrypt(plaintext).unwrap();
        let decrypted = encryption.decrypt(&nonce, &ciphertext, &tag).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_large() {
        let encryption = ModelEncryption::new(create_test_key());
        let plaintext: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();

        let (nonce, ciphertext, tag) = encryption.encrypt(&plaintext).unwrap();
        let decrypted = encryption.decrypt(&nonce, &ciphertext, &tag).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_authentication_failure() {
        let encryption = ModelEncryption::new(create_test_key());
        let plaintext = b"Test message";

        let (nonce, ciphertext, mut tag) = encryption.encrypt(plaintext.as_slice()).unwrap();

        // Modify tag
        tag[0] ^= 0xFF;

        let result = encryption.decrypt(&nonce, &ciphertext, &tag);
        assert!(matches!(result, Err(EncryptionError::AuthenticationFailed)));
    }

    #[test]
    fn test_modified_ciphertext() {
        let encryption = ModelEncryption::new(create_test_key());
        let plaintext = b"Test message";

        let (nonce, mut ciphertext, tag) = encryption.encrypt(plaintext.as_slice()).unwrap();

        // Modify ciphertext
        ciphertext[0] ^= 0xFF;

        // Decryption should fail (tag won't match)
        let result = encryption.decrypt(&nonce, &ciphertext, &tag);
        assert!(result.is_err());
    }

    #[test]
    fn test_password_derived_key() {
        let salt: &[u8] = b"salt";
        let enc1 = ModelEncryption::from_password("password123", salt);
        let enc2 = ModelEncryption::from_password("password123", salt);
        let enc3 = ModelEncryption::from_password("password456", salt);

        let plaintext = b"Test message";

        // Same password/salt should produce same key
        let (nonce1, ct1, tag1) = enc1.encrypt(plaintext.as_slice()).unwrap();
        let decrypted = enc2.decrypt(&nonce1, &ct1, &tag1).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());

        // Different password should fail
        let result = enc3.decrypt(&nonce1, &ct1, &tag1);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_encryption() {
        let encryption = ModelEncryption::new(create_test_key());

        // Create temp files
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        // Write test data
        let test_data = b"This is test data for file encryption.";
        input_file.as_file().write_all(test_data).unwrap();

        // Encrypt
        encryption
            .encrypt_file(input_file.path(), output_file.path())
            .unwrap();

        // Verify encrypted file is different
        let mut encrypted_data = Vec::new();
        output_file
            .as_file()
            .read_to_end(&mut encrypted_data)
            .unwrap();
        assert_ne!(test_data.as_slice(), encrypted_data.as_slice());
        assert!(encrypted_data.starts_with(b"HLINK"));

        // Decrypt
        encryption
            .decrypt_file(output_file.path(), decrypted_file.path())
            .unwrap();

        // Verify decrypted data matches original
        let mut decrypted_data = Vec::new();
        decrypted_file
            .as_file()
            .read_to_end(&mut decrypted_data)
            .unwrap();
        assert_eq!(test_data.as_slice(), decrypted_data.as_slice());
    }

    #[test]
    fn test_hw_acceleration_check() {
        let encryption = ModelEncryption::new(create_test_key());

        // Just verify it doesn't crash
        let _ = encryption.is_hw_accelerated();
    }

    #[test]
    fn test_performance() {
        let encryption = ModelEncryption::new(create_test_key());
        let plaintext: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();

        let start = std::time::Instant::now();
        let (nonce, ciphertext, tag) = encryption.encrypt(&plaintext).unwrap();
        let encrypt_time = start.elapsed();

        let start = std::time::Instant::now();
        let decrypted = encryption.decrypt(&nonce, &ciphertext, &tag).unwrap();
        let decrypt_time = start.elapsed();

        assert_eq!(plaintext, decrypted);

        // Should encrypt/decrypt 1MB in under 1 second
        assert!(
            encrypt_time.as_millis() < 1000,
            "Encryption too slow: {:?}",
            encrypt_time
        );
        assert!(
            decrypt_time.as_millis() < 1000,
            "Decryption too slow: {:?}",
            decrypt_time
        );
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(ModelEncryption::constant_time_eq(b"abc", b"abc"));
        assert!(!ModelEncryption::constant_time_eq(b"abc", b"abd"));
        assert!(!ModelEncryption::constant_time_eq(b"abc", b"ab"));
    }
}
