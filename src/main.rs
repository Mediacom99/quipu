//Log utilities for tracing and tracing-sub
mod logutil;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    logutil::tracing_subscriber_setup("trace").await;


    Ok(())
}

