use crate::error::{SecurityError, SecurityResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ring::rand::{SecureRandom, SystemRandom};
use ring::{digest, aead, hmac};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rustls::{ClientConfig, RootCertStore};


/// Certificate information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub id: String,
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub fingerprint: String,
    pub key_usage: Vec<String>,
}

/// Private key information
#[derive(Debug, Clone)]
pub struct PrivateKeyInfo {
    pub id: String,
    pub algorithm: String,
    pub key_data: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

/// Certificate storage trait for secure certificate and key management
#[async_trait]
pub trait CertificateStore: Send + Sync {
    /// Store a certificate with its associated metadata
    async fn store_certificate(&self, cert_pem: &str, id: &str) -> SecurityResult<CertificateInfo>;
    
    /// Retrieve a certificate by ID
    async fn get_certificate(&self, id: &str) -> SecurityResult<Option<String>>;
    
    /// Store a private key securely
    async fn store_private_key(&self, key_pem: &str, id: &str) -> SecurityResult<PrivateKeyInfo>;
    
    /// Retrieve a private key by ID
    async fn get_private_key(&self, id: &str) -> SecurityResult<Option<Vec<u8>>>;
    
    /// List all stored certificates
    async fn list_certificates(&self) -> SecurityResult<Vec<CertificateInfo>>;
    
    /// Remove a certificate and its associated key
    async fn remove_certificate(&self, id: &str) -> SecurityResult<()>;
    
