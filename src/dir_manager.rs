use std::env::current_exe;
use crate::logic::LOG_DIR;

/// Returns the path to the log directory.
///
/// # Panics
///
/// - If Getting the path to the executable fails
/// - If the executable does not have a parent directory.
pub fn get_log_dir() -> String {
    get_dir(LOG_DIR)
}

// TODO move config to dir
#[allow(dead_code)]
pub fn get_config_dir() -> String {
    get_dir("config")
}

#[cfg(target_os = "windows")]
fn get_dir(to_add: &str) -> String {
    let mut path = current_exe()
        .expect("Failed to get current executable")
        .parent()
        .unwrap()
        .to_path_buf();
    path.push(to_add);
    let path_string = path.to_str().unwrap();
    format!("{path_string}\\")
}

#[cfg(target_os = "unix")]
fn get_dir(to_add: &str) -> String {
    format!("{to_add}/")
}
