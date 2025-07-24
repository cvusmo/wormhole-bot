// ~/src/lib.rs

pub mod feed;
pub mod feed_config;
pub mod handler;
pub mod logger;

pub use feed::{feed_fetch, feed_format, feed_interval};
pub use feed_config::load_feeds;
pub use handler::Handler;
pub use logger::{init_logger, log_error, log_info};
