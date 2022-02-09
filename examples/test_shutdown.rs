#[macro_use] extern crate async_trait;
#[macro_use] extern crate tracing;

use std::time::Duration;

use tracing_subscriber::{self, EnvFilter};

use async_aggregation_pipeline::prelude::*;

#[derive(Debug)]
struct WaitingAggregator(Duration);

impl WaitingAggregator {
    fn new(secs: u64) -> Self {
        Self(Duration::from_secs(secs))
    }
}

#[async_trait]
impl Aggregator for WaitingAggregator {
    type Item = usize;
    type PipelineState = ();

    fn sleep_duration(&self) -> Duration {
        info!("Sleeping for {:?}", self.0);
        self.0.clone()
    }

    async fn poll(&mut self, ctx: &mut Context<Self::Item, Self::PipelineState>) -> PipelineResult<()> {
        ctx.sender.send(self.0.as_secs() as usize).await;
        
        Ok(())
    }
}


#[derive(Debug)]
struct BlockingAggregator(Duration);


impl BlockingAggregator {
    fn new(secs: u64) -> Self {
        Self(Duration::from_secs(secs))
    }
}

#[async_trait]
impl Aggregator for BlockingAggregator {
    type Item = usize;
    type PipelineState = ();

    async fn poll(&mut self, ctx: &mut Context<Self::Item, Self::PipelineState>) -> PipelineResult<()> {
        ctx.sender.send(self.0.as_secs() as usize).await;
        info!("Blocking thread...");
        std::thread::sleep(self.0.clone());
        Ok(())
    }
}


struct Print;

#[async_trait]
impl OutputFilter for Print {
    type Item = usize;

    async fn filter(&mut self, entry: Self::Item) -> Option<Self::Item> {
        println!("{}", entry);
        None
    }
}


#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::fmt::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    info!("Press Ctrl+C to terminate");

    Pipeline::new(())
        .add_aggregator(WaitingAggregator::new(50000))
        .add_aggregator(WaitingAggregator::new(7))
        .add_aggregator(BlockingAggregator::new(10))
        .add_output_filter(Print)
        .setup_and_run()
        .await
}
