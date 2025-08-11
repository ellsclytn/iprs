use num_traits::PrimInt;
use rand::distr::uniform::SampleUniform;
use std::ops::Range;

pub trait RandomRangeGenerator<T> {
    fn random_range(&mut self, range: Range<T>) -> T;
}

pub struct DefaultRng;

impl<T> RandomRangeGenerator<T> for DefaultRng
where
    T: PrimInt + SampleUniform,
{
    fn random_range(&mut self, range: std::ops::Range<T>) -> T {
        rand::random_range(range)
    }
}
