mod client;
mod server;

pub use client::{
    DEFAULT_PORT,
    Counted,
    Timed,
    Gauged,
    StatsdClientUdp
};

#[test]
fn it_works() {
}
