#[derive(Debug)]
pub enum EncodeError {
    Bincode(bincode::error::EncodeError),
    Flate2(std::io::Error),
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            EncodeError::Bincode(ref err) => write!(f, "Bincode error: {}", err),
            EncodeError::Flate2(ref err) => write!(f, "Flate2 error: {}", err),
        }
    }
}

impl From<bincode::error::EncodeError> for EncodeError {
    fn from(error: bincode::error::EncodeError) -> Self {
        EncodeError::Bincode(error)
    }
}

impl From<std::io::Error> for EncodeError {
    fn from(error: std::io::Error) -> Self {
        EncodeError::Flate2(error)
    }
}

#[derive(Debug)]
pub enum DecodeError {
    Bincode(bincode::error::DecodeError),
    Flate2(std::io::Error),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            DecodeError::Bincode(ref err) => write!(f, "Bincode error: {}", err),
            DecodeError::Flate2(ref err) => write!(f, "Flate2 error: {}", err),
        }
    }
}

impl From<bincode::error::DecodeError> for DecodeError {
    fn from(error: bincode::error::DecodeError) -> Self {
        DecodeError::Bincode(error)
    }
}

impl From<std::io::Error> for DecodeError {
    fn from(error: std::io::Error) -> Self {
        DecodeError::Flate2(error)
    }
}
