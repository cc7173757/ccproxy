use crate::built_info;
use crate::config::CCProxyConfig;
use crate::error::CCProxyResult;
use clap::{Parser, Subcommand};

pub mod run;

#[derive(Debug, Parser)]
#[command(about = built_info::PKG_DESCRIPTION, long_about = None, version = built_info::PKG_VERSION)]
struct CCProxyCli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run the proxy server.
    Run,
}

pub async fn execute(config: CCProxyConfig) -> CCProxyResult<()> {
    let cli = CCProxyCli::parse();

    match &cli.cmd {
        Commands::Run => {
            run::run(config).await?;
        }
    };

    Ok(())
}
