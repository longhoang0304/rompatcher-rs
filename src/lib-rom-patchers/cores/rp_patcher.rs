use crate::utils::unsigned_int::{UnsignedInt};

pub trait RPPatcher {
    fn patch<T: UnsignedInt>(rom: &[T], patch: &[T]) -> Vec<T>;
}