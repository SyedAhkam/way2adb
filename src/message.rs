pub enum StreamMessage {
    Ready,
    Frame { count: usize, data: Vec<u8> },
}

impl std::fmt::Debug for StreamMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ready => write!(f, "Ready"),
            Self::Frame { count, data } => {
                write!(f, "{}", format!("Frame[{}]({})", data.len(), count))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum TcpMessage {
    Frame(Vec<u8>),
}
