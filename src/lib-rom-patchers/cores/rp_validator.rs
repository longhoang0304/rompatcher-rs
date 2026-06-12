use crate::utils::unsigned_int::{UnsignedInt};

pub trait RPValidator {
    fn validate<T: UnsignedInt>(patch: &[T]) -> Vec<T>;
}