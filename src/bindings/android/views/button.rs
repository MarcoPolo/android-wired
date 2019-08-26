use super::*;
use crate::bindings::callback::Callback;

#[derive(UpdateProp)]
pub struct Button {
  inner: PlatformView,
  after_remove: AttachedFutures,
  on_press: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Default for Button {
  fn default() -> Self {
    Button {
      inner: PlatformView::new(create_wired_native_view("BtnView")),
      after_remove: vec![],
      on_press: None,
    }
  }
}

impl Button {
  pub fn new<F>(on_press: F) -> Self
  where
    F: Fn() + Send + Sync + 'static,
  {
    let mut t = Self::default();
    t.on_press = Some(Box::new(on_press));
    t
  }
}

impl SetText for Button {}
impl SetTextSize for Button {}
impl OnPress for Button {}

impl Composable for Button {
  fn compose(&mut self, composer: &mut Composer) {
    if let Some(on_press) = self.on_press.take() {
      info!("REGISTERING in RUST");
      let cb: Box<dyn Any + Send> = Box::new(Some(Callback {
        f: Arc::new(on_press),
      }));
      self
        .inner
        .update_prop("on_press", cb)
        .expect("NO native android view");
    }
    composer
      .add_view(&mut self.inner)
      .expect("Couldn't add btn view");
  }
}
