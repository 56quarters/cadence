#[cfg(feature = "sample-rate")]
pub(crate) use rng::Sampler;

#[cfg(not(feature = "sample-rate"))]
pub(crate) use noop::Sampler;

pub(crate) trait Sampling {
    fn new_with_rate(rate: f32) -> Self;
    fn sample<T>(&self, item: T) -> Option<T>;
}

mod noop {
    use super::Sampling;

    pub struct Sampler;

    impl Sampling for Sampler {
        fn new_with_rate(_rate: f32) -> Self {
            Self
        }

        fn sample<T>(&self, item: T) -> Option<T> {
            Some(item)
        }
    }
}

#[cfg(feature = "sample-rate")]
pub mod rng {
    use super::Sampling;
    use rand::Rng;

    pub struct Sampler(f32);

    impl Sampling for Sampler {
        fn new_with_rate(rate: f32) -> Self {
            Self(rate)
        }

        fn sample<T>(&self, item: T) -> Option<T> {
            let mut rng = rand::thread_rng();

            if rng.gen_bool(self.0.into()) {
                Some(item)
            } else {
                None
            }
        }
    }
}
