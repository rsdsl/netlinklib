use std::{ffi, io};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("link {0} not found")]
    LinkNotFound(String),

    #[error("link name contains nul bytes: {0}")]
    Nul(#[from] ffi::NulError),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("rtnetlink error: {0}")]
    RtNetlink(#[from] rtnetlink::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
