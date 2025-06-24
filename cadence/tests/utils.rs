#![allow(unused)]

use cadence::prelude::*;
use cadence::StatsdClient;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn run_arc_threaded_test(
    client: StatsdClient,
    num_threads: u64,
    iterations: u64,
    iteration_interval: Option<Duration>,
) {
    let shared_client = Arc::new(client);

    let threads: Vec<_> = (0..num_threads)
        .map(|_| {
            let local_client = shared_client.clone();

            thread::spawn(move || {
                for i in 0..iterations {
                    local_client.count("some.counter", i as i64).unwrap();
                    local_client.time("some.timer", i).unwrap();
                    local_client.time("some.timer", Duration::from_millis(i)).unwrap();
                    local_client.gauge("some.gauge", i).unwrap();
                    local_client.gauge("some.gauge", i as f64).unwrap();
                    local_client.meter("some.meter", i).unwrap();
                    local_client.histogram("some.histogram", i).unwrap();
                    local_client
                        .histogram("some.histogram", Duration::from_nanos(i))
                        .unwrap();
                    local_client.histogram("some.histogram", i as f64).unwrap();
                    local_client.distribution("some.distribution", i).unwrap();
                    local_client.distribution("some.distribution", i as f64).unwrap();
                    local_client.set("some.set", i as i64).unwrap();
                    thread::sleep(iteration_interval.unwrap_or(Duration::from_millis(1)));
                }
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }
}

#[derive(Debug, Default)]
pub struct InstrumentedAllocator {
    num_allocs: AtomicUsize,
    num_bytes: AtomicUsize,
    enabled: AtomicBool,
}

impl InstrumentedAllocator {
    pub const fn new() -> Self {
        InstrumentedAllocator {
            num_allocs: AtomicUsize::new(0),
            num_bytes: AtomicUsize::new(0),
            enabled: AtomicBool::new(false),
        }
    }

    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Release);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Release);
    }

    pub fn num_allocs(&self) -> usize {
        self.num_allocs.load(Ordering::Acquire)
    }

    pub fn num_bytes(&self) -> usize {
        self.num_bytes.load(Ordering::Acquire)
    }
}

unsafe impl GlobalAlloc for InstrumentedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let res = System.alloc(layout);

        if !res.is_null() && self.is_enabled() {
            self.num_bytes.fetch_add(layout.size(), Ordering::Release);
            self.num_allocs.fetch_add(1, Ordering::Release);
        }

        res
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}
