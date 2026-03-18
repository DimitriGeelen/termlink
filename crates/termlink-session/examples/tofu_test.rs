//! Quick TOFU integration test against a remote hub.
//! Usage: cargo run --package termlink-session --example tofu_test -- <host:port> <secret>

use termlink_session::client::Client;
use termlink_session::auth::{self, PermissionScope};
use termlink_protocol::TransportAddr;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <host:port> <hex-secret>", args[0]);
        std::process::exit(1);
    }

    let parts: Vec<&str> = args[1].split(':').collect();
    let host = parts[0].to_string();
    let port: u16 = parts[1].parse().expect("invalid port");
    let hex_secret = &args[2];

    // Convert hex secret to bytes
    let secret_bytes: Vec<u8> = (0..hex_secret.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_secret[i..i + 2], 16).expect("valid hex"))
        .collect();
    let secret: auth::TokenSecret = secret_bytes.try_into().expect("secret must be 32 bytes");

    // Generate a proper capability token
    let token = auth::create_token(&secret, PermissionScope::Execute, "", 3600);

    let addr = TransportAddr::Tcp { host, port };

    println!("Connecting to {} with TOFU...", args[1]);
    match Client::connect_addr(&addr).await {
        Ok(mut client) => {
            println!("PASS: TOFU TLS handshake succeeded");

            // Check known_hubs
            let kh_path = termlink_session::tofu::known_hubs_path();
            if kh_path.exists() {
                let content = std::fs::read_to_string(&kh_path).unwrap_or_default();
                println!("PASS: known_hubs created at {}", kh_path.display());
                for line in content.lines() {
                    if !line.starts_with('#') && !line.is_empty() {
                        println!("  {}", line);
                    }
                }
            }

            // Auth (hub uses hub.auth with HMAC-signed capability token)
            match client.call("hub.auth", serde_json::json!("auth"), serde_json::json!({"token": token.raw})).await {
                Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                    println!("PASS: Auth succeeded: {}", serde_json::to_string_pretty(&r.result).unwrap());
                }
                Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                    println!("FAIL: Auth failed: {} {}", e.error.code, e.error.message);
                    std::process::exit(1);
                }
                Err(e) => {
                    println!("FAIL: Auth error: {e}");
                    std::process::exit(1);
                }
            }

            // List sessions
            match client.call("hub.list", serde_json::json!("t1"), serde_json::json!({})).await {
                Ok(resp) => {
                    println!("PASS: hub.list succeeded");
                    match resp {
                        termlink_protocol::jsonrpc::RpcResponse::Success(r) => {
                            println!("  {}", serde_json::to_string_pretty(&r.result).unwrap());
                        }
                        termlink_protocol::jsonrpc::RpcResponse::Error(e) => {
                            println!("  Error: {} {}", e.error.code, e.error.message);
                        }
                    }
                }
                Err(e) => println!("FAIL: hub.list failed: {e}"),
            }
        }
        Err(e) => {
            println!("FAIL: TOFU TLS handshake failed: {e}");
            std::process::exit(1);
        }
    }
}
