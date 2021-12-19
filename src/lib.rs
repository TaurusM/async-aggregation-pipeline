//! A pipeline framework for dealing with async aggregation of data.

#[macro_use]
extern crate async_trait;

#[macro_use]
extern crate tracing;

pub mod output_filter;

pub mod pipeline;

pub mod async_sender;

pub mod aggregator;

pub mod error;

pub mod prelude {
    pub use super::aggregator::{Aggregator, Context};
    pub use super::async_sender::AsyncSender;
    pub use super::error::{PipelineError, PipelineResult};
    pub use super::output_filter::OutputFilter;
    pub use super::pipeline::Pipeline;
}
