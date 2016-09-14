extern crate num;

#[macro_use]
extern crate static_fir;

use static_fir::{FIRCoefs, FIRFilter};

pub trait DecimationFactor {
    fn factor() -> u32;
}

pub struct Decimator<D: DecimationFactor, C: FIRCoefs> {
    factor: std::marker::PhantomData<D>,
    filter: FIRFilter<C>,
    idx: u32,
}

impl<D: DecimationFactor, C: FIRCoefs> Decimator<D, C> {
    pub fn new() -> Decimator<D, C> {
        Decimator {
            factor: std::marker::PhantomData,
            filter: FIRFilter::new(),
            idx: 0,
        }
    }

    pub fn feed(&mut self, sample: C::Sample) -> Option<C::Sample> {
        let out = self.filter.feed(sample);

        self.idx += 1;
        self.idx %= D::factor();

        if self.idx != 0 {
            None
        } else {
            Some(out)
        }
    }

    pub fn decim_in_place(&mut self, samples: &mut [C::Sample]) -> usize {
        let mut src = 0;
        let mut dest = 0;

        loop {
            let sample = match samples.get(src) {
                Some(&s) => s,
                None => break,
            };

            if let Some(out) = self.feed(sample) {
                samples[dest] = out;
                dest += 1;
            }

            src += 1;
        }

        dest
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl_fir!(TestFIR, f32, 6, [
        1.0, 0.0, 2.0, 3.0, 0.0, 1.0,
    ]);

    pub struct TestDecim;

    impl DecimationFactor for TestDecim {
        fn factor() -> u32 { 4 }
    }

    #[test]
    fn test_decim() {
        let mut d = Decimator::<TestDecim, TestFIR>::new();

        assert_eq!(d.feed(1.0), None);
        assert_eq!(d.feed(1.0), None);
        assert_eq!(d.feed(1.0), None);
        assert_eq!(d.feed(1.0), Some(6.0));
        assert_eq!(d.feed(2.0), None);
        assert_eq!(d.feed(2.0), None);
        assert_eq!(d.feed(2.0), None);
        assert_eq!(d.feed(2.0), Some(13.0));
    }

    #[test]
    fn test_in_place() {
        let mut d = Decimator::<TestDecim, TestFIR>::new();
        let mut samples = [1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 2.0];

        d.decim_in_place(&mut samples[..]);

        assert_eq!(samples[0], 6.0);
        assert_eq!(samples[1], 13.0);
        assert_eq!(samples[2], 1.0);
    }
}
