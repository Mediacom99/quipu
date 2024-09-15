//Log utilities for tracing and tracing-sub
pub mod log;
pub mod message;
//Common use statements within this project modules
pub mod prelude {
    pub use std::error::Error;
    pub use tracing::{debug, error, info, trace, warn};
}

use crate::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log::tracing_subscriber_setup("trace").await;
    info!("quipu is starting...");

    Ok(())
}
