#[macro_use]
extern crate log;

mod client;
mod server;

pub use client::{
    DEFAULT_PORT,
    Counted,
    Timed,
    Gauged,
    ByteSink,
    StatsdClient
};

#[test]
fn it_works() {
}
