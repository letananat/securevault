// src/exec.rs
use std::process::Command;
use std::path::Path;
use crate::vault::VaultError;
use crate::storage::load_vault;
use crate::cli::prompt_password;
use crate::audit::{AuditLog, Operation};

/// Exécute une commande avec tous les secrets injectés en variables d'env
pub fn handle_exec(
    vault_path: &Path,
    command_args: Vec<String>,
) -> Result<(), VaultError> {
    if command_args.is_empty() {
        eprintln!("[!] Usage : vault exec -- <commande> [args...]");
        return Ok(());
    }

    // 1. Charger le vault
    let password = prompt_password("Mot de passe maître : ")?;
    let (vault, _) = load_vault(vault_path, &password)?;

    // 2. Construire le processus enfant
    let (program, args) = command_args.split_first().unwrap();
    let mut cmd = Command::new(program);
    cmd.args(args);

    // 3. Injecter TOUS les secrets comme variables d'environnement
    for entry in &vault.entries {
        // Vérifier l'expiration
        if let Some(expires) = entry.expires_at {
            if chrono::Utc::now() > expires {
                eprintln!("[!] Secret '{}' expiré, non injecté.", entry.secret.key);
                continue;
            }
        }
        cmd.env(&entry.secret.key, &entry.secret.value);
    }

    // 4. Exécuter et attendre la fin
    let status = cmd.status()?;

    // 5. Log d'audit
    AuditLog::append(vault_path, Operation::Exec {
        command: command_args.join(" "),
    });

    // 6. Propager le code de retour du processus enfant
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

// ─── EXPORT .env ──────────────────────────────────────────────────────

/// Exporte tous les secrets au format .env
#[allow(dead_code)]
pub fn export_dotenv(
    vault_path: &Path,
    output_path: &Path,
) -> Result<(), VaultError> {
    let password = prompt_password("Mot de passe maître : ")?;
    let (vault, _) = load_vault(vault_path, &password)?;

    let mut content = String::from("# Généré par SecureVault\n");
    for entry in &vault.entries {
        // Échapper les guillemets dans la valeur
        let escaped = entry.secret.value.replace('"', "\\\"");
        content.push_str(&format!("{}=\"{}\"\n",
            entry.secret.key, escaped));
    }

    std::fs::write(output_path, content)?;
    eprintln!("[!] ATTENTION : Le fichier .env contient des secrets en clair !");
    eprintln!("    Ajoutez-le à votre .gitignore immédiatement.");
    Ok(())
}
