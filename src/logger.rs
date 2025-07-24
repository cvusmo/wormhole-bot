// ~/src/logger.rs

use log::{error, info};

pub fn init_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();
}

pub fn log_info(msg: &str) {
    info!("{}", msg);
}

pub fn log_error(msg: &str) {
    error!("{}", msg);
}
