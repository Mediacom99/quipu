//! Utility wrapper functions around `tracing` and `tracing-subscriber` crates.
//! TODO: Implement this as a struct-object and re-export macros, then re-export
//! this module into a prelude in crate root.
use tracing::*;
use tracing_subscriber::filter::EnvFilter;

/// Fast tracing subscriber setup with RUST_LOG env variable
/// filter.
pub async fn tracing_subscriber_setup(log_level: &str) {
    //TODO check if user already set it
    std::env::set_var("RUST_LOG", log_level);

    //Construct a format for a subscriber
    let format = tracing_subscriber::fmt::format()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .compact();

    //control with default env RUST_LOG
    //like RUST_LOG=debug cargo run
    let filter = EnvFilter::from_default_env();

    tracing_subscriber::fmt()
        .event_format(format)
        .with_env_filter(filter)
        .init();

    debug!("Tracing subscriber successfully initialized")
}

///Stupid function to test different tracing event macros
#[instrument(level = "trace")] //add tracing span to this function
pub async fn try_event_macros() {
    println!("Do smth");
    trace!("This is a trace");
    debug!("This is a debug");
    info!("This is an info");
    warn!("This is a warning");
    error!("This is an error")
}
