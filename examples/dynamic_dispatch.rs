#[macro_use]
extern crate async_trait;

use async_aggregation_pipeline::prelude::*;

trait Entry: core::fmt::Debug + Send {}

#[derive(Debug)]
struct EntryOne(String);

#[derive(Debug)]
struct EntryTwo(usize);

impl Entry for EntryOne {}
impl Entry for EntryTwo {}

#[derive(Debug)]
struct Input(bool);

#[async_trait]
impl Aggregator for Input {
    type Item = Box<dyn Entry>;
    type PipelineState = ();

    async fn poll(
        &mut self,
        ctx: &mut Context<Self::Item, Self::PipelineState>,
    ) -> PipelineResult<()> {
        if self.0 {
            ctx.sender
                .send(Box::new(EntryOne(String::from("hi"))))
                .await;
            ctx.sender.send(Box::new(EntryTwo(0xcafe))).await;

            self.0 = false;
        }

        Ok(())
    }
}

struct Output;

#[async_trait]
impl OutputFilter for Output {
    type Item = Box<dyn Entry>;

    async fn filter(&mut self, entry: Self::Item) -> Option<Self::Item> {
        println!("{:?}", entry);
        None
    }
}

#[tokio::main]
async fn main() {
    Pipeline::new(())
        .add_aggregator(Input(true))
        .add_output_filter(Output)
        .spawn_aggregators()
        .await
        .spawn_output_filters()
        .await
        .run()
        .await
}
