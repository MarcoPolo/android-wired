use super::*;
use crate::bindings::callback::Callback;

pub struct Button {
  inner: PlatformView,
  on_press: Option<Box<dyn Fn() + Send + Sync>>,
  after_remove: AttachedFutures,
}

impl Default for Button {
  fn default() -> Self {
    VIEWFACTORY.with(|view_factory| {
      let mut view_factory_ref = view_factory.borrow_mut();
      let view_factory = view_factory_ref.as_mut().expect("No View Factory");
      let env = view_factory.jvm.get_env().expect("Couldn't get env");
      let native_view = env
        .call_method(
          view_factory.inner.as_obj(),
          "createBtnView",
          "()Ldev/fruit/androiddemo/WiredPlatformView;",
          &[],
        )
        .unwrap();
      let wired_native_view = WiredNativeView {
        kind: "Button",
        jvm: view_factory.jvm.clone(),
        native_view: wrap_native_view(env.new_global_ref(native_view.l().unwrap()).unwrap()),
      };
      Button {
        inner: PlatformView::new(wired_native_view),
        after_remove: vec![],
        on_press: None,
      }
    })
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

  pub fn size(mut self, f: f32) -> Self {
    self.inner.update_prop("text_size", f).unwrap();
    self
  }

  pub fn label<S>(mut self, s: S) -> Self
  where
    S: Into<String>,
  {
    let string: String = s.into();
    self
      .inner
      .update_prop("text", string)
      .expect("Couldn't update text");
    self
  }

  pub fn text_signal<S>(mut self, s: S) -> Self
  where
    S: 'static + Signal<Item = String> + Send,
  {
    self.update_prop_signal("text", s).unwrap();
    self
  }
}

impl UpdateProp<Callback> for Button {
  fn update_prop(&mut self, k: &str, v: Callback) -> Result<(), Box<dyn Error>> {
    self.inner.update_prop(k, v)?;
    Ok(())
  }
}

impl UpdatePropSignal<Callback> for Button {
  fn update_prop_signal<S>(&mut self, k: &'static str, s: S) -> Result<(), Box<dyn Error>>
  where
    S: 'static + Signal<Item = Callback> + Send,
  {
    let mut platform_view = self.inner.clone();
    let f = s.for_each(move |i| {
      platform_view.update_prop(k, i).expect("view is there");
      ready(())
    });
    self.after_remove.push(spawn_future(f));
    Ok(())
  }
}

impl UpdatePropSignal<String> for Button {
  fn update_prop_signal<S>(&mut self, k: &'static str, s: S) -> Result<(), Box<dyn Error>>
  where
    S: 'static + Signal<Item = String> + Send,
  {
    let mut platform_view = self.inner.clone();
    let f = s.for_each(move |i| {
      platform_view.update_prop(k, i).expect("view is there");
      ready(())
    });
    self.after_remove.push(spawn_future(f));
    Ok(())
  }
}

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
