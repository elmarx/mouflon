use crate::config::keycloak::KeycloakClientConfig;
use crate::config::ClientConfig;
use log::{debug, error};
use std::ffi::OsStr;
use std::fs::{read_dir, DirEntry, File};
use std::io::{BufReader, Result as IoResult};
use std::path::Path;

pub fn read_configs<P: AsRef<Path>>(path: P) -> IoResult<Vec<ClientConfig>> {
    Ok(read_dir(path)?
        .filter_map(|e| {
            let config = read_config_file(e);
            if let Ok(config) = config {
                config
            } else {
                error!(
                    "Error reading config file: {:?}. Ignoring this file",
                    config
                );
                None
            }
        })
        .collect())
}

fn read_config_file(e: IoResult<DirEntry>) -> IoResult<Option<ClientConfig>> {
    let e = e?.path();
    let file_stem = e
        .file_stem()
        .and_then(|n| n.to_str().map(ToOwned::to_owned));

    match (e.extension(), file_stem) {
        (Some(extension), Some(config_name)) if extension == OsStr::new("json") => {
            let file = File::open(e)?;
            let reader = BufReader::new(file);

            // TODO: detect and support other config formats
            let kc_config = serde_json::from_reader::<_, KeycloakClientConfig>(reader);
            let kc_config = kc_config.expect("invalid keycloak OIDC config file");

            Ok(Some(kc_config.into_client_config(config_name)))
        }
        (Some(extension), Some(file_stem)) => {
            debug!(
                "Ignoring file {:?}: unknown extension {:?}",
                file_stem, extension
            );
            Ok(None)
        }
        _ => {
            error!(
                "cannot read filename/extension for {:?}, ignoring this file",
                e
            );
            Ok(None)
        }
    }
}
