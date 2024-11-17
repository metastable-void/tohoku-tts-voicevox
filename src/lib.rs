
pub mod error;
mod vvc;
pub mod types;

pub mod deps {
    pub use serde_json;
    pub use serde;
}

pub use vvc::*;

pub use error::{
    ErrorDescription,
    GenericError,
};

#[non_exhaustive]
pub enum EngineErrorDescription {
    AlreadyInitialized,
    InitializationFailed,
    NotInitialized,
    InvalidParameter,
    SynthesisFailed,
    Unkown,
}

impl ErrorDescription for EngineErrorDescription {
    #[allow(refining_impl_trait)]
    fn description(&self) -> &'static str {
        match self {
            Self::AlreadyInitialized => "Engine is already initialized",
            _ => "Unknown error",
        }
    }
}

pub type EngineError = GenericError<EngineErrorDescription>;
