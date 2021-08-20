use cadence::prelude::*;
use cadence::{NopMetricSink, QueuingMetricSink, StatsdClient};
use utils::InstrumentedAllocator;

mod utils;

#[global_allocator]
static GLOBAL: InstrumentedAllocator = InstrumentedAllocator::new();

#[test]
fn test_allocs_statsdclient_nop_queuing_no_tags() {
    let client = StatsdClient::from_sink("alloc.test", QueuingMetricSink::from(NopMetricSink));

    // one initial metric while we're not recording to make sure any one-time costs don't
    // count towards what we measure (seems like the channels used by the queuing sink do
    // some lazy setup that results in about 1kb of allocation).
    client.incr("foo").unwrap();

    GLOBAL.enable();
    client.incr("bar").unwrap();
    GLOBAL.disable();

    let num_allocs = GLOBAL.num_allocs();
    assert_eq!(2, num_allocs)
}
