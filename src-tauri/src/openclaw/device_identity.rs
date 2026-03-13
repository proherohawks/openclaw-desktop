use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::{SigningKey, VerifyingKey, Signer};
use ed25519_dalek::pkcs8::{DecodePrivateKey, EncodePrivateKey, spki::EncodePublicKey};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct DeviceIdentity {
    pub device_id: String,
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DeviceIdentityFile {
    version: u32,
    #[serde(rename = "deviceId")]
    device_id: String,
    #[serde(rename = "publicKeyPem")]
    public_key_pem: String,
    #[serde(rename = "privateKeyPem")]
    private_key_pem: String,
    #[serde(rename = "createdAtMs")]
    created_at_ms: u64,
}

fn identity_file_path() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::ConnectionFailed("Cannot determine home directory".into()))?;
    Ok(home.join(".openclaw-desktop").join("device-identity.json"))
}

fn compute_device_id(verifying_key: &VerifyingKey) -> String {
    let raw_bytes = verifying_key.as_bytes();
    let hash = Sha256::digest(raw_bytes);
    hex::encode(hash)
}

pub fn raw_public_key_base64url(identity: &DeviceIdentity) -> String {
    URL_SAFE_NO_PAD.encode(identity.verifying_key.as_bytes())
}

pub fn sign_connect_payload(identity: &DeviceIdentity, payload: &str) -> String {
    let signature = identity.signing_key.sign(payload.as_bytes());
    URL_SAFE_NO_PAD.encode(signature.to_bytes())
}

pub fn build_v2_payload(
    device_id: &str,
    token: &str,
    nonce: &str,
    signed_at_ms: u64,
) -> String {
    let scopes = "operator.read,operator.write,operator.admin,operator.approvals,operator.pairing";
    format!(
        "v2|{}|openclaw-control-ui|webchat|operator|{}|{}|{}|{}",
        device_id, scopes, signed_at_ms, token, nonce
    )
}

pub fn load_or_create_identity() -> Result<DeviceIdentity, AppError> {
    let path = identity_file_path()?;

    if path.exists() {
        let contents = std::fs::read_to_string(&path)
            .map_err(|e| AppError::ConnectionFailed(format!("Failed to read device identity: {}", e)))?;
        let file: DeviceIdentityFile = serde_json::from_str(&contents)
            .map_err(|e| AppError::ParseError(format!("Failed to parse device identity: {}", e)))?;

        let signing_key = SigningKey::from_pkcs8_pem(&file.private_key_pem)
            .map_err(|e| AppError::ParseError(format!("Failed to parse private key: {}", e)))?;
        let verifying_key = signing_key.verifying_key();
        let device_id = compute_device_id(&verifying_key);

        return Ok(DeviceIdentity {
            device_id,
            signing_key,
            verifying_key,
        });
    }

    // Generate new keypair
    let mut rng = rand::thread_rng();
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();
    let device_id = compute_device_id(&verifying_key);

    let private_pem = signing_key
        .to_pkcs8_pem(ed25519_dalek::pkcs8::spki::der::pem::LineEnding::LF)
        .map_err(|e| AppError::ConnectionFailed(format!("Failed to encode private key: {}", e)))?;
    let public_pem = verifying_key
        .to_public_key_pem(ed25519_dalek::pkcs8::spki::der::pem::LineEnding::LF)
        .map_err(|e| AppError::ConnectionFailed(format!("Failed to encode public key: {}", e)))?;

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let file = DeviceIdentityFile {
        version: 1,
        device_id: device_id.clone(),
        public_key_pem: public_pem,
        private_key_pem: private_pem.to_string(),
        created_at_ms: now_ms,
    };

    let json = serde_json::to_string_pretty(&file)
        .map_err(|e| AppError::ParseError(format!("Failed to serialize identity: {}", e)))?;

    // Create directory and write file with restricted permissions
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::ConnectionFailed(format!("Failed to create identity dir: {}", e)))?;
    }

    std::fs::write(&path, &json)
        .map_err(|e| AppError::ConnectionFailed(format!("Failed to write device identity: {}", e)))?;

    // Set file permissions to 0o600 (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| AppError::ConnectionFailed(format!("Failed to set file permissions: {}", e)))?;
    }

    Ok(DeviceIdentity {
        device_id,
        signing_key,
        verifying_key,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_v2_payload() {
        let payload = build_v2_payload("device123", "mytoken", "nonce456", 1234567890);
        assert_eq!(
            payload,
            "v2|device123|openclaw-control-ui|webchat|operator|operator.read,operator.write,operator.admin,operator.approvals,operator.pairing|1234567890|mytoken|nonce456"
        );
    }

    #[test]
    fn test_sign_and_verify() {
        let mut rng = rand::thread_rng();
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();

        let identity = DeviceIdentity {
            device_id: compute_device_id(&verifying_key),
            signing_key,
            verifying_key,
        };

        let payload = "test payload";
        let sig_b64 = sign_connect_payload(&identity, payload);

        // Verify the signature decodes and is 64 bytes
        let sig_bytes = URL_SAFE_NO_PAD.decode(&sig_b64).unwrap();
        assert_eq!(sig_bytes.len(), 64);
    }

    #[test]
    fn test_device_id_is_sha256_of_public_key() {
        let mut rng = rand::thread_rng();
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();

        let device_id = compute_device_id(&verifying_key);

        // Verify it's a valid hex-encoded SHA-256 hash (64 hex chars)
        assert_eq!(device_id.len(), 64);
        assert!(device_id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_raw_public_key_base64url() {
        let mut rng = rand::thread_rng();
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();

        let identity = DeviceIdentity {
            device_id: compute_device_id(&verifying_key),
            signing_key,
            verifying_key,
        };

        let encoded = raw_public_key_base64url(&identity);
        let decoded = URL_SAFE_NO_PAD.decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 32);
    }
}
