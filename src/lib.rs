#![allow(unused)]

pub mod fetch;
pub mod logger;

/// Result type, equivalent to [`std::result::Result<T, MDLError>`]
///
/// Do not import this type - using [`mdl::Result<T>`] or [`crate::Result<T>`] is much more 
/// cleaner.
pub type Result<T> = std::result::Result<T, MDLError>;

use thiserror::Error;

/// Error type representing errors, which can occur when using the MDL library.
#[derive(Error, Debug)]
pub enum MDLError {
    #[error("unhandled Python exception")]
    Python(#[from] pyo3::PyErr),
    #[error("conversion from info dict failed")]
    Depythonize(#[from] pythonize::PythonizeError),
    #[error("parsing the URL failed")]
    UrlParse(#[from] url::ParseError),
    #[error("TOML serialization failed")]
    TOMLSer(#[from] toml::ser::Error),
    #[error("can't determine the URL")]
    GetUrlError(#[from] fetch::GetUrlError),
}
