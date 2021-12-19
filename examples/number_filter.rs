#[macro_use]
extern crate async_trait;

use async_aggregation_pipeline::prelude::*;

use std::ops::Range;

#[derive(Debug)]
struct NumberAggregator(Range<usize>);

#[async_trait]
impl Aggregator for NumberAggregator {
    type Item = usize;
    type PipelineState = ();

    fn sleep_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs_f32(0.1)
    }

    async fn poll(
        &mut self,
        ctx: &mut Context<Self::Item, Self::PipelineState>,
    ) -> PipelineResult<()> {
        if self.0.start < self.0.end {
            ctx.sender.send(self.0.start).await;
            self.0.start += 1;
        }

        Ok(())
    }
}

struct NotAMultipleOfFilter(usize);

#[async_trait]
impl OutputFilter for NotAMultipleOfFilter {
    type Item = usize;

    async fn filter(&mut self, entry: Self::Item) -> Option<Self::Item> {
        if entry % self.0 == 0 {
            None
        } else {
            Some(entry)
        }
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
    Pipeline::new(())
        .add_aggregator(NumberAggregator(0..20))
        .add_aggregator(NumberAggregator(30..50))
        .add_aggregator(NumberAggregator(2..7))
        .set_output_filters(vec![
            Box::new(NotAMultipleOfFilter(2)),
            Box::new(NotAMultipleOfFilter(5)),
        ])
        .add_output_filter(Print)
        .spawn_aggregators()
        .await
        .spawn_output_filters()
        .await
        .run()
        .await
}
