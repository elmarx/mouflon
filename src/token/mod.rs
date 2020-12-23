use crate::config::ClientConfig;
use std::path::Path;
mod receive_code;

pub async fn get_access_token<P: AsRef<Path>>(cache_directory: P, config: &ClientConfig) -> String {
    todo!()
}
