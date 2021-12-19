use crate::{async_sender::AsyncSender, error::PipelineResult};
use core::{
    any::type_name,
    fmt::{self, Debug},
    time::Duration,
};
use std::sync::{Arc, Mutex};

/// A context struct that will be passed to each time an [`Aggregator`] is polled. 
pub struct Context<Item: Send + 'static, State> {
    pub sender: AsyncSender<Box<Item>>,
    task_num: usize,
    pub state: Arc<Mutex<State>>,
}

impl<Item: Send + 'static, State> Context<Item, State> {
    /// Constructs a new Context. 
    pub(crate) fn new(
        sender: AsyncSender<Box<Item>>,
        state: Arc<Mutex<State>>,
        task_num: usize,
    ) -> Self {
        Self {
            sender,
            state,
            task_num,
        }
    }

    /// Retrieves the number of the current task. Mostly useful for debugging, 
    pub fn task_num(&self) -> usize {
        self.task_num
    }
}

impl<Item: Send + 'static, State> Debug for Context<Item, State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(&format!(
            "Context<Item = \"{}\", State = \"{}\">",
            type_name::<Item>(),
            type_name::<State>()
        ))
        .field("task_num", &self.task_num)
        .finish()
    }
}

/// The interface for collecting data and sending them into the pipeline
#[async_trait]
pub trait Aggregator: Debug + Send + Sync + 'static {
    type Item: Send + 'static;
    type PipelineState;

    /// Decide how long this task should go idle after getting polled. 
    /// The default is fine for most cases but sometimes you need custom logic (e.g. rate limiting)
    /// and need to dynamically adjust the timeout. 
    fn sleep_duration(&self) -> Duration {
        Duration::from_secs(15 * 60)
    }

    /// Poll the aggregator and let it do its magic. 
    async fn poll(
        &mut self,
        ctx: &mut Context<Self::Item, Self::PipelineState>,
    ) -> PipelineResult<()>;
}
