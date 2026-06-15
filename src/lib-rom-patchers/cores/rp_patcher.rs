pub trait RPPatcher<P> {
    fn patch(rom: &[u8], patch: &[P]) -> Vec<u8>;
}