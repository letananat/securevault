// src/crypto.rs
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
    Aes256Gcm, Nonce, Key,
};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use crate::vault::VaultError;

pub const SALT_LEN: usize = 32;    // 256 bits de sel
pub const KEY_LEN:  usize = 32;    // 256 bits pour AES-256
pub const NONCE_LEN: usize = 12;   // 96 bits pour GCM
pub const PBKDF2_ITERATIONS: u32 = 100_000;  // 100k itérations

/// Dérive une clé AES-256 à partir du mot de passe et du sel
pub fn derive_key(password: &str, salt: &[u8; SALT_LEN]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        salt,
        PBKDF2_ITERATIONS,
        &mut key,
    );
    key
}

/// Génère un sel aléatoire cryptographiquement sûr
pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Chiffre des données en clair → retourne nonce || données_chiffrées
pub fn encrypt(
    plaintext: &[u8],
    key: &[u8; KEY_LEN],
) -> Result<Vec<u8>, VaultError> {
    // Génère un nonce aléatoire unique pour chaque chiffrement
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    // Format de sortie : [nonce (12 octets)][données chiffrées]
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Déchiffre nonce || données_chiffrées → données en clair
pub fn decrypt(
    encrypted_data: &[u8],
    key: &[u8; KEY_LEN],
) -> Result<Vec<u8>, VaultError> {
    // Vérifie que le buffer est assez long
    if encrypted_data.len() < NONCE_LEN {
        return Err(VaultError::CorruptedVault(
            "Données chiffrées trop courtes".to_string()
        ));
    }

    // Sépare le nonce des données chiffrées
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    cipher
        .decrypt(nonce, ciphertext)
        // Si la clé est mauvaise, le tag GCM est invalide → AuthenticationFailed
        .map_err(|_| VaultError::AuthenticationFailed)
}

