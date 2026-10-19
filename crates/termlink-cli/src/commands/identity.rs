//! CLI command handlers for agent cryptographic identity.
//! CLI glue for agent cryptographic identity (T-1159).

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use serde_json::json;

use termlink_session::agent_identity::{Identity, IdentityError, check_detached_hex, identity_path};

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

/// T-3485: sign a file's bytes with the resolved per-agent identity. Uses the
/// READ-ONLY resolver: a key minted on the spot would produce a signature from an
/// identity nobody has pinned, so a missing key is exit 3 and nothing is created.
pub(crate) fn cmd_identity_sign(file: &std::path::Path, json_output: bool) -> Result<()> {
    let (src, found) = super::channel::load_identity_no_create()?;
    let ident = match found {
        Ok(ident) => ident,
        Err(path) => {
            if json_output {
                println!(
                    "{}",
                    json!({
                        "ok": false,
                        "error": "no_identity",
                        "path": path.display().to_string(),
                        "source": src.kind(),
                        "hint": "read-only sign: no key exists for this identity, and none was created",
                    })
                );
            } else {
                eprintln!(
                    "No identity at {} ({}); nothing was created, nothing was signed.",
                    path.display(),
                    src.kind()
                );
            }
            std::process::exit(3);
        }
    };
    let bytes = std::fs::read(file)
        .unwrap_or_else(|e| bad_input_exit(json_output, "unreadable_file", &format!("cannot read {}: {e}", file.display())));
    let signature_hex = ident.sign_hex(&bytes);
    if json_output {
        println!(
            "{}",
            json!({
                "ok": true,
                "signature_hex": signature_hex,
                "fingerprint": ident.fingerprint(),
                "public_key_hex": ident.public_key_hex(),
                "source": src.kind(),
            })
        );
    } else {
        println!("{signature_hex}");
    }
    Ok(())
}

/// T-3485: verify a detached signature over a file. Exit 0 valid, 1 invalid (says
/// which check failed), 2 bad input (unreadable file or malformed hex). Never writes.
pub(crate) fn cmd_identity_verify(
    pubkey_hex: &str,
    sig_hex: &str,
    file: &std::path::Path,
    fingerprint: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let bytes = std::fs::read(file)
        .unwrap_or_else(|e| bad_input_exit(json_output, "unreadable_file", &format!("cannot read {}: {e}", file.display())));
    let check = match check_detached_hex(pubkey_hex, sig_hex, &bytes) {
        Ok(check) => check,
        Err(e) => bad_input_exit(json_output, "bad_input", &e.to_string()),
    };
    let fingerprint_ok = fingerprint.is_none_or(|want| want.trim().eq_ignore_ascii_case(&check.fingerprint));
    let reason = if !check.signature_ok {
        Some("bad_signature")
    } else if !fingerprint_ok {
        Some("fingerprint_mismatch")
    } else {
        None
    };
    let Some(reason) = reason else {
        if json_output {
            println!(
                "{}",
                json!({"ok": true, "valid": true, "fingerprint": check.fingerprint})
            );
        } else {
            println!("VALID: signature verifies (fingerprint {})", check.fingerprint);
        }
        return Ok(());
    };
    if json_output {
        println!(
            "{}",
            json!({
                "ok": false,
                "valid": false,
                "reason": reason,
                "fingerprint": check.fingerprint,
                "expected_fingerprint": fingerprint,
            })
        );
    } else if reason == "bad_signature" {
        println!("INVALID: bad signature (the file bytes or the signature differ from what was signed)");
    } else {
        println!(
            "INVALID: fingerprint mismatch (key is {}, expected {})",
            check.fingerprint,
            fingerprint.unwrap_or_default()
        );
    }
    std::process::exit(1);
}

/// Report a bad-input failure and exit 2 (distinct from 1 = signature invalid).
fn bad_input_exit(json_output: bool, error: &str, detail: &str) -> ! {
    if json_output {
        println!("{}", json!({"ok": false, "error": error, "detail": detail}));
    } else {
        eprintln!("{detail}");
    }
    std::process::exit(2);
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
