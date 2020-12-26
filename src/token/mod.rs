use crate::config::ClientConfig;
use crate::token::cache::{read_cached_auth, write_cached_auth};
use crate::token::oauth2::Oauth2Client;
use crate::token::valid::{is_at_valid, is_rt_valid};
use crate::BoxResult;
use std::path::Path;

mod cache;
mod oauth2;
mod receive_code;
mod valid;

pub async fn get_access_token<P: AsRef<Path>>(
    cache_directory: P,
    config: &ClientConfig,
) -> BoxResult<String> {
    let cached_auth = read_cached_auth(&cache_directory, &config.name);
    match cached_auth {
        // if there is a valid at, just return it
        Some(auth) if is_at_valid(&auth) => Ok(auth.at_response.access_token),
        Some(auth) if is_rt_valid(&auth) => {
            let client = Oauth2Client::from(config).await;
            let atr = client
                .refresh_token(auth.at_response.refresh_token.as_str())
                .await?;

            write_cached_auth(cache_directory, &*config.name, &atr).expect("writing cache failed");
            Ok(atr.access_token)
        }
        // if there is a cached at, but neither the at, nor the at is valid, it's the same as no cached at at all
        _ => {
            let client = Oauth2Client::from(config).await;
            let atr = client.access_token().await?;

            write_cached_auth(cache_directory, &*config.name, &atr)?;
            Ok(atr.access_token)
        }
    }
}
