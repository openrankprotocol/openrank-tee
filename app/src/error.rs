use alloy::{
    hex::FromHexError, signers::local::LocalSignerError, transports::RpcError,
    transports::TransportError, transports::TransportErrorKind,
};
use aws_sdk_s3::{primitives::ByteStreamError, Error as AwsError};
use csv::Error as CsvError;
use openrank_common::eigenda::EigenDAError;
use openrank_common::runner::Error as ComputeRunnerError;
use serde_json::Error as SerdeError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("LocalSignerError: {0}")]
    LocalSignerError(LocalSignerError),
    #[error("TransportError: {0}")]
    TransportError(TransportError),
    #[error("RpcError: {0}")]
    RpcError(String),
    #[error("Hex error: {0}")]
    HexError(FromHexError),
    #[error("Serde error: {0}")]
    SerdeError(SerdeError),
    #[error("Aws error: {0}")]
    AwsError(AwsError),
    #[error("File error: {0}")]
    FileError(String),
    #[error("Csv error: {0}")]
    CsvError(CsvError),
    #[error("ComputeRunnerError: {0}")]
    ComputeRunnerError(ComputeRunnerError),
    #[error("Tx Error: {0}")]
    TxError(String),
    #[error("ByteStreamError: {0}")]
    ByteStreamError(ByteStreamError),
    #[error("EigenDA error: {0}")]
    EigenDAError(EigenDAError),
}

impl From<EigenDAError> for Error {
    fn from(err: EigenDAError) -> Self {
        Error::EigenDAError(err)
    }
}

impl From<RpcError<TransportErrorKind>> for Error {
    fn from(err: RpcError<TransportErrorKind>) -> Self {
        Error::RpcError(format!("{}", err))
    }
}
