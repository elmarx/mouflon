use crate::config::ClientConfig;
use crate::token::cache::read_cached_auth;
use crate::token::valid::{is_at_valid, is_rt_valid};
use std::path::Path;

mod cache;
mod receive_code;
mod valid;

pub async fn get_access_token<P: AsRef<Path>>(cache_directory: P, config: &ClientConfig) -> String {
    let cached_auth = read_cached_auth(cache_directory, &config.name);
    match cached_auth {
        Some(auth) if is_at_valid(&auth) => auth.at_response.access_token,
        Some(auth) if is_rt_valid(&auth) => todo!("execute refresh_token flow"),
        // if there is a cached at, but neither the at, nor the at is valid, it's the same as no cached at at all
        _ => todo!("execute access_token flow"),
    }
}
