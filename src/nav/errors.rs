use thiserror::Error;


#[allow(unused)]
#[derive(Error, Debug)]
pub enum NavigatorError {
    #[error("ClingoError: ")]
    Clingo(#[from] clingo::ClingoError),
    #[error("Unwrapped None.")]
    None,
    #[error("Unwrapped no control object.")]
    NoControl,
    #[error("IOError: ")]
    IOError(#[from] std::io::Error),
    #[error("Invalid input.")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, NavigatorError>;

