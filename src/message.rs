pub enum StreamMessage {
    Connected,
    Frame(Vec<u8>),
}

impl std::fmt::Debug for StreamMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connected => write!(f, "Connected"),
            Self::Frame(v) => write!(f, "{}", format!("Frame[{}]", v.len())),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TcpMessage {
    Frame(Vec<u8>),
}
