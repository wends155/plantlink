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
    let runtime: std::sync::Arc<tokio::sync::RwLock<dyn plantlink_runtime::FlowRuntime>> =
        std::sync::Arc::new(tokio::sync::RwLock::new(
            plantlink_runtime::RuntimeEngine::new(tx.clone())?,
        ));

    // Spawn Web Server
    WebServer::run(args.port, tx, runtime).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_port_is_3000() {
        let args = Args::try_parse_from(["plantlink-cli"]).unwrap();
        assert_eq!(args.port, 3000);
    }

    #[test]
    fn test_port_flag_parsing() {
        let args = Args::try_parse_from(["plantlink-cli", "--port", "8080"]).unwrap();
        assert_eq!(args.port, 8080);
    }

    #[test]
    fn test_port_short_flag_parsing() {
        let args = Args::try_parse_from(["plantlink-cli", "-p", "9090"]).unwrap();
        assert_eq!(args.port, 9090);
    }
}
