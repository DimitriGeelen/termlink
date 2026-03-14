use std::path::Path;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use termlink_protocol::control;
use termlink_protocol::jsonrpc::{Request, RpcResponse};
use termlink_protocol::TransportAddr;

/// A JSON-RPC client that connects to a session via any transport.
pub struct Client {
    writer: Box<dyn tokio::io::AsyncWrite + Send + Unpin>,
    reader: tokio::io::Lines<BufReader<Box<dyn tokio::io::AsyncRead + Send + Unpin>>>,
}

impl Client {
    /// Connect to a session's control plane via transport address.
    pub async fn connect_addr(addr: &TransportAddr) -> std::io::Result<Self> {
        match addr {
            TransportAddr::Unix { path } => {
                let stream = UnixStream::connect(path).await?;
                let (reader, writer) = tokio::io::split(stream);
                Ok(Self {
                    writer: Box::new(writer),
                    reader: BufReader::new(Box::new(reader) as Box<dyn tokio::io::AsyncRead + Send + Unpin>).lines(),
                })
            }
            TransportAddr::Tcp { host, port } => {
                let stream = tokio::net::TcpStream::connect((host.as_str(), *port)).await?;
                stream.set_nodelay(true)?;
                let (reader, writer) = tokio::io::split(stream);
                Ok(Self {
                    writer: Box::new(writer),
                    reader: BufReader::new(Box::new(reader) as Box<dyn tokio::io::AsyncRead + Send + Unpin>).lines(),
                })
            }
        }
    }

    /// Connect to a session's control plane socket (convenience for Unix paths).
    pub async fn connect(socket_path: &Path) -> std::io::Result<Self> {
        Self::connect_addr(&TransportAddr::unix(socket_path)).await
    }

    /// Send a JSON-RPC request and wait for the response.
    pub async fn call(
        &mut self,
        method: &str,
        id: serde_json::Value,
        params: serde_json::Value,
    ) -> Result<RpcResponse, ClientError> {
        let req = Request::new(method, id, params);
        let mut json = serde_json::to_string(&req)?;
        json.push('\n');

        self.writer.write_all(json.as_bytes()).await?;

        let line = self
            .reader
            .next_line()
            .await?
            .ok_or(ClientError::ConnectionClosed)?;

        let resp: RpcResponse = serde_json::from_str(&line)?;
        Ok(resp)
    }

    /// Authenticate with a capability token (auth.token RPC method).
    ///
    /// On success, the connection's scope is upgraded to the token's scope.
    /// Returns the granted scope string on success.
    pub async fn authenticate(&mut self, token: &str) -> Result<String, ClientError> {
        let resp = self
            .call(
                control::method::AUTH_TOKEN,
                serde_json::json!("auth"),
                serde_json::json!({"token": token}),
            )
            .await?;

        match resp {
            RpcResponse::Success(r) => {
                let scope = r.result["scope"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string();
                Ok(scope)
            }
            RpcResponse::Error(e) => Err(ClientError::AuthFailed(format!(
                "{}: {}",
                e.error.code, e.error.message
            ))),
        }
    }

    /// Send a notification (no response expected).
    pub async fn notify(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<(), ClientError> {
        let req = Request::notification(method, params);
        let mut json = serde_json::to_string(&req)?;
        json.push('\n');

        self.writer.write_all(json.as_bytes()).await?;
        Ok(())
    }
}

/// Convenience function: connect via address, send one request, return response.
pub async fn rpc_call_addr(
    addr: &TransportAddr,
    method: &str,
    params: serde_json::Value,
) -> Result<RpcResponse, ClientError> {
    let mut client = Client::connect_addr(addr).await?;
    client.call(method, serde_json::json!("cli-1"), params).await
}

/// Convenience function: connect via Unix socket path, send one request, return response.
pub async fn rpc_call(
    socket_path: &Path,
    method: &str,
    params: serde_json::Value,
) -> Result<RpcResponse, ClientError> {
    rpc_call_addr(&TransportAddr::unix(socket_path), method, params).await
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("connection closed by server")]
    ConnectionClosed,

    #[error("authentication failed: {0}")]
    AuthFailed(String),
}

/// Extract the successful result from an RpcResponse, or format the error.
pub fn unwrap_result(resp: RpcResponse) -> Result<serde_json::Value, String> {
    match resp {
        RpcResponse::Success(r) => Ok(r.result),
        RpcResponse::Error(e) => Err(format!(
            "JSON-RPC error {}: {}",
            e.error.code, e.error.message
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::SessionContext;
    use crate::identity::SessionId;
    use crate::lifecycle::SessionState;
    use crate::registration::{Registration, SessionConfig};
    use crate::server;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_socket_path() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        PathBuf::from(format!("/tmp/tl-cli-{}-{}.sock", std::process::id(), n))
    }

    #[tokio::test]
    async fn client_call_ping() {
        let socket_path = test_socket_path();
        let _ = std::fs::remove_file(&socket_path);

        let listener = tokio::net::UnixListener::bind(&socket_path).unwrap();
        let id = SessionId::generate();
        let mut reg = Registration::new(
            id,
            SessionConfig {
                display_name: Some("client-test".into()),
                ..Default::default()
            },
            socket_path.clone(),
        );
        reg.state = SessionState::Ready;
        let ctx = SessionContext::new(reg);
        let shared = Arc::new(RwLock::new(ctx));

        let shared_clone = shared.clone();
        let handle = tokio::spawn(async move {
            server::run_accept_loop(listener, shared_clone).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let resp = rpc_call(&socket_path, "termlink.ping", serde_json::json!({}))
            .await
            .unwrap();
        let result = unwrap_result(resp).unwrap();
        assert_eq!(result["display_name"], "client-test");
        assert_eq!(result["state"], "ready");

        handle.abort();
        let _ = std::fs::remove_file(&socket_path);
    }

    #[tokio::test]
    async fn client_multiple_calls() {
        let socket_path = test_socket_path();
        let _ = std::fs::remove_file(&socket_path);

        let listener = tokio::net::UnixListener::bind(&socket_path).unwrap();
        let id = SessionId::generate();
        let mut reg = Registration::new(
            id,
            SessionConfig::default(),
            socket_path.clone(),
        );
        reg.state = SessionState::Ready;
        let ctx = SessionContext::new(reg);
        let shared = Arc::new(RwLock::new(ctx));

        let shared_clone = shared.clone();
        let handle = tokio::spawn(async move {
            server::run_accept_loop(listener, shared_clone).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let mut client = Client::connect(&socket_path).await.unwrap();

        let resp1 = client
            .call("termlink.ping", serde_json::json!(1), serde_json::json!({}))
            .await
            .unwrap();
        assert!(matches!(resp1, RpcResponse::Success(_)));

        let resp2 = client
            .call("query.capabilities", serde_json::json!(2), serde_json::json!({}))
            .await
            .unwrap();
        let result = unwrap_result(resp2).unwrap();
        assert!(result["capabilities"].is_array());

        handle.abort();
        let _ = std::fs::remove_file(&socket_path);
    }
}
