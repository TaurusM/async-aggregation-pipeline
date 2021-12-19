#[macro_use]
extern crate async_trait;

pub use async_aggregation_pipeline::prelude::*;

// a dummy state which contains a single string
struct DummyState(String);

// a dummy aggregator which contains a vec with numbers and sends each element into the pipeline, until the vec is empty
#[derive(Debug)]
struct DummyAggregator {
    values: Vec<u8>,
}

impl DummyAggregator {
    fn new() -> Self {
        Self {
            values: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]
        }
    }
}

#[async_trait]
impl Aggregator for DummyAggregator {
    type Item = u8;
    type PipelineState = DummyState;

    fn sleep_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(5)
    }

    async fn poll(&mut self, ctx: &mut Context<Self::Item, Self::PipelineState>) -> PipelineResult<()> {
        if let Some(v) = self.values.pop() {
            ctx.sender.send(Box::new(v)).await;
        }        

        Ok(())
    }
}

// a dummy output that contains no data itself, and just prints each received element and consumes it
struct DummyOutput;

#[async_trait]
impl OutputFilter for DummyOutput {
    type Item = Box<u8>;
    
    async fn filter(&mut self, entry: Self::Item) -> Option<Self::Item> {
        println!("{}", *entry);
        None
    }
}

#[tokio::main]
async fn main() -> ! {
    Pipeline::new(DummyState(String::from("cool")))
        .add_aggregator(DummyAggregator::new())
        .add_output_filter(DummyOutput)
        .spawn_output_filters()
        .await
        .spawn_aggregators()
        .await
        .run()
        .await
}
