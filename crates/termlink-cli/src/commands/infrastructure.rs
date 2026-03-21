use anyhow::{Context, Result};

pub(crate) async fn cmd_hub_start(tcp_addr: Option<&str>) -> Result<()> {
    let socket_path = termlink_hub::server::hub_socket_path();
    let pidfile_path = termlink_hub::pidfile::hub_pidfile_path();

    println!("Starting hub server...");
    println!("  Socket:  {}", socket_path.display());
    if let Some(addr) = tcp_addr {
        println!("  TCP:     {}", addr);
    }
    println!("  Pidfile: {}", pidfile_path.display());

    let handle = termlink_hub::server::run_with_tcp(&socket_path, tcp_addr)
        .await
        .context("Hub server error")?;

    if tcp_addr.is_some() {
        let secret_path = termlink_hub::server::hub_secret_path();
        let cert_path = termlink_hub::tls::hub_cert_path();
        println!("  Secret:  {}", secret_path.display());
        println!("  TLS cert: {}", cert_path.display());
        println!();
        println!("TCP connections use TLS with auto-generated self-signed certificate.");
        println!("Auth required. Clients must call 'hub.auth' with a token.");
        println!("Read the secret: cat {}", secret_path.display());
    }
    println!();
    println!("Listening for connections... (Ctrl+C to stop)");

    tokio::signal::ctrl_c().await.ok();
    println!();
    println!("Shutting down hub...");
    handle.shutdown();

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    println!("Hub stopped.");

    Ok(())
}

pub(crate) fn cmd_hub_stop() -> Result<()> {
    let pidfile_path = termlink_hub::pidfile::hub_pidfile_path();

    match termlink_hub::pidfile::check(&pidfile_path) {
        termlink_hub::pidfile::PidfileStatus::NotRunning => {
            println!("Hub is not running.");
        }
        termlink_hub::pidfile::PidfileStatus::Stale(pid) => {
            println!("Hub pidfile found (PID {pid}) but process is dead. Cleaning up.");
            termlink_hub::pidfile::remove(&pidfile_path);
            let socket_path = termlink_hub::server::hub_socket_path();
            let _ = std::fs::remove_file(&socket_path);
        }
        termlink_hub::pidfile::PidfileStatus::Running(pid) => {
            println!("Stopping hub (PID {pid})...");
            unsafe { libc::kill(pid as i32, libc::SIGTERM) };
            for _ in 0..20 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if !termlink_session::liveness::process_exists(pid) {
                    println!("Hub stopped.");
                    return Ok(());
                }
            }
            println!("Hub did not stop within 2 seconds. You may need to kill -9 {pid}.");
        }
    }
    Ok(())
}

pub(crate) fn cmd_hub_status() -> Result<()> {
    let pidfile_path = termlink_hub::pidfile::hub_pidfile_path();
    let socket_path = termlink_hub::server::hub_socket_path();

    match termlink_hub::pidfile::check(&pidfile_path) {
        termlink_hub::pidfile::PidfileStatus::NotRunning => {
            println!("Hub: not running");
        }
        termlink_hub::pidfile::PidfileStatus::Stale(pid) => {
            println!("Hub: stale (PID {pid} is dead, pidfile needs cleanup)");
            println!("  Run 'termlink hub stop' to clean up.");
        }
        termlink_hub::pidfile::PidfileStatus::Running(pid) => {
            println!("Hub: running (PID {pid})");
            println!("  Socket: {}", socket_path.display());
            println!("  Pidfile: {}", pidfile_path.display());
        }
    }
    Ok(())
}