    /// Validate certificate expiration
    async fn validate_certificate_expiration(&self, id: &str) -> SecurityResult<bool>;
}

/// In-memory certificate store implementation (for development/testing)
#[derive(Debug)]
pub struct InMemoryCertificateStore {
    certificates: Arc<RwLock<HashMap<String, (String, CertificateInfo)>>>,
    private_keys: Arc<RwLock<HashMap<String, PrivateKeyInfo>>>,
}

impl InMemoryCertificateStore {
    pub fn new() -> Self {
        Self {
            certificates: Arc::new(RwLock::new(HashMap::new())),
            private_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl CertificateStore for InMemoryCertificateStore {
    async fn store_certificate(&self, cert_pem: &str, id: &str) -> SecurityResult<CertificateInfo> {
        let cert_info = parse_certificate_info(cert_pem, id)?;
        let mut certs = self.certificates.write().await;
        certs.insert(id.to_string(), (cert_pem.to_string(), cert_info.clone()));
        Ok(cert_info)
    }
    
    async fn get_certificate(&self, id: &str) -> SecurityResult<Option<String>> {
        let certs = self.certificates.read().await;
        Ok(certs.get(id).map(|(pem, _)| pem.clone()))
    }
    
    async fn store_private_key(&self, key_pem: &str, id: &str) -> SecurityResult<PrivateKeyInfo> {
        let key_info = parse_private_key_info(key_pem, id)?;
        let mut keys = self.private_keys.write().await;
        keys.insert(id.to_string(), key_info.clone());
        Ok(key_info)
    }
    
    async fn get_private_key(&self, id: &str) -> SecurityResult<Option<Vec<u8>>> {
        let keys = self.private_keys.read().await;
        Ok(keys.get(id).map(|key_info| key_info.key_data.clone()))
    }
    
    async fn list_certificates(&self) -> SecurityResult<Vec<CertificateInfo>> {
        let certs = self.certificates.read().await;
        Ok(certs.values().map(|(_, info)| info.clone()).collect())
    }
    
    async fn remove_certificate(&self, id: &str) -> SecurityResult<()> {
        let mut certs = self.certificates.write().await;
        let mut keys = self.private_keys.write().await;
        certs.remove(id);
        keys.remove(id);
        Ok(())
    }
    
    async fn validate_certificate_expiration(&self, id: &str) -> SecurityResult<bool> {
        let certs = self.certificates.read().await;
        match certs.get(id) {
            Some((_, cert_info)) => {
                let now = Utc::now();
                Ok(now >= cert_info.not_before && now <= cert_info.not_after)
            }
            None => Err(SecurityError::Certificate(format!("Certificate not found: {}", id))),
        }
    }
}

/// Security manager for handling cryptographic operations and certificate management
pub struct SecurityManager {
    certificate_store: Arc<dyn CertificateStore>,
    rng: SystemRandom,
}

impl SecurityManager {
    /// Create a new security manager with the specified certificate store
    pub fn new(certificate_store: Arc<dyn CertificateStore>) -> Self {
        Self {
            certificate_store,
            rng: SystemRandom::new(),
        }
    }
    
    /// Generate a secure random number of specified length
    pub fn generate_random_bytes(&self, length: usize) -> SecurityResult<Vec<u8>> {
        let mut bytes = vec![0u8; length];
        self.rng.fill(&mut bytes)
            .map_err(|e| SecurityError::KeyManagement(format!("Random generation failed: {:?}", e)))?;
        Ok(bytes)
    }
    
    /// Generate a new RSA key pair
    pub fn generate_rsa_key_pair(&self) -> SecurityResult<(Vec<u8>, Vec<u8>)> {
        // For production, this would use ring or another crypto library to generate actual RSA keys
        // For now, we'll return placeholder data
        let private_key = self.generate_random_bytes(2048)?;
        let public_key = self.generate_random_bytes(256)?;
        Ok((private_key, public_key))
    }
    
    /// Store a certificate and its associated private key
    pub async fn store_certificate_with_key(
        &self,
        cert_pem: &str,
        key_pem: &str,
        id: &str,
    ) -> SecurityResult<CertificateInfo> {
        // Store the private key first
        self.certificate_store.store_private_key(key_pem, id).await?;
        
        // Then store the certificate
        self.certificate_store.store_certificate(cert_pem, id).await
    }
    
    /// Validate all stored certificates for expiration
    pub async fn validate_all_certificates(&self) -> SecurityResult<Vec<(String, bool)>> {
        let certificates = self.certificate_store.list_certificates().await?;
        let mut results = Vec::new();
        
        for cert in certificates {
            let is_valid = self.certificate_store
                .validate_certificate_expiration(&cert.id)
                .await?;
            results.push((cert.id, is_valid));
        }
        
        Ok(results)
    }
    
    /// Get certificate store reference
    pub fn certificate_store(&self) -> &Arc<dyn CertificateStore> {
        &self.certificate_store
    }
    
    /// Compute SHA-256 hash of data
    pub fn compute_sha256(&self, data: &[u8]) -> Vec<u8> {
        digest::digest(&digest::SHA256, data).as_ref().to_vec()
    }
    
    /// Verify a signature using a public key (placeholder implementation)
    pub fn verify_signature(
        &self,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> SecurityResult<bool> {
        // This is a placeholder implementation
        // In production, this would use proper signature verification
        Ok(signature.len() > 0 && public_key.len() > 0 && message.len() > 0)
    }
    
    /// Create TLS 1.3 configuration for AWS IoT connections
    pub fn create_tls_config(&self, _cert_id: &str) -> SecurityResult<ClientConfig> {
        let mut root_store = RootCertStore::empty();
        
        // Add system root certificates
        for cert in rustls_native_certs::load_native_certs()
            .map_err(|e| SecurityError::Certificate(format!("Failed to load native certs: {}", e)))? {
            root_store.add(cert)
                .map_err(|e| SecurityError::Certificate(format!("Failed to add cert: {:?}", e)))?;
        }
        
        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
            
        Ok(config)
    }
    
    /// Encrypt sensitive data using AES-GCM
    pub fn encrypt_data(&self, data: &[u8], key: &[u8]) -> SecurityResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(SecurityError::Encryption("Key must be 32 bytes for AES-256".to_string()));
        }
        
        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, key)
            .map_err(|e| SecurityError::Encryption(format!("Failed to create key: {:?}", e)))?;
        let sealing_key = aead::LessSafeKey::new(unbound_key);
        
        // Generate random nonce
        let mut nonce_bytes = vec![0u8; 12];
        self.rng.fill(&mut nonce_bytes)
            .map_err(|e| SecurityError::Encryption(format!("Failed to generate nonce: {:?}", e)))?;
        
        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes.clone().try_into().unwrap());
        
        let mut in_out = data.to_vec();
        sealing_key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|e| SecurityError::Encryption(format!("Encryption failed: {:?}", e)))?;
        
        // Prepend nonce to encrypted data
        let mut result = nonce_bytes;
        result.extend_from_slice(&in_out);
        
        Ok(result)
    }
    
