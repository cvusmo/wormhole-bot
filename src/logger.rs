// ~/src/logger.rs

use log::{error, info};

pub fn init_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .filter_module("serenity", log::LevelFilter::Off)
        .filter_module("tracing::span", log::LevelFilter::Off)
        //.filter_module("wormhole_rss", log::LevelFilter::Error)
        .format_timestamp_secs()
        .init();
}

pub fn log_info(msg: &str) {
    info!("{msg}");
}

pub fn log_error(msg: &str) {
    error!("{msg}");
}
