mod service;

use miette::IntoDiagnostic;
use service::VoxputService;
use tracing_subscriber::EnvFilter;
use voxput_core::config::load_config;
use zbus::connection;

#[tokio::main]
async fn main() -> miette::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let config = load_config()?;
    let api_key = config.api_key()?;

    tracing::info!("Starting voxputd...");

    let service = VoxputService::new(api_key, config.model, config.device, None);
    let inner = service.inner_arc();

    let conn = connection::Builder::session()
        .into_diagnostic()?
        .name("com.github.jonochang.Voxput")
        .into_diagnostic()?
        .serve_at("/com/github/jonochang/Voxput", service)
        .into_diagnostic()?
        .build()
        .await
        .into_diagnostic()?;

    // Store connection so background pipeline tasks can emit signals.
    inner
        .connection
        .set(conn.clone())
        .expect("connection set only once");

    tracing::info!("voxputd ready — D-Bus name: com.github.jonochang.Voxput");

    // Run until the connection is closed or the process is killed.
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(u64::MAX)).await;
    }
}
