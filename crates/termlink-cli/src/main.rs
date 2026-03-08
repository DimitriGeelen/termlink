use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "termlink=info".into()),
        )
        .init();

    tracing::info!("TermLink v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!(
        "Runtime dir: {}",
        termlink_session::discovery::runtime_dir().display()
    );

    // CLI command dispatch will be implemented in subsequent tasks.
    println!("termlink — cross-terminal session communication");
    println!("Use --help for usage information.");

    Ok(())
}
