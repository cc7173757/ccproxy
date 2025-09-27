use ccproxy::cli;
use ccproxy::config::CCProxyConfig;
use ccproxy::error::CCProxyResult;

#[tokio::main]
async fn main() -> CCProxyResult<()> {
    // Init config.
    let config = init()?;

    // Init tracing subscriber.
    let (subscriber, _guard) = config.log.tracing_subscriber()?;
    tracing::subscriber::set_global_default(subscriber).expect("Failed to init tracing subscriber");

    #[cfg(debug_assertions)]
    rust_raknet::enable_raknet_log(7);

    if let Err(err) = cli::execute(config).await {
        tracing::error!("{}", err);
    };

    Ok(())
}

/// Set environment variables from .env file and load the config.
pub fn init() -> CCProxyResult<CCProxyConfig> {
    // Get from .env file.
    dotenvy::dotenv().ok();

    // Load config from environment variables.
    CCProxyConfig::init()
}
