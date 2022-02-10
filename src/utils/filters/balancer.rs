use crate::output_filter::OutputFilter;

/// A filter that will iterate through all inner filters
/// one by one.
///
/// Useful for e.g. balancing the load between multiple webhooks
/// to get around rate limiting.
pub struct LoadBalancerFilter<Item: Send + Sized + 'static> {
    filters: Vec<Box<dyn OutputFilter<Item = Item>>>,
    cur_index: usize,
    identifier: String,
}

impl<Item: Send + Sized + 'static> LoadBalancerFilter<Item> {
    pub fn new(identifier: String) -> Self {
        Self {
            filters: Vec::new(),
            cur_index: 0,
            identifier,
        }
    }

    pub fn from(identifier: String, filters: Vec<Box<dyn OutputFilter<Item = Item>>>) -> Self {
        Self {
            filters,
            cur_index: 0,
            identifier,
        }
    }

    pub fn add(mut self, filter: impl OutputFilter<Item = Item> + 'static) -> Self {
        self.filters.push(Box::new(filter));
        self
    }
}

#[async_trait]
impl<Item: Send + Sized + 'static> OutputFilter for LoadBalancerFilter<Item> {
    type Item = Item;

    async fn filter(&mut self, entry: Self::Item) -> Option<Self::Item> {
        if self.filters.len() == 0 {
            return Some(entry);
        }

        let filter = self
            .filters
            .get_mut(self.cur_index)
            .expect("see check above");

        let res = filter.filter(entry).await;
        self.cur_index = (self.cur_index + 1) % self.filters.len();
        res
    }

    fn on_load(&mut self) {
        info!("Loading LoadBalancerFilter(\"{}\") with {} wrapped filters", self.identifier, self.filters.len())
    }
}
