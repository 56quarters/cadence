use cadence::{SpyMetricSink, StatsdClient};
use cadence_macros::{statsd_count, statsd_gauge, statsd_histogram, statsd_meter, statsd_set, statsd_time};
use std::io;
use std::sync::{Arc, Mutex, Once};

/// Underlying writer to be used by a `SpyMetricSink`
static mut WRITER: Option<Arc<Mutex<TestWriter>>> = None;

/// Control initialization of the underlying writer
static WRITER_INIT: Once = Once::new();

/// `Write` implementation to use with a SpyMetricSink for testing
///
/// This write implementation converts all incoming writes to strings
/// (assuming utf-8 encoding) and stores them in a vector that grows
/// without bound.
struct TestWriter {
    saved: Vec<String>,
}

impl TestWriter {
    fn new() -> Self {
        TestWriter { saved: Vec::new() }
    }

    /// Get the underlying vector used for storage
    fn storage(&self) -> &Vec<String> {
        &self.saved
    }
}

impl io::Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // test code, this should never be invalid utf-8
        let s = String::from_utf8(buf.to_vec()).unwrap();
        self.saved.push(s);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Set a default client and save a reference to the underlying writer
fn init_default_client() {
    WRITER_INIT.call_once(|| {
        // Save a reference to the underlying writer used by the SpyMetricSink so
        // that we can inspect its contents after every test. All metrics are saved
        // so are able to assert that it contains the metric we just wrote.
        unsafe {
            WRITER = Some(Arc::new(Mutex::new(TestWriter::new())));
        }

        // Set the global default to be a client that writes to a SpyMetricSink so
        // we can verify the metrics being written are what we expect. It would be
        // safe to do this outside of the `call_once` block (since set_global_default
        // will only set the client a single time) but we might as well avoid extra
        // work if we can.
        let sink = SpyMetricSink::from(unsafe { &WRITER }.clone().unwrap());
        cadence_macros::set_global_default(StatsdClient::from_sink("my.prefix", sink));
    });
}

/// Method to get the underlying vector of strings written to the sink.
///
/// This exists so that the lock for the writer is dropped before any assertions
/// are made that might panic (and hence poison the lock).
fn get_default_storage() -> Vec<String> {
    let writer = unsafe { WRITER.clone() }.unwrap();
    let inner = writer.lock().unwrap();
    inner.storage().clone()
}

#[test]
fn test_statsd_count() {
    init_default_client();
    statsd_count!("some.counter", 123);
    statsd_count!("some.counter", 123, "host" => "web01.example.com", "slice" => "a");

    let storage = get_default_storage();
    assert!(storage.contains(&"my.prefix.some.counter:123|c".to_owned()));
    assert!(storage.contains(&"my.prefix.some.counter:123|c|#host:web01.example.com,slice:a".to_owned()));
}

#[test]
fn test_statsd_time() {
    init_default_client();
    statsd_time!("some.timer", 334);
    statsd_time!("some.timer", 334, "type" => "api", "status" => "200");

    let storage = get_default_storage();
    assert!(storage.contains(&"my.prefix.some.timer:334|ms".to_owned()));
    assert!(storage.contains(&"my.prefix.some.timer:334|ms|#type:api,status:200".to_owned()));
}

#[test]
fn test_statsd_gauge() {
    init_default_client();
    statsd_gauge!("some.gauge", 42);
    statsd_gauge!("some.gauge", 42, "org" => "123", "service" => "gateway");

    let storage = get_default_storage();
    assert!(storage.contains(&"my.prefix.some.gauge:42|g".to_owned()));
    assert!(storage.contains(&"my.prefix.some.gauge:42|g|#org:123,service:gateway".to_owned()));
}

#[test]
fn test_statsd_meter() {
    init_default_client();
    statsd_meter!("some.meter", 1);
    statsd_meter!("some.meter", 1, "foo" => "bar", "result" => "reject");

    let storage = get_default_storage();
    assert!(storage.contains(&"my.prefix.some.meter:1|m".to_owned()));
    assert!(storage.contains(&"my.prefix.some.meter:1|m|#foo:bar,result:reject".to_owned()));
}

#[test]
fn test_statsd_histogram() {
    init_default_client();
    statsd_histogram!("some.histogram", 223);
    statsd_histogram!("some.histogram", 223, "method" => "auth", "result" => "error");

    let storage = get_default_storage();
    assert!(storage.contains(&"my.prefix.some.histogram:223|h".to_owned()));
    assert!(storage.contains(&"my.prefix.some.histogram:223|h|#method:auth,result:error".to_owned()));
}

#[test]
fn test_statsd_set() {
    init_default_client();
    statsd_set!("some.set", 348);
    statsd_set!("some.set", 348, "service" => "user", "host" => "app01.example.com");

    let storage = get_default_storage();
    assert!(storage.contains(&"my.prefix.some.set:348|s".to_owned()));
    assert!(storage.contains(&"my.prefix.some.set:348|s|#service:user,host:app01.example.com".to_owned()));
}
