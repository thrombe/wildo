#![allow(dead_code)]
// #![allow(unused_variables)]
#![allow(unused_imports)]

mod app;
mod content;
mod display;
mod register;
mod run;
mod service;
mod stack;
mod traits;

pub use service::log::{
    // dbg macro is defined at the root of the crate automatically for some reason
    debug,
    error,
};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    run::run().await
}