    /// Decrypt data using AES-GCM
    pub fn decrypt_data(&self, encrypted_data: &[u8], key: &[u8]) -> SecurityResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(SecurityError::Encryption("Key must be 32 bytes for AES-256".to_string()));
        }
        
        if encrypted_data.len() < 12 {
            return Err(SecurityError::Encryption("Encrypted data too short".to_string()));
        }
        
        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, key)
            .map_err(|e| SecurityError::Encryption(format!("Failed to create key: {:?}", e)))?;
        let opening_key = aead::LessSafeKey::new(unbound_key);
        
        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes)
            .map_err(|e| SecurityError::Encryption(format!("Invalid nonce: {:?}", e)))?;
        
        let mut in_out = ciphertext.to_vec();
        let plaintext = opening_key.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|e| SecurityError::Encryption(format!("Decryption failed: {:?}", e)))?;
        
        Ok(plaintext.to_vec())
    }
    
    /// Verify signature of downloaded programs and firmware
    pub fn verify_program_signature(
        &self,
        program_data: &[u8],
        signature: &[u8],
        public_key_id: &str,
    ) -> SecurityResult<bool> {
        // Compute hash of program data
        let hash = self.compute_sha256(program_data);
        
        // In production, this would:
        // 1. Retrieve the public key using public_key_id
        // 2. Verify the signature against the hash using proper cryptographic verification
        // For now, we'll do a simplified check
        
        if signature.is_empty() || public_key_id.is_empty() {
            return Ok(false);
        }
        
        // Placeholder verification - in production would use actual signature verification
        Ok(hash.len() == 32 && signature.len() >= 64)
    }
    
    /// Generate HMAC for message authentication
    pub fn generate_hmac(&self, message: &[u8], key: &[u8]) -> SecurityResult<Vec<u8>> {
        let key = hmac::Key::new(hmac::HMAC_SHA256, key);
        let tag = hmac::sign(&key, message);
        Ok(tag.as_ref().to_vec())
    }
    
    /// Verify HMAC for message authentication
    pub fn verify_hmac(&self, message: &[u8], key: &[u8], expected_hmac: &[u8]) -> SecurityResult<bool> {
        let key = hmac::Key::new(hmac::HMAC_SHA256, key);
        match hmac::verify(&key, message, expected_hmac) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// Generate secure random number for cryptographic operations
    pub fn generate_secure_random(&self, length: usize) -> SecurityResult<Vec<u8>> {
        self.generate_random_bytes(length)
    }
}

/// Parse certificate information from PEM format
fn parse_certificate_info(cert_pem: &str, id: &str) -> SecurityResult<CertificateInfo> {
    // For now, create a simplified certificate info without full parsing
    // In production, this would use proper X.509 parsing
    let fingerprint = hex::encode(ring::digest::digest(&ring::digest::SHA256, cert_pem.as_bytes()).as_ref());
    
    Ok(CertificateInfo {
        id: id.to_string(),
        subject: "CN=Test Certificate".to_string(),
        issuer: "CN=Test CA".to_string(),
        serial_number: "123456".to_string(),
        not_before: DateTime::from_timestamp(1704067200, 0).unwrap_or_else(|| Utc::now()), // 2024-01-01
        not_after: DateTime::from_timestamp(1767225600, 0).unwrap_or_else(|| Utc::now()),  // 2026-01-01
        fingerprint,
        key_usage: vec!["digital_signature".to_string(), "key_encipherment".to_string()],
    })
}

/// Parse private key information from PEM format
fn parse_private_key_info(key_pem: &str, id: &str) -> SecurityResult<PrivateKeyInfo> {
    // For now, create a simplified key info without full parsing
    // In production, this would use proper key parsing
    Ok(PrivateKeyInfo {
        id: id.to_string(),
        algorithm: "RSA".to_string(),
        key_data: key_pem.as_bytes().to_vec(),
        created_at: Utc::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_in_memory_certificate_store() {
        let store = InMemoryCertificateStore::new();
        
        // Test certificate storage and retrieval
        let cert_pem = include_str!("../tests/fixtures/test_cert.pem");
        let result = store.store_certificate(cert_pem, "test_cert").await;
        assert!(result.is_ok());
        
        let retrieved = store.get_certificate("test_cert").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), cert_pem);
    }
    
    #[tokio::test]
    async fn test_security_manager_random_generation() {
        let store = Arc::new(InMemoryCertificateStore::new());
        let manager = SecurityManager::new(store);
        
        let random_bytes = manager.generate_random_bytes(32).unwrap();
        assert_eq!(random_bytes.len(), 32);
        
        // Generate another set and ensure they're different
        let random_bytes2 = manager.generate_random_bytes(32).unwrap();
        assert_ne!(random_bytes, random_bytes2);
    }
    
    #[tokio::test]
    async fn test_security_manager_key_generation() {
        let store = Arc::new(InMemoryCertificateStore::new());
        let manager = SecurityManager::new(store);
        
        let (private_key, public_key) = manager.generate_rsa_key_pair().unwrap();
        assert_eq!(private_key.len(), 2048);
        assert_eq!(public_key.len(), 256);
    }
    
    #[test]
    fn test_sha256_computation() {
        let store = Arc::new(InMemoryCertificateStore::new());
        let manager = SecurityManager::new(store);
        
        let data = b"test data";
        let hash = manager.compute_sha256(data);
        assert_eq!(hash.len(), 32); // SHA-256 produces 32-byte hash
        
        // Verify deterministic behavior
        let hash2 = manager.compute_sha256(data);
        assert_eq!(hash, hash2);
    }
}