use crate::config::get_configs;
use crate::token::get_access_token;
use std::env;
use std::path::PathBuf;

mod config;
mod token;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cache_dir: PathBuf = todo!();

    let configs = get_configs();

    let config = env::args()
        .nth(2)
        .unwrap_or_else(|| String::from("default"));

    let config = configs
        .iter()
        .find(|c| *c.name == config)
        .expect("did not find config");

    let at = get_access_token(cache_dir, config).await;

    println!("{}", at);

    Ok(())
}
