use std::{error::Error, fmt};

#[derive(Debug)]
pub enum PipelineError {}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(&self, f)
    }
}

impl Error for PipelineError {}

pub type PipelineResult<T> = Result<T, Box<dyn Error>>;
