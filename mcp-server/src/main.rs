mod audit;
mod auth;
mod catalog;
mod config;
mod errors;
mod helm;
pub mod k8s;
mod mcp;
mod oci_client;
mod policy;
mod prompts;
mod resources;
mod server;
mod session;
mod skills;
mod tools;
pub mod vault;

use crate::config::Config;
use crate::server::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new(
            config.log_level.clone(),
        )?)
        .init();

    run(config).await
}
