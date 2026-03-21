use anyhow::{Context, Result};

use termlink_session::manager;

pub(crate) async fn cmd_token_create(target: &str, scope: &str, ttl: u64) -> Result<()> {
    use termlink_session::auth;

    let sessions_dir = termlink_session::discovery::sessions_dir();
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let secret_hex = reg
        .token_secret
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!(
            "Session '{}' does not have token auth enabled. Register with --token-secret.",
            target
        ))?;

    let secret_bytes: auth::TokenSecret = {
        if secret_hex.len() != 64 {
            anyhow::bail!("Invalid token_secret in registration (expected 64 hex chars)");
        }
        let mut bytes = [0u8; 32];
        for i in 0..32 {
            bytes[i] = u8::from_str_radix(&secret_hex[i * 2..i * 2 + 2], 16)
                .context("Invalid hex in token_secret")?;
        }
        bytes
    };

    let permission_scope = auth::parse_scope(scope)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let token = auth::create_token(&secret_bytes, permission_scope, reg.id.as_str(), ttl);

    println!("{}", token.raw);
    eprintln!("Scope: {scope}, TTL: {ttl}s, Session: {}", reg.id);

    let _ = sessions_dir; // suppress unused
    Ok(())
}

pub(crate) fn cmd_token_inspect(token_str: &str) -> Result<()> {
    use base64::Engine;

    let parts: Vec<&str> = token_str.splitn(2, '.').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid token format (expected payload.signature)");
    }

    let payload_json = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[0])
        .context("Invalid base64 in token payload")?;

    let payload: serde_json::Value =
        serde_json::from_slice(&payload_json).context("Invalid JSON in token payload")?;

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

    Ok(())
}
