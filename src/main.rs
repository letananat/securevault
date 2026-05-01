mod crypto;
mod vault;
mod storage;
mod cli;
mod audit;
mod exec;

use clap::Parser;
use cli::{Cli, Commands, prompt_password, prompt_password_confirm};
use storage::{load_vault, save_vault};
use crypto::generate_salt;
use audit::{AuditLog, Operation};

fn main() {
    let cli = Cli::parse();
    let path = &cli.vault_path;

    let result = match cli.command {
        Commands::Init              => handle_init(path),
        Commands::Set { key, value }=> handle_set(path, key, value),
        Commands::Get { key }       => handle_get(path, key),
        Commands::Delete { key }    => handle_delete(path, key),
        Commands::List              => handle_list(path),
        Commands::Rotate            => handle_rotate(path),
        Commands::Exec { command }  => exec::handle_exec(path, command),
    };

    if let Err(e) = result {
        eprintln!("[ERREUR] {}", e);
        std::process::exit(1);
    }
}

fn handle_init(path: &std::path::Path) -> Result<(), vault::VaultError> {
    if path.exists() {
        eprintln!("[!] Un vault existe déjà à {:?}. Utilisez rotate pour changer le mot de passe.", path);
        return Ok(());
    }
    let password = prompt_password_confirm()?;
    let salt = generate_salt();
    let vault = vault::Vault::new();
    save_vault(path, &vault, &password, &salt)?;
    println!("[✓] Vault initialisé avec succès à {:?}", path);
    Ok(())
}

fn handle_set(path: &std::path::Path, key: String, value: String) -> Result<(), vault::VaultError> {
    let password = prompt_password("Mot de passe maître : ")?;
    let (mut v, salt): (vault::Vault, _) = load_vault(path, &password)?;
    v.set(key.clone(), value);
    save_vault(path, &v, &password, &salt)?;
    AuditLog::append(path, Operation::Set { key: key.clone() });
    println!("[✓] Secret '{}' enregistré.", key);
    Ok(())
}

fn handle_get(path: &std::path::Path, key: String) -> Result<(), vault::VaultError> {
    let password = prompt_password("Mot de passe maître : ")?;
    let (v, _): (vault::Vault, _) = load_vault(path, &password)?;
    match v.get(&key) {
        Some(entry) => {
            AuditLog::append(path, Operation::Get { key: key.clone() });
            println!("{}", entry.secret.value);
        }
        None => return Err(vault::VaultError::SecretNotFound(key)),
    }
    Ok(())
}

fn handle_delete(path: &std::path::Path, key: String) -> Result<(), vault::VaultError> {
    let password = prompt_password("Mot de passe maître : ")?;
    let (mut v, salt): (vault::Vault, _) = load_vault(path, &password)?;
    if v.delete(&key) {
        save_vault(path, &v, &password, &salt)?;
        AuditLog::append(path, Operation::Delete { key: key.clone() });
        println!("[✓] Secret '{}' supprimé.", key);
    } else {
        return Err(vault::VaultError::SecretNotFound(key));
    }
    Ok(())
}

fn handle_list(path: &std::path::Path) -> Result<(), vault::VaultError> {
    let password = prompt_password("Mot de passe maître : ")?;
    let (v, _): (vault::Vault, _) = load_vault(path, &password)?;
    let keys = v.list();
    if keys.is_empty() {
        println!("(aucun secret stocké)");
    } else {
        println!("Secrets stockés ({}) :", keys.len());
        for k in keys {
            println!("  - {}", k);
        }
    }
    Ok(())
}

fn handle_rotate(path: &std::path::Path) -> Result<(), vault::VaultError> {
    let old_password = prompt_password("Ancien mot de passe : ")?;
    let (v, _): (vault::Vault, _) = load_vault(path, &old_password)?;
    let new_password = prompt_password_confirm()?;
    let new_salt = generate_salt();
    save_vault(path, &v, &new_password, &new_salt)?;
    AuditLog::append(path, Operation::Rotate);
    println!("[✓] Mot de passe maître changé avec succès.");
    Ok(())
}