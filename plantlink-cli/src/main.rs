use clap::Parser;
use plantlink_web::WebServer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let version = env!("CARGO_PKG_VERSION");
    tracing::info!("Starting PlantLink v{} on port {}", version, args.port);

    // Create a broadcast channel for events (capacity 100)
    let (tx, _rx) = tokio::sync::broadcast::channel(100);

    // Create the Runtime Engine
    let runtime = std::sync::Arc::new(tokio::sync::RwLock::new(
        plantlink_runtime::RuntimeEngine::new(tx.clone()),
    ));

    // Spawn Web Server
    WebServer::run(args.port, tx, runtime).await?;

    Ok(())
}
