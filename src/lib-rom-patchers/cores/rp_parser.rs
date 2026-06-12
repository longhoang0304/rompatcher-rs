use crate::utils::unsigned_int::{UnsignedInt};

pub trait RPParser {
    fn parse<T: UnsignedInt>(patch: &[T]) -> Vec<T>;
}