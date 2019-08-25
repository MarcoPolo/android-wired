use std::sync::Arc;

pub struct Callback {
  pub(crate) f: Arc<Box<dyn Fn() + Send + Sync>>,
}