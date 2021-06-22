#[derive(Debug)]
pub enum KVRInitializationError {
    MagicDeserializationFailed(Box<bincode::ErrorKind>),
    MagicVerificationFailed,
    FailedToOpenFile(std::io::Error),
    FailedToReadFile(std::io::Error),
    FailedToDeserializeValue(Box<bincode::ErrorKind>),
}

pub enum KVRInsertionError {
    KeyExists,
    SerializationError(Box<bincode::ErrorKind>),
    IOError(std::io::Error),
}

pub enum KVRUpdateError {
    KeyDoesNotExist,
    PrevRevMismatch,
    SerializationError(Box<bincode::ErrorKind>),
    IOError(std::io::Error),
}
