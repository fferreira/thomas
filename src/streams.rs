
// A trait for Streams
pub trait Stream {
    type Item: PartialEq + std::fmt::Debug;
    //FIXME: Debug is only needed for the tests, if I write them
    fn next(&self) -> Option<(&Self, &Self::Item)>;
    fn peek(&self) -> Option<&Self::Item>;
    fn drop(&self) -> Option<&Self>;
}