use crate::{async_sender::AsyncSender, error::PipelineResult, pipeline::Command};
use core::{
    any::type_name,
    fmt::{self, Debug},
    time::Duration,
};
use std::sync::{Arc, Mutex, RwLock};

/// A context struct that will be passed to each time an [`Aggregator`] is polled.
pub struct Context<Item: Send + Sized + 'static, State> {
    pub sender: AsyncSender<Item>,
    task_num: usize,
    pub state: Arc<Mutex<State>>,
    pub(crate) cmd_flag: Arc<RwLock<Command>>,
}

impl<Item: Send + Sized + 'static, State> Context<Item, State> {
    /// Constructs a new Context.
    pub(crate) fn new(
        sender: AsyncSender<Item>,
        state: Arc<Mutex<State>>,
        task_num: usize,
        cmd_flag: Arc<RwLock<Command>>,
    ) -> Self {
        Self {
            sender,
            state,
            task_num,
            cmd_flag,
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
    type Item: Send + Sized + 'static;
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
