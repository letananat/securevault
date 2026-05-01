#  SecureVault

> Gestionnaire de secrets et configuration chiffrée en ligne de commande, écrit en **Rust**.

![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange?logo=rust)
![Status](https://img.shields.io/badge/status-fonctionnel-brightgreen)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)


## Présentation

SecureVault est un gestionnaire de secrets minimaliste inspiré de [HashiCorp Vault](https://www.vaultproject.io/), conçu pour résoudre un problème critique et très répandu chez les développeurs : **stocker des mots de passe, clés API et tokens en clair dans les fichiers de configuration**.


#  Ce que beaucoup font (dangereux)
DATABASE_URL=postgres://user:monmotdepasse@prod/db   # visible dans Git !

# Ce que SecureVault permet
vault set DATABASE_URL "postgres://user:monmotdepasse@prod/db"
vault exec -- node server.js   # secrets injectés de façon sécurisée
```

SecureVault chiffre vos secrets avec **AES-256-GCM** (niveau militaire), les stocke dans un fichier binaire local, et les expose uniquement à des processus enfants via des variables d'environnement, sans jamais les écrire en clair sur le disque.


## Fonctionnalités

 Fonctionnalités et Descriptions 
 **Chiffrement AES-256-GCM** correspond au Chiffrement symétrique authentifié et garantit confidentialité ET intégrité 
**Dérivation PBKDF2** correspond a la Transformation du mot de passe en clé cryptographique robuste (100 000 itérations) 
**Persistance binaire** correspond au Format binaire structuré avec magic bytes, version et sel cryptographique 
**CLI complète** correspond aux 7 commandes : `init`, `set`, `get`, `delete`, `list`, `rotate`, `exec`
**Audit log immuable** correspond au Journal append-only en JSON Lines — chaque accès est tracé 
**Injection d'environnement**  corespond a l'injection des secrets dans un processus enfant sans exposer le shell parent 
**Rotation de mot de passe** correspond au changement de mot de passe maître avec nouveau sel cryptographique 
**Expiration des secrets** correspond a chaque secret qui peut avoir une date d'expiration 


##  Architecture

securevault/
├── Cargo.toml              # Dépendances et métadonnées du projet
└── src/
    ├── main.rs             # Point d'entrée — routage des commandes CLI
    ├── crypto.rs           # Moteur cryptographique (PBKDF2 + AES-256-GCM)
    ├── vault.rs            # Types de données : Secret, VaultEntry, Vault, VaultError
    ├── storage.rs          # Persistance binaire chiffrée sur disque
    ├── cli.rs              # Définition des commandes clap + saisie sécurisée
    ├── audit.rs            # Journal d'audit immuable (append-only JSON Lines)
    └── exec.rs             # Injection de secrets dans des processus enfants


### Flux de données

Mot de passe utilisateur
        │
        ▼
   PBKDF2-SHA256 ──── Sel aléatoire (256 bits)
   (100 000 tours)
        │
        ▼
   Clé AES-256 (32 octets)
        │
        ├──[CHIFFREMENT]──► Nonce aléatoire (12 octets)
        │                          │
        │                          ▼
        │                   AES-256-GCM
        │                          │
        │                          ▼
        │              [MAGIC][VERSION][SEL][NONCE+CIPHERTEXT]
        │                          │
        │                          ▼
        │                   Fichier .vault sur disque
        │
        └──[DÉCHIFFREMENT]──► Lecture fichier → vérif. magic → dériver clé → déchiffrer → désérialiser




## Installation et démarrage rapide

### Prérequis

- [Rust](https://rustup.rs/) (édition 2021, version stable)

```bash
# Vérifier l'installation de Rust
rustc --version
cargo --version
```

### Cloner et compiler

```bash
git clone https://github.com/darelle-kameni/securevault.git
cd securevault
cargo build --release
```

L'exécutable se trouve dans `target/release/securevault` (Linux/macOS) ou `target\release\securevault.exe` (Windows).

### Utilisation directe avec cargo

    bash
cargo run -- <commande>




## Guide d'utilisation

### 1. Initialiser un vault

```bash
cargo run -- init
# Mot de passe maître : ********
# Confirmer le mot de passe : ********
# [✓] Vault initialisé avec succès à ".vault"
```

Crée le fichier `.vault` dans le répertoire courant. Le mot de passe doit faire **au minimum 8 caractères**.


### 2. Stocker un secret

```bash
cargo run -- set DATABASE_URL "postgres://alice:s3cr3t@localhost/mydb"
cargo run -- set API_KEY "sk-abc123xyz789"
cargo run -- set REDIS_URL "redis://localhost:6379"
```


### 3. Récupérer un secret

```bash
cargo run -- get DATABASE_URL
# Mot de passe maître : ********
# postgres://alice:s3cr3t@localhost/mydb
```


### 4. Lister les clés

```bash
cargo run -- list
# Mot de passe maître : ********
# Secrets stockés (1) :
#   - DATABASE_URL
```

> Les **valeurs** ne sont jamais affichées — seulement les clés.


### 5. Supprimer un secret

```bash
cargo run -- delete API_KEY
# Mot de passe maître : ********
# [✓] Secret 'API_KEY' supprimé.
```

---

### 6. Changer le mot de passe maître

```bash
cargo run -- rotate
# Ancien mot de passe : ********
# Mot de passe maître : ********     ← nouveau
# Confirmer le mot de passe : ********
# [✓] Mot de passe maître changé avec succès.
```

Un **nouveau sel cryptographique** est généré à chaque rotation — les deux versions du vault sont cryptographiquement indépendantes.


### 7. Exécuter une commande avec les secrets injectés

```bash
cargo run -- exec -- node server.js
cargo run -- exec -- python app.py
cargo run -- exec -- printenv DATABASE_URL
```

Les secrets sont injectés comme variables d'environnement dans le processus enfant **uniquement** — le shell parent n'y a pas accès.

---

### 8. Spécifier un vault personnalisé

```bash
# Utiliser un fichier vault différent
cargo run -- -v /chemin/vers/mon.vault get DATABASE_URL
```

---

##  Détails cryptographiques

### AES-256-GCM (Chiffrement)

- **Algorithme** : AES (Advanced Encryption Standard) en mode GCM (Galois/Counter Mode)
- **Clé** : 256 bits (32 octets) — issue de PBKDF2
- **Nonce** : 96 bits (12 octets) — généré aléatoirement à **chaque** chiffrement
- **Tag d'authentification** : 128 bits — détecte toute modification du ciphertext
- **Propriété AEAD** : garantit simultanément confidentialité ET intégrité

### PBKDF2 (Dérivation de clé)

- **Fonction** : PBKDF2-HMAC-SHA256
- **Sel** : 256 bits (32 octets), généré aléatoirement par `OsRng`
- **Itérations** : 100 000 — rend la force brute computationnellement très coûteuse
- **Sortie** : clé de 256 bits prête pour AES-256

### Format du fichier `.vault`

```
┌─────────────┬──────────┬────────────────┬──────────────────────────────┐
│ Magic bytes │ Version  │   Sel PBKDF2   │      Données chiffrées       │
│  "SVLT"     │  u16 LE  │   32 octets    │  Nonce (12o) + Ciphertext    │
│  4 octets   │ 2 octets │                │         (variable)           │
└─────────────┴──────────┴────────────────┴──────────────────────────────┘
```


## Journal d'audit

Chaque opération est automatiquement enregistrée dans `.vault.audit` au format **JSON Lines** :

```json
{"timestamp":"2026-04-28T10:23:15Z","operation":{"Set":{"key":"DATABASE_URL"}},"process_id":4521,"user":"alice"}
{"timestamp":"2026-04-28T10:24:02Z","operation":{"Get":{"key":"API_KEY"}},"process_id":4522,"user":"alice"}
{"timestamp":"2026-04-28T11:00:00Z","operation":"Rotate","process_id":4600,"user":"admin"}
```

Le journal est **immuable** (append-only) — aucune entrée ne peut être supprimée sans détruire tout le fichier. Il est consultable avec des outils standards :

```bash
# Voir toutes les opérations
cat .vault.audit

# Filtrer les accès à une clé spécifique
grep "DATABASE_URL" .vault.audit

# Compter le nombre d'opérations
wc -l .vault.audit

# Parser avec jq
cat .vault.audit | jq '.operation'
```


## Dépendances

| Crate | Version | Rôle |
|---|---|---|
| `aes-gcm` | 0.10 | Chiffrement AES-256-GCM (AEAD) |
| `pbkdf2` | 0.12 | Dérivation de clé basée sur mot de passe |
| `sha2` | 0.10 | SHA-256 pour PBKDF2 |
| `rand` | 0.8 | Génération cryptographiquement sûre (OsRng) |
| `serde` + `bincode` | 1 | Sérialisation binaire compacte du vault |
| `serde_json` | 1 | Sérialisation JSON du journal d'audit |
| `clap` | 4 | Interface CLI avec API derive |
| `rpassword` | 7 | Saisie de mot de passe sans écho terminal |
| `chrono` | 0.4 | Timestamps UTC pour l'audit log |
| `thiserror` | 1 | Types d'erreur personnalisés et expressifs |



##  Limitations connues

- **Stockage local uniquement** — pas de synchronisation réseau (contrairement à HashiCorp Vault)
- **Pas de zérotisation mémoire** — les secrets en RAM ne sont pas effacés explicitement après usage (amélioration possible avec la crate `zeroize`)
- **Permissions fichier** — sur Windows, les permissions ne sont pas restreintes au propriétaire (sur Linux/macOS, un `chmod 600 .vault` est recommandé)
- **Pas de multi-utilisateurs** — un seul mot de passe maître par vault


## Comparaison avec HashiCorp Vault

| Critère | SecureVault | HashiCorp Vault |
|---------|-------------|-----------------|
| Chiffrement | AES-256-GCM | AES-256-GCM |
| Dérivation de clé | PBKDF2 | Shamir's Secret Sharing |
| Stockage | Fichier local | Cluster distribué |
| Interface | CLI | REST API + CLI + UI |
| Audit | JSON Lines local | SIEM / Syslog |
| Cas d'usage | Dev local, serveurs simples | Production enterprise |
| Complexité | Minimal | Très complexe |

## Développement

```bash
# Compiler en mode debug
cargo build

# Compiler en mode release (optimisé)
cargo build --release

# Lancer les tests unitaires
cargo test

# Vérifier sans compiler
cargo check

# Voir les warnings détaillés
cargo clippy
```


## Exemple de session complète

```bash
$ cargo run -- init
Mot de passe maître : ********
Confirmer le mot de passe : ********
[✓] Vault initialisé avec succès à ".vault"

$ cargo run -- set DATABASE_URL "postgres://alice:s3cr3t@prod/db"
Mot de passe maître : ********
[✓] Secret 'DATABASE_URL' enregistré.

$ cargo run -- set API_KEY "sk-prod-abc123"
Mot de passe maître : ********
[✓] Secret 'API_KEY' enregistré.

$ cargo run -- list
Mot de passe maître : ********
Secrets stockés (2) :
  - DATABASE_URL
  - API_KEY

$ cargo run -- get DATABASE_URL
Mot de passe maître : ********
postgres://j'aime_le_Rust/db

$ cargo run -- exec -- printenv DATABASE_URL
Mot de passe maître : ********
postgres://j'aime_le_Rust/db

$ cargo run -- delete API_KEY
Mot de passe maître : ********
[✓] Secret 'API_KEY' supprimé.

$ cargo run -- rotate
Ancien mot de passe : ********
Mot de passe maître : ********
Confirmer le mot de passe : ********
[✓] Mot de passe maître changé avec succès.
```


## Auteur

Développé dans le cadre du **Projet 4 — Programmation Système en Rust**  
Université de Douala - ENSPD / GIT/GLO 4 — 2025-2026


