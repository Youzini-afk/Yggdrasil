use std::net::SocketAddr;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use serde_json::json;
use ygg_core::KERNEL_PACKAGE_ID;
use ygg_runtime::{AppendEventRequest, EventStore, InMemoryEventStore, OpenSessionRequest, Runtime, RuntimeConfig};

#[derive(Debug, Parser)]
#[command(name = "ygg")]
#[command(about = "Yggdrasil kernel CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run a content-free kernel event demo.
    Demo,
    /// Run the headless kernel HTTP service.
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
        Command::Demo => demo().await,
        Command::Serve { bind } => serve(bind).await,
    }
}

async fn demo() -> anyhow::Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store.clone(), RuntimeConfig::default());

    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime
        .append_event(AppendEventRequest {
            session_id: session.id.clone(),
            writer_package_id: "example/echo".to_string(),
            kind: "example/echo/event.demo".to_string(),
            payload: json!({"message": "content-free kernel event"}),
            metadata: json!({"created_by": "ygg-cli demo"}),
        })
        .await?;

    let events = store.list_session(&session.id).await?;

    println!("session_id: {}", session.id);
    println!("kernel_package_id: {KERNEL_PACKAGE_ID}");
    println!("\nevents:");
    for event in events {
        println!("- #{} {} {}", event.sequence, event.writer_package_id, event.kind);
    }

    Ok(())
}

async fn serve(bind: SocketAddr) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(bind).await?;
    println!("Yggdrasil kernel service listening on http://{bind}");
    axum::serve(listener, ygg_service::app()).await?;
    Ok(())
}
