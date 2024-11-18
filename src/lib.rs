
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

pub struct TextSplitter {
    sentence_splitter: Vec<String>,
}

impl Default for TextSplitter {
    fn default() -> Self {
        Self {
            sentence_splitter: vec!["。".to_string(), "？".to_string(), "！".to_string(), "!".to_string(), "?".to_string(), "\n".to_string()],
        }
    }
}

impl TextSplitter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn split_text(&self, text: &str) -> Vec<String> {
        let sentences = self.sentence_splitter.iter().fold(vec![text.to_owned()], |acc, splitter| {
            acc.iter().flat_map(|sentence| sentence.split(splitter)).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect::<Vec<String>>()
        });

        sentences
    }
}
