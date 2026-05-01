// src/cli.rs
use clap::{Parser, Subcommand};
use rpassword::read_password;
use std::io::{self, Write};
use std::path::PathBuf;
use crate::vault::VaultError;


/// SecureVault — Gestionnaire de secrets chiffrés
#[derive(Parser)]
#[command(name = "vault", version = "0.1.0", about = "Gestionnaire de secrets chiffrés")]
pub struct Cli {
    /// Chemin du fichier vault (défaut : ~/.vault)
    #[arg(short, long, default_value = ".vault")]
    pub vault_path: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialise un nouveau vault avec un mot de passe maître
    Init,

    /// Stocke un secret : vault set DATABASE_URL postgres://...
    Set {
        key: String,
        value: String,
    },

    /// Récupère un secret : vault get DATABASE_URL
    Get { key: String },

    /// Supprime un secret : vault delete DATABASE_URL
    Delete { key: String },

    /// Liste toutes les clés stockées
    List,

    /// Change le mot de passe maître
    Rotate,

    /// Exécute une commande avec les secrets injectés
    Exec {
        #[arg(last = true)]
        command: Vec<String>,
    },
}

// ─── FONCTION UTILITAIRE : saisie sécurisée du mot de passe ──────────

pub fn prompt_password(prompt: &str) -> Result<String, VaultError> {
    print!("{}", prompt);
    io::stdout().flush()?;
    read_password().map_err(|e| VaultError::IoError(e))
}

pub fn prompt_password_confirm() -> Result<String, VaultError> {
    loop {
        let pwd1 = prompt_password("Mot de passe maître : ")?;
        let pwd2 = prompt_password("Confirmer le mot de passe : ")?;
        if pwd1 == pwd2 {
            if pwd1.len() < 8 {
                eprintln!("[!] Le mot de passe doit faire au moins 8 caractères.");
                continue;
            }
            return Ok(pwd1);
        }
        eprintln!("[!] Les mots de passe ne correspondent pas. Réessayez.");
    }
}
