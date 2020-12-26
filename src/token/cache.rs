use crate::model::{AccessTokenResponse, AuthorizationData, BorrowedAuthorizationData};
use log::warn;
use std::fs::File;
use std::io::{BufReader, BufWriter, Result as IoResult};
use std::path::{Path, PathBuf};

fn cache_file_path<P: AsRef<Path>>(cache_dir: P, config_name: &str) -> PathBuf {
    let mut path = PathBuf::new();
    path.push(cache_dir);
    path.push(config_name);
    path.set_extension("json");

    path
}

pub(super) fn read_cached_auth<P: AsRef<Path>>(
    cache_dir: P,
    config_name: &str,
) -> Option<AuthorizationData> {
    // open the file, if it fails (because it does not exist), simply return none
    let file = File::open(cache_file_path(cache_dir, config_name)).ok()?;

    let reader = BufReader::new(file);
    // read/parse the file. If something fails, return none
    match serde_json::from_reader::<_, AuthorizationData>(reader) {
        Ok(auth_data) => Some(auth_data),
        Err(e) => {
            warn!("Error reading auth data for {}: {:?}", config_name, e);
            None
        }
    }
}

pub(super) fn write_cached_auth<P: AsRef<Path>>(
    cache_dir: P,
    config_name: &str,
    atr: &AccessTokenResponse,
) -> IoResult<()> {
    let path = cache_file_path(cache_dir, config_name);
    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    let atd = BorrowedAuthorizationData::new(atr);
    serde_json::to_writer_pretty(writer, &atd)?;

    Ok(())
}
