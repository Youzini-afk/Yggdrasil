use clap::Parser;
use ygg_cli::cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    ygg_cli::run_cli(cli).await
}
