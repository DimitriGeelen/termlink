use anyhow::{Context, Result};

use termlink_session::manager;

pub(crate) async fn cmd_token_create(target: &str, scope: &str, ttl: u64, json: bool) -> Result<()> {
    use termlink_session::auth;

    let sessions_dir = termlink_session::discovery::sessions_dir();
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {}", target, e)}));
                std::process::exit(1);
            }
            return Err(e).context(format!("Session '{}' not found", target));
        }
    };

    let secret_hex = match reg.token_secret.as_ref() {
        Some(s) => s,
        None => {
            let msg = format!("Session '{}' does not have token auth enabled. Register with --token-secret.", target);
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": msg}));
                std::process::exit(1);
            }
            anyhow::bail!("{}", msg);
        }
    };

    let secret_bytes: auth::TokenSecret = {
        if secret_hex.len() != 64 {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": "Invalid token_secret in registration (expected 64 hex chars)"}));
                std::process::exit(1);
            }
            anyhow::bail!("Invalid token_secret in registration (expected 64 hex chars)");
        }
        let mut bytes = [0u8; 32];
        for i in 0..32 {
            bytes[i] = match u8::from_str_radix(&secret_hex[i * 2..i * 2 + 2], 16) {
                Ok(v) => v,
                Err(e) => {
                    if json {
                        println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Invalid hex in token_secret: {}", e)}));
                        std::process::exit(1);
                    }
                    return Err(e.into());
                }
            };
        }
        bytes
    };

    let permission_scope = match auth::parse_scope(scope) {
        Ok(s) => s,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Invalid scope '{}': {}", scope, e)}));
                std::process::exit(1);
            }
            anyhow::bail!("Invalid scope '{}': {}", scope, e);
        }
    };

    let token = auth::create_token(&secret_bytes, permission_scope, reg.id.as_str(), ttl);

    if json {
        println!("{}", serde_json::json!({
            "ok": true,
            "token": token.raw,
            "scope": scope,
            "ttl": ttl,
            "session": reg.id.as_str(),
        }));
    } else {
        println!("{}", token.raw);
        eprintln!("Scope: {scope}, TTL: {ttl}s, Session: {}", reg.id);
    }

    let _ = sessions_dir; // suppress unused
    Ok(())
}

pub(crate) fn cmd_token_inspect(token_str: &str, json: bool) -> Result<()> {
    use base64::Engine;

    let parts: Vec<&str> = token_str.splitn(2, '.').collect();
    if parts.len() != 2 {
        if json {
            println!("{}", serde_json::json!({"ok": false, "error": "Invalid token format (expected payload.signature)"}));
            std::process::exit(1);
        }
        anyhow::bail!("Invalid token format (expected payload.signature)");
    }

    let payload_json = match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[0]) {
        Ok(v) => v,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "error": format!("Invalid base64 in token payload: {}", e)}));
                std::process::exit(1);
            }
            return Err(e.into());
        }
    };

    let payload: serde_json::Value = match serde_json::from_slice(&payload_json) {
        Ok(v) => v,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "error": format!("Invalid JSON in token payload: {}", e)}));
                std::process::exit(1);
            }
            return Err(e.into());
        }
    };

    if json {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let expired = payload["expires_at"].as_u64().map(|e| now > e).unwrap_or(false);
        println!("{}", serde_json::json!({
            "ok": true,
            "payload": payload,
            "expired": expired,
        }));
    } else {
        println!("{}", serde_json::to_string_pretty(&payload)?);

        if let Some(expires) = payload["expires_at"].as_u64() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now > expires {
                eprintln!("WARNING: Token has expired ({} seconds ago)", now - expires);
            } else {
                eprintln!("Expires in {} seconds", expires - now);
            }
        }
    }

    Ok(())
}
