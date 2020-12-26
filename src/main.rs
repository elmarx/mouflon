use crate::config::read_configs;
use crate::token::get_access_token;
use directories_next::ProjectDirs;
use std::env;
use std::fs::create_dir_all;

mod config;
mod model;
mod token;

pub(crate) type BoxResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
pub async fn main() -> BoxResult<()> {
    env_logger::init();

    let project_dirs = ProjectDirs::from("org", "Athmer", "Mouflon")
        .expect("could not determine user's home directory");
    let cache_dir = project_dirs.cache_dir();
    let config_dir = project_dirs.config_dir();
    create_dir_all(cache_dir).expect("could not create cache directory");
    create_dir_all(config_dir).expect("could not create config directory");

    let configs = read_configs(config_dir)?;

    let config = env::args()
        .nth(2)
        .unwrap_or_else(|| String::from("default"));

    let config = configs
        .iter()
        .find(|c| *c.name == config)
        .expect("did not find config");

    let at = get_access_token(cache_dir, config).await?;

    println!("{}", at);

    Ok(())
}
