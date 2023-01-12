#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("encoding error: {0}")]
    BinEnc(bincode::error::EncodeError),

    #[error("numerical error: {0}")]
    Num(#[from] std::num::TryFromIntError),

    #[error("serial error: {0}")]
    Serial(#[from] tokio_serial::Error),

    #[error("channel send error: {0}")]
    Send(#[from] async_channel::SendError<pactlers_lib::Cmd>),

    #[error("channel recv error: {0}")]
    Recv(#[from] async_channel::RecvError),
}
