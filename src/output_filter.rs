#[async_trait]
pub trait OutputFilter: Send + 'static {
    type Item: Sized + Send;

    async fn filter(&mut self, entry: Self::Item) -> Option<Self::Item>;

    fn on_load(&mut self) {}
}
