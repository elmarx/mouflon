use crate::model::AuthorizationData;
use log::warn;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub(super) fn read_cached_auth<P: AsRef<Path>>(
    cache_dir: P,
    config_name: &str,
) -> Option<AuthorizationData> {
    let mut path = PathBuf::new();
    path.push(cache_dir);
    path.push(config_name);
    path.set_extension("json");

    // open the file, if it fails (because it does not exist), simply return none
    let file = File::open(path).ok()?;

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
