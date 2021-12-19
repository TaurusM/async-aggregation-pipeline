#[async_trait]
pub trait OutputFilter: Send + 'static {
    type Item;

    async fn filter(&mut self, entry: Self::Item) -> Option<Self::Item>;
}
