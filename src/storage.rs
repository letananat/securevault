use std::fs;
use std::path::Path;
use crate::vault::{Vault, VaultError};
use crate::crypto::{derive_key, encrypt, decrypt, SALT_LEN};

const MAGIC: &[u8; 4] = b"SVLT";
const VERSION: u16 = 1;

pub fn save_vault(
    path: &Path,
    vault: &Vault,
    password: &str,
    salt: &[u8; SALT_LEN],
) -> Result<(), VaultError> {
    let plaintext = bincode::serialize(vault)
        .map_err(|e| VaultError::CorruptedVault(e.to_string()))?;

    let key = derive_key(password, salt);
    let ciphertext = encrypt(&plaintext, &key)?;

    let mut file_data = Vec::new();
    file_data.extend_from_slice(MAGIC);
    file_data.extend_from_slice(&VERSION.to_le_bytes());
    file_data.extend_from_slice(salt);
    file_data.extend_from_slice(&ciphertext);

    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, &file_data)?;
    fs::rename(&tmp_path, path)?;

    Ok(())
}

pub fn load_vault(
    path: &Path,
    password: &str,
) -> Result<(Vault, [u8; SALT_LEN]), VaultError> {
    let data = fs::read(path).map_err(|_| VaultError::VaultNotInitialized)?;

    let min_size = 4 + 2 + SALT_LEN + 12;
    if data.len() < min_size {
        return Err(VaultError::CorruptedVault(
            "Fichier trop court".to_string()
        ));
    }

    if &data[0..4] != MAGIC {
        return Err(VaultError::CorruptedVault(
            "Magic bytes invalides".to_string()
        ));
    }

    let version = u16::from_le_bytes([data[4], data[5]]);  
    if version != VERSION {
        return Err(VaultError::CorruptedVault(
            format!("Version {} non supportée", version)
        ));
    } 
   
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&data[6..6 + SALT_LEN]);

    let key = derive_key(password, &salt);
    let ciphertext = &data[6 + SALT_LEN..];
    let plaintext = decrypt(ciphertext, &key)?;

    let vault: Vault = bincode::deserialize(&plaintext)
        .map_err(|e| VaultError::CorruptedVault(e.to_string()))?;

    Ok((vault, salt))
}
