use anyhow::Result;
use cli::Cli;

pub mod cli;
pub mod config;

#[tokio::main]
async fn main() -> Result<()> {
    Cli::run().await
}
