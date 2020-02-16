pub struct Index<T> {
    prefix: Vec<u8>,
    action: Box<dyn Fn(T) -> Vec<u8>>,
}

