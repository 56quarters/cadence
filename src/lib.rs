mod client;
mod server;

pub use client::{
    DEFAULT_PORT,
    Counter,
    Timer,
    Gauge,
    Counted,
    Timed,
    Gauged,
    StatsdClientUdp
};

#[test]
fn it_works() {
}
