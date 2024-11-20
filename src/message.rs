#[derive(Debug)]
pub enum StreamMessage {
    Connected,
    Frame(Vec<u8>),
}
