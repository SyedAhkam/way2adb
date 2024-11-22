pub enum StreamMessage {
    Ready,
    Header(Vec<u8>),
    Frame { count: u64, data: Vec<u8> },
}

impl std::fmt::Debug for StreamMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ready => write!(f, "Ready"),
            Self::Frame { count, data } => {
                write!(f, "{}", format!("Frame[{}]({})", data.len(), count))
            }
            Self::Header(v) => write!(f, "{}", format!("Header[{}]", v.len())),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TcpMessage {
    Frame(Vec<u8>),
}
