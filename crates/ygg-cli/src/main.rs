use std::net::SocketAddr;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use ygg_runtime::{EventStore, InMemoryEventStore, MockModelProvider, Runtime, RuntimeConfig};

#[derive(Debug, Parser)]
#[command(name = "ygg")]
#[command(about = "Yggdrasil runtime CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run an in-memory runtime demo turn.
    Demo {
        /// Input text for the mock runtime.
        #[arg(default_value = "Hello from Yggdrasil")]
        input: String,
    },
    /// Run the headless HTTP service.
    Serve {
        #[arg(long, default_value = "127.0.0.1:8787")]
        bind: SocketAddr,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Command::Demo { input } => demo(input).await,
        Command::Serve { bind } => serve(bind).await,
    }
}

async fn demo(input: String) -> anyhow::Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let model = Arc::new(MockModelProvider::default());
    let runtime = Runtime::new(store.clone(), model, RuntimeConfig::default());

    let session = runtime.create_session(Some("CLI Demo".to_string())).await?;
    let output = runtime.input(session.id.clone(), input).await?;
    let events = store.list_session(&session.id).await?;

    println!("session_id: {}", session.id);
    println!("turn_id: {}", output.turn_id);
    println!("prompt_frame_id: {}", output.prompt_frame.id);
    println!("output: {}", output.output);
    println!("\nevents:");
    for event in events {
        println!("- {:?} {}", event.kind, event.id);
    }
    println!("\nprompt_frame:");
    println!("{}", serde_json::to_string_pretty(&output.prompt_frame)?);

    Ok(())
}

async fn serve(bind: SocketAddr) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(bind).await?;
    println!("Yggdrasil service listening on http://{bind}");
    axum::serve(listener, ygg_service::app()).await?;
    Ok(())
}
