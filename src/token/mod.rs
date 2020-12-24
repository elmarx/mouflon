use crate::config::ClientConfig;
use crate::model::AuthorizationData;
use std::path::Path;

mod receive_code;
mod valid;

pub async fn get_access_token<P: AsRef<Path>>(cache_directory: P, config: &ClientConfig) -> String {
    todo!()
}
