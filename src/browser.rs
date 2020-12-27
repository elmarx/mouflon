use crate::BoxResult;
use log::{info, warn};
use std::process::Command;

#[cfg(target_os = "linux")]
const BROWSER_CMD: &str = "xdg-open";
#[cfg(target_os = "windows")]
const BROWSER_CMD: &str = "explorer";
#[cfg(target_os = "macos")]
const BROWSER_CMD: &str = "open";

pub fn open_url_in_browser(url: &str) -> BoxResult<()> {
    match Command::new(BROWSER_CMD).arg(url).spawn() {
        Ok(_) => Ok(()),
        Err(e) => {
            warn!("failed to browser: {:?}", e);
            info!("Please open {}", url);

            Ok(())
        }
    }
}
