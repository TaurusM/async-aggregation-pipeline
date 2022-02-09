use crate::{
    aggregator::{Aggregator, Context},
    async_sender::AsyncSender,
    output_filter::OutputFilter,
};

use futures::stream::{FuturesUnordered, StreamExt};

use core::{mem, time::Duration};
use std::sync::{Arc, Mutex, RwLock};

use crossbeam_channel::Receiver;
use tokio::{signal, task::JoinHandle, time};

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Command {
    Continue,
    Shutdown,
}

/// The async aggregator pipeline.
/// An example for why generics are horrible.
pub struct Pipeline<Item: Send + Sized + 'static, State: Send + 'static = ()> {
    /// List of OutputFilters to iterate over in the receiver task.
    filters: Vec<Box<dyn OutputFilter<Item = Item>>>,

    /// List of aggregator tasks to spawn.
    aggregators: Vec<Box<dyn Aggregator<Item = Item, PipelineState = State>>>,

    /// The number of aggregator tasks spawned so far.
    task_number: Option<usize>,

    /// The sending part of the item pipeline.
    /// This will be cloned for each aggregator task spawned.
    item_sender: AsyncSender<Item>,

    /// The receiving end of the item pipeline.
    /// There's only one and it will be taken after spawning the receiver task.
    item_receiver: Option<Receiver<Item>>,

    cmd_flag: Arc<RwLock<Command>>,

    /// State that gets shared through the context struct
    /// for the Aggregators
    state: Arc<Mutex<State>>,

    handles: Vec<JoinHandle<()>>,

    output_handle: Option<JoinHandle<()>>,
}

impl<Item: Send + Sized + 'static, State: Send + 'static> Pipeline<Item, State> {
    pub fn new(state: State) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();

        Self {
            filters: Vec::new(),
            aggregators: Vec::new(),
            task_number: None,
            item_sender: AsyncSender::new(sender, None),
            item_receiver: Some(receiver),
            cmd_flag: Arc::new(RwLock::new(Command::Continue)),
            state: Arc::new(Mutex::new(state)),
            handles: Vec::new(),
            output_handle: None,
        }
    }

    pub fn set_output_filters(mut self, filters: Vec<Box<dyn OutputFilter<Item = Item>>>) -> Self {
        self.filters = filters;
        self
    }

    pub fn add_output_filter(mut self, filter: impl OutputFilter<Item = Item>) -> Self {
        self.filters.push(Box::new(filter));
        self
    }

    pub fn set_aggregators(
        mut self,
        aggregators: Vec<Box<dyn Aggregator<Item = Item, PipelineState = State>>>,
    ) -> Self {
        self.aggregators = aggregators;
        self
    }

    pub fn add_aggregator(
        mut self,
        aggregator: impl Aggregator<Item = Item, PipelineState = State>,
    ) -> Self {
        self.aggregators.push(Box::new(aggregator));
        self
    }

    /// Spawns an async task that will listen for items found by the aggregators
    /// and apply all result filters in order, until one consumes it.
    ///
    /// # Example
    /// ```no_run
    /// Pipeline::new()
    ///     .add_output_filter(some_output_filter)
    ///     .spawn_output_filters()
    /// ```
    ///
    /// # Panics
    /// This panics if it gets called more than one time
    pub async fn spawn_output_filters(mut self) -> Self {
        let receiver = self
            .item_receiver
            .take()
            .expect("Attempted to spawn receiver task twice (no receiver left)");
        let mut out_filters = mem::take(&mut self.filters);

        let cmd_flag = Arc::clone(&self.cmd_flag);
        let output_handle: JoinHandle<()> = tokio::spawn(async move {
            debug!("Spawned receiver task!");

            let cmd_flag = Arc::clone(&cmd_flag);

            'outer: loop {
                if let Some(mut item) = receiver.try_recv().ok() {
                    debug!("Found a match!");

                    'l: for filter in out_filters.iter_mut() {
                        item = match filter.filter(item).await {
                            Some(i) => i,
                            None => break 'l,
                        }
                    }
                } else {
                    // if nothing is in the queue and the shutdown flag is set terminate the loop
                    if let Ok(lock) = cmd_flag.read() {
                        if *lock == Command::Shutdown {
                            info!("Shutting down output filter loop");
                            break 'outer;
                        }
                    }

                    trace!("Nothing in the queue, sleeping for about a second...");
                    time::sleep(Duration::from_millis(100)).await;
                }
            }
        });

        self.output_handle = Some(output_handle);

        self
    }

    pub async fn spawn_aggregators(mut self) -> Self {
        let mut task_number = self.task_number.map(|n| n + 1).unwrap_or(0);

        for mut aggregator in self.aggregators.drain(..) {
            let mut ctx = Context::new(
                self.item_sender.clone(),
                Arc::clone(&self.state),
                task_number,
                Arc::clone(&self.cmd_flag),
            );

            let handle = tokio::spawn(async move {
                info!("Spawned aggregator task #{}", task_number);

                let mut next = time::Instant::now();

                'l: loop {
                    if let Ok(lock) = ctx.cmd_flag.read() {
                        if *lock == Command::Shutdown {
                            info!("Shutting down task #{}", ctx.task_num());
                            break 'l;
                        }
                    }

                    if time::Instant::now() < next {
                        time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }

                    if let Err(why) = aggregator.poll(&mut ctx).await {
                        error!("{:?}", why)
                    }

                    next += aggregator.sleep_duration();
                }
            });

            self.handles.push(handle);

            task_number += 1;
        }

        self.task_number = Some(task_number);
        self
    }

    pub async fn shutdown(self) {
        {
            let mut lock = self.cmd_flag.write().unwrap();
            *lock = Command::Shutdown;
            info!("Set command flag to SHUTDOWN");
        }

        info!("Waiting for aggregators to finish...");

        let mut handles: FuturesUnordered<JoinHandle<()>> = self.handles.into_iter().collect();

        while let Some(r) = handles.next().await {
            if let Err(e) = r {
                error!("aggregator threw an error while terminating: {}", e);
            }
        }

        if let Some(h) = self.output_handle {
            info!("Processing leftover aggregated data");
            h.await.ok();
        }
    }

    pub async fn spin(self) {
        if let Ok(_) = signal::ctrl_c().await {
            self.shutdown().await;
        }
    }

    pub async fn setup_and_run(self) {
        self.spawn_output_filters()
            .await
            .spawn_aggregators()
            .await
            .spin()
            .await
    }
}
