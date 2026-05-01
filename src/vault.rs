// src/vault.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use thiserror::Error;

// TYPES DE SECRETS 

/// Représente un secret stocké dans le vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub key: String,          // ex: "DATABASE_URL"
    pub value: String,        // ex: "postgres://user:pass@host/db"
}

/// Une entrée dans le vault avec métadonnées (étendu en Partie 4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub secret: Secret,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,  // None = pas d'expiration
    pub tags: Vec<String>,
}

/// Le vault complet (sera chiffré sur disque)
#[derive(Debug, Serialize, Deserialize)]
pub struct Vault {
    pub entries: Vec<VaultEntry>,
}

impl Vault {
    pub fn new() -> Self {
        Vault { entries: Vec::new() }
    }

    /// Insère ou met à jour une entrée
    pub fn set(&mut self, key: String, value: String) {
        // Cherche si la clé existe déjà
        if let Some(entry) = self.entries.iter_mut().find(|e| e.secret.key == key) {
            entry.secret.value = value;
        } else {
            self.entries.push(VaultEntry {
                secret: Secret { key, value },
                created_at: Utc::now(),
                expires_at: None,
                tags: Vec::new(),
            });
        }
    }

    pub fn get(&self, key: &str) -> Option<&VaultEntry> {
        self.entries.iter().find(|e| e.secret.key == key)
    }

    pub fn delete(&mut self, key: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.secret.key != key);
        self.entries.len() < before  // true si supprimé
    }

    pub fn list(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.secret.key.as_str()).collect()
    }
}

//  ERREURS 

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("Authentification échouée : mot de passe incorrect")]
    AuthenticationFailed,

    #[error("Secret introuvable : {0}")]
    SecretNotFound(String),

    #[error("Le vault n'est pas initialisé. Lancez : vault init")]
    VaultNotInitialized,

    #[error("Fichier vault corrompu : {0}")]
    CorruptedVault(String),

    #[error("Erreur de chiffrement : {0}")]
    CryptoError(String),

    #[error("Erreur d'I/O : {0}")]
    IoError(#[from] std::io::Error),
}
