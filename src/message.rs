#[derive(Debug)]
pub enum StreamMessage {
    Connected,
    Frame(Vec<u8>),
}

#[derive(Debug, Clone)]
pub enum TcpMessage {
    Connected,
    Frame(Vec<u8>),
}
