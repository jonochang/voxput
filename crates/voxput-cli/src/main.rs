mod cli;

use clap::Parser;

#[tokio::main]
async fn main() -> miette::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = cli::Cli::parse();
    cli::dispatch(cli)
        .await
        .map_err(|e| miette::miette!("{e}"))
}
