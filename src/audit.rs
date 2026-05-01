// src/audit.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

/// Toutes les opérations traçables
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Operation {
    Init,
    Set  { key: String },
    Get  { key: String },
    Delete { key: String },
    List,
    Rotate,
    Exec { command: String },
}

/// Une entrée dans le journal d'audit
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp:  DateTime<Utc>,
    pub operation:  Operation,
    pub process_id: u32,
    pub user:       String,
}

pub struct AuditLog;

impl AuditLog {
    /// Chemin du fichier d'audit (vault.audit à côté du vault)
    pub fn audit_path(vault_path: &Path) -> std::path::PathBuf {
        vault_path.with_extension("audit")
    }

    /// Ajoute une entrée dans le journal (append-only, format JSON Lines)
    pub fn append(vault_path: &Path, operation: Operation) {
        let entry = AuditEntry {
            timestamp:  Utc::now(),
            operation,
            process_id: std::process::id(),
            // Récupère le nom d'utilisateur système (Linux/macOS)
            user: std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))  // Windows
                .unwrap_or_else(|_| "unknown".to_string()),
        };

        // Sérialisation JSON + newline (format JSON Lines)
        if let Ok(json) = serde_json::to_string(&entry) {
            let audit_path = Self::audit_path(vault_path);
            // OpenOptions::append ouvre ou crée, et n'écrase jamais
            if let Ok(mut f) = OpenOptions::new()
                .create(true)
                .append(true)  // <-- Clé : append-only
                .open(&audit_path)
            {
                let _ = writeln!(f, "{}", json);
            }
        }
    }

    /// Lit et affiche le journal d'audit
    #[allow(dead_code)]
    pub fn read(vault_path: &Path) -> Vec<AuditEntry> {
        let audit_path = Self::audit_path(vault_path);
        let content = std::fs::read_to_string(&audit_path).unwrap_or_default();
        content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }
}
