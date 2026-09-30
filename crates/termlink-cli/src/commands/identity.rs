//! CLI glue for agent cryptographic identity (T-1159).

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use serde_json::json;

use termlink_session::agent_identity::{Identity, IdentityError, identity_path};

/// Resolve the base directory for the identity file. Default is
/// `~/.termlink`. Tests and alternate installs can override with the
/// `TERMLINK_IDENTITY_DIR` env var.
fn identity_base_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("TERMLINK_IDENTITY_DIR") {
        return Ok(PathBuf::from(dir));
    }
    let home = std::env::var("HOME").context("HOME is not set; cannot resolve identity dir")?;
    Ok(PathBuf::from(home).join(".termlink"))
}

pub(crate) fn cmd_identity_init(force: bool, json_output: bool) -> Result<()> {
    let base = identity_base_dir()?;
    match Identity::init(&base, force) {
        Ok(ident) => print_identity_result(&base, &ident, "initialized", json_output),
        Err(IdentityError::AlreadyExists(path)) => {
            if json_output {
                super::json_error_exit(json!({
                    "ok": false,
                    "error": "already_exists",
                    "path": path.display().to_string(),
                    "hint": "run 'termlink identity rotate --force' to rotate",
                }));
            } else {
                eprintln!(
                    "Identity file already exists at {}. Run 'termlink identity rotate --force' to rotate.",
                    path.display()
                );
                std::process::exit(1);
            }
        }
        Err(e) => Err(anyhow!(e)),
    }
}

pub(crate) fn cmd_identity_show(json_output: bool, resolve: bool, no_create: bool) -> Result<()> {
    if resolve && no_create {
        // T-3286 (T-3284 GO, option C): READ-ONLY resolve. Same precedence as the
        // signing path, but a missing key is REPORTED (exit 3), never minted.
        // Plain `--resolve` still creates on first use (unchanged, pinned by test),
        // because the signing path genuinely needs a key; a check does not.
        let (src, found) = super::channel::load_identity_no_create()?;
        match found {
            Ok(ident) => {
                if json_output {
                    println!(
                        "{}",
                        json!({
                            "ok": true,
                            "action": "resolved",
                            "fingerprint": ident.fingerprint(),
                            "public_key_hex": ident.public_key_hex(),
                            "source": src.kind(),
                        })
                    );
                } else {
                    println!("Identity resolved (per-agent resolver, read-only)");
                    println!("  Fingerprint: {}", ident.fingerprint());
                    println!("  Public key:  {}", ident.public_key_hex());
                }
                return Ok(());
            }
            Err(path) => {
                use std::io::Write;
                if json_output {
                    println!(
                        "{}",
                        json!({
                            "ok": false,
                            "error": "no_identity",
                            "path": path.display().to_string(),
                            "source": src.kind(),
                            "hint": "read-only lookup (--no-create): no key exists for this identity, and none was created",
                        })
                    );
                    let _ = std::io::stdout().flush();
                } else {
                    eprintln!(
                        "No identity at {} ({}); nothing was created (--no-create).",
                        path.display(),
                        src.kind()
                    );
                }
                std::process::exit(3);
            }
        }
    }
    if resolve {
        // T-2324 / PL-236: honor the per-agent identity resolver
        // (TERMLINK_IDENTITY_FILE > TERMLINK_AGENT_ID > TERMLINK_IDENTITY_DIR >
        // shared host default) so a per-agent session reports ITS OWN
        // fingerprint, not the shared host key. This is the exact precedence the
        // signing path uses (channel::load_identity_or_create) and that
        // registration.rs::resolve_identity_key_path stamps into
        // SessionMetadata.identity_fingerprint — so the wire sender_id, the
        // registration fingerprint, and this readout all agree. Without
        // --resolve the base-dir semantics are preserved (in lockstep with
        // `identity init`/`rotate`, which operate on the base-dir key).
        let ident = super::channel::load_identity_or_create()?;
        if json_output {
            println!(
                "{}",
                json!({
                    "ok": true,
                    "action": "resolved",
                    "fingerprint": ident.fingerprint(),
                    "public_key_hex": ident.public_key_hex(),
                })
            );
        } else {
            println!("Identity resolved (per-agent resolver)");
            println!("  Fingerprint: {}", ident.fingerprint());
            println!("  Public key:  {}", ident.public_key_hex());
        }
        return Ok(());
    }
    let base = identity_base_dir()?;
    let path = identity_path(&base);
    if !path.exists() {
        if json_output {
            super::json_error_exit(json!({
                "ok": false,
                "error": "not_initialized",
                "path": path.display().to_string(),
                "hint": "run 'termlink identity init' first",
            }));
        } else {
            eprintln!(
                "No identity at {}. Run 'termlink identity init' first.",
                path.display()
            );
            std::process::exit(1);
        }
    }
    let ident = Identity::load_or_create(&base)?;
    print_identity_result(&base, &ident, "loaded", json_output)
}

pub(crate) fn cmd_identity_rotate(force: bool, json_output: bool) -> Result<()> {
    if !force {
        if json_output {
            super::json_error_exit(json!({
                "ok": false,
                "error": "force_required",
                "hint": "rotation is destructive; re-run with --force",
            }));
        } else {
            eprintln!("Rotation is destructive. Re-run with --force to proceed.");
            std::process::exit(1);
        }
    }
    let base = identity_base_dir()?;
    let ident = Identity::init(&base, true)?;
    print_identity_result(&base, &ident, "rotated", json_output)
}

fn print_identity_result(
    base: &std::path::Path,
    ident: &Identity,
    action: &str,
    json_output: bool,
) -> Result<()> {
    let path = identity_path(base);
    if json_output {
        println!(
            "{}",
            json!({
                "ok": true,
                "action": action,
                "path": path.display().to_string(),
                "fingerprint": ident.fingerprint(),
                "public_key_hex": ident.public_key_hex(),
            })
        );
    } else {
        println!("Identity {action}");
        println!("  Path:        {}", path.display());
        println!("  Fingerprint: {}", ident.fingerprint());
        println!("  Public key:  {}", ident.public_key_hex());
    }
    Ok(())
}
