use std::process::Command;

use log::{debug, error, info};

pub fn issue_bg_update(path: String, swww_path: String, resize_mode: String) {
    let output = Command::new(swww_path)
        .args(["img", path.as_str(), "--resize", resize_mode.as_str()])
        .output()
        .expect("Failed to execute command");
    info!("swww command status: {}", output.status);
    debug!(
        "sww cmd output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    if !&output.stderr.is_empty() {
        error!(
            "Error executing swww cmd: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
