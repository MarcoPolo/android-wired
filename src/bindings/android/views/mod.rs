pub mod button;
use crate::android_executor::spawn_future;
use crate::bindings::view_helpers::*;
use crate::style;
use crate::ui_tree::{
  with_parent, AttachedFutures, Composable, Composer, PlatformView, PlatformViewInner, COMPOSER,
};
use futures::future::ready;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use futures_signals::CancelableFutureHandle;
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
use jni::{JNIEnv, JavaVM};
use std::any::Any;
use std::cell::RefCell;
use std::error::Error;
use std::fmt::{self, Formatter};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use {
  discard::{Discard, DiscardOnDrop},
  futures::future::{BoxFuture, FutureExt},
};

use crate::bindings::callback::Callback;

pub use button::Button;
pub use wired_native_view::WiredNativeView;

auto_compose!(PhysicsLayout);
auto_compose!(StackLayout);
auto_compose!(Text);
auto_compose!(Button);

pub struct ViewFactory {
  inner: GlobalRef,
  jvm: Arc<JavaVM>,
}

impl ViewFactory {
  pub fn new(inner: GlobalRef, jvm: Arc<JavaVM>) -> Self {
    ViewFactory { inner, jvm }
  }
}

thread_local! {
    pub static VIEWFACTORY: RefCell<Option<ViewFactory>> = RefCell::new(None);
}

pub struct Text {
  inner: PlatformView,
  after_remove: AttachedFutures,
}

impl UpdateProp<f32> for Text {
  fn update_prop(&mut self, k: &str, v: f32) -> Result<(), Box<dyn Error>> {
    self.inner.update_prop(k, v)?;
    Ok(())
  }
}

impl UpdatePropSignal<f32> for Text {
  fn update_prop_signal<S>(&mut self, k: &'static str, s: S) -> Result<(), Box<dyn Error>>
  where
    S: 'static + Signal<Item = f32> + Send,
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

impl UpdateProp<String> for Text {
  fn update_prop(&mut self, k: &str, v: String) -> Result<(), Box<dyn Error>> {
    self.inner.update_prop(k, v)?;
    Ok(())
  }
}

impl UpdatePropSignal<String> for Text {
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

pub(crate) fn wrap_native_view(g: GlobalRef) -> Arc<Mutex<GlobalRef>> {
  Arc::new(Mutex::new(g))
}

fn create_wired_native_view(view_name: &'static str) -> WiredNativeView {
  VIEWFACTORY.with(|view_factory| {
    let mut view_factory_ref = view_factory.borrow_mut();
    let view_factory = view_factory_ref.as_mut().expect("No View Factory");
    let env = view_factory.jvm.get_env().expect("Couldn't get env");
    let native_view = env
      .call_method(
        view_factory.inner.as_obj(),
        format!("create{}", view_name),
        "()Ldev/fruit/androiddemo/WiredPlatformView;",
        &[],
      )
      .unwrap();
    WiredNativeView {
      kind: view_name,
      jvm: view_factory.jvm.clone(),
      native_view: wrap_native_view(env.new_global_ref(native_view.l().unwrap()).unwrap()),
    }
  })
}

impl Default for Text {
  fn default() -> Self {
    Text {
      inner: PlatformView::new(create_wired_native_view("TextView")),
      after_remove: vec![],
    }
  }
}

impl SetXY for Text {}
impl SetText for Text {}
impl Padding for Text {}
impl SetTextSize for Text {}

impl Text {
  pub fn new<S>(s: S) -> Text
  where
    S: Into<String>,
  {
    let t = Self::default();
    t.text(s.into())
  }
}

impl Composable for Text {
  fn compose(&mut self, composer: &mut Composer) {
    let mut after_remove = vec![];
    std::mem::swap(&mut self.after_remove, &mut after_remove);
    composer
      .add_view_with_futures(&mut self.inner, Some(after_remove))
      .expect("Couldn't add view");
  }
}

pub struct StackLayout {
  pub(crate) inner: PlatformView,
  after_remove: AttachedFutures,
}

impl SetHeightWidth for StackLayout {}
impl SetXY for StackLayout {}
impl SetOrientation for StackLayout {}

impl Composable for StackLayout {
  fn compose(&mut self, composer: &mut Composer) {
    info!("Composing stack layout");
    composer
      .add_view(&mut self.inner)
      .expect("Couldn't add view");
  }
}

impl Default for StackLayout {
  fn default() -> Self {
    StackLayout::new()
  }
}

impl UpdateProp<String> for StackLayout {
  fn update_prop(&mut self, k: &str, v: String) -> Result<(), Box<dyn Error>> {
    self.inner.update_prop(k, v)?;
    Ok(())
  }
}

impl UpdateProp<f32> for StackLayout {
  fn update_prop(&mut self, k: &str, v: f32) -> Result<(), Box<dyn Error>> {
    self.inner.update_prop(k, v)?;
    Ok(())
  }
}

impl UpdatePropSignal<f32> for StackLayout {
  fn update_prop_signal<S>(&mut self, k: &'static str, s: S) -> Result<(), Box<dyn Error>>
  where
    S: 'static + Signal<Item = f32> + Send,
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

impl ParentWith for StackLayout {
  fn with<F>(mut self, f: F) -> Self
  where
    F: FnOnce(),
  {
    with_parent(&mut self.inner, f);
    self
  }
}

impl StackLayout {
  pub fn new() -> Self {
    StackLayout {
      inner: PlatformView::new(create_wired_native_view("StackLayoutView")),
      after_remove: vec![],
    }
  }
}

// Physics layout

pub struct PhysicsLayout {
  pub(crate) inner: PlatformView,
  after_remove: AttachedFutures,
}

impl Composable for PhysicsLayout {
  fn compose(&mut self, composer: &mut Composer) {
    info!("Composing physics layout");
    composer
      .add_view(&mut self.inner)
      .expect("Couldn't add view");
  }
}

impl Default for PhysicsLayout {
  fn default() -> Self {
    PhysicsLayout::new()
  }
}

impl SetOrientation for PhysicsLayout {}
impl SetHeightWidth for PhysicsLayout {}
impl ParentWith for PhysicsLayout {
  fn with<F>(mut self, f: F) -> Self
  where
    F: FnOnce(),
  {
    with_parent(&mut self.inner, f);
    self
  }
}


impl UpdateProp<String> for PhysicsLayout {
  fn update_prop(&mut self, k: &str, v: String) -> Result<(), Box<dyn Error>> {
    self.inner.update_prop(k, v)?;
    Ok(())
  }
}

impl UpdateProp<f32> for PhysicsLayout {
  fn update_prop(&mut self, k: &str, v: f32) -> Result<(), Box<dyn Error>> {
    self.inner.update_prop(k, v)?;
    Ok(())
  }
}

impl UpdatePropSignal<f32> for PhysicsLayout {
  fn update_prop_signal<S>(&mut self, k: &'static str, s: S) -> Result<(), Box<dyn Error>>
  where
    S: 'static + Signal<Item = f32> + Send,
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

impl PhysicsLayout {
  pub fn new() -> Self {
    PhysicsLayout {
      inner: PlatformView::new(create_wired_native_view("PhysicsLayout")),
      after_remove: vec![],
    }
  }
}

mod wired_native_view {
  use super::*;

pub struct WiredNativeView {
  pub kind: &'static str,
  pub jvm: Arc<JavaVM>,
  pub native_view: Arc<Mutex<GlobalRef>>,
}

impl fmt::Debug for WiredNativeView {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "WiredNativeView [{}]", self.kind)
  }
}

impl UpdateProp<f32> for WiredNativeView {
  fn update_prop(&mut self, s: &str, v: f32) -> Result<(), Box<dyn Error>> {
    let env = self.jvm.get_env()?;
    env.call_method(
      self.native_view.lock().unwrap().as_obj(),
      "updateProp",
      "(Ljava/lang/String;F)V",
      &[
        JValue::Object(env.new_string(s).unwrap().into()),
        JValue::Float(v),
      ],
    )?;
    Ok(())
  }
}

impl UpdateProp<String> for WiredNativeView {
  fn update_prop(&mut self, s: &str, string: String) -> Result<(), Box<dyn Error>> {
    let env = self.jvm.get_env()?;
    env.call_method(
      self.native_view.lock().unwrap().as_obj(),
      "updateProp",
      "(Ljava/lang/String;Ljava/lang/String;)V",
      &[
        JValue::Object(env.new_string(s).unwrap().into()),
        JValue::Object(env.new_string(&string).unwrap().into()),
      ],
    )?;
    Ok(())
  }
}

impl UpdateProp<Callback> for WiredNativeView {
  fn update_prop(&mut self, s: &str, cb: Callback) -> Result<(), Box<dyn Error>> {
    let env = self.jvm.get_env()?;
    debug!("Setting callback");
    let view_class = env.find_class("dev/fruit/androiddemo/RustCallback")?;
    let callback_obj = env.new_object(view_class, "()V", &[])?;
    env.set_rust_field(callback_obj, "ptr", cb)?;

    env.call_method(
      self.native_view.lock().unwrap().as_obj(),
      "updateProp",
      "(Ljava/lang/String;Ldev/fruit/androiddemo/RustCallback;)V",
      &[
        JValue::Object(env.new_string(s).unwrap().into()),
        JValue::Object(callback_obj),
      ],
    )?;
    Ok(())
  }
}

impl UpdateProp<Box<dyn Any + Send>> for WiredNativeView {
  fn update_prop(&mut self, s: &str, mut v: Box<dyn Any + Send>) -> Result<(), Box<dyn Error>> {
    let env = self.jvm.get_env()?;
    if let Some(cb) = v.downcast_mut::<Option<Callback>>() {
      debug!("Setting callback");
      let view_class = env.find_class("dev/fruit/androiddemo/RustCallback")?;
      let callback_obj = env.new_object(view_class, "()V", &[])?;
      let cb: Callback = cb.take().expect("No Callback?");

      debug!("Set obj");
      env.set_rust_field(callback_obj, "ptr", cb)?;

      env.call_method(
        self.native_view.lock().unwrap().as_obj(),
        "updateProp",
        "(Ljava/lang/String;Ldev/fruit/androiddemo/RustCallback;)V",
        &[
          JValue::Object(env.new_string(s).unwrap().into()),
          JValue::Object(callback_obj),
        ],
      )?;
    } else {
      info!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!               COULDN'T UPDATE");
    }
    Ok(())
  }
}

impl PlatformViewInner for WiredNativeView {
  /// If you append a child that is attached somewhere else, you should move the child.
  fn append_child(&mut self, c: &PlatformView) -> Result<(), Box<dyn Error>> {
    let env = self.jvm.get_env()?;
    info!("Appending {} ", self.kind);
    env.call_method(
      self.native_view.lock().unwrap().as_obj(),
      "appendChild",
      "(Ldev/fruit/androiddemo/WiredPlatformView;)V",
      &[JValue::Object(
        c.get_raw_view()?
          .lock()
          .unwrap()
          .downcast_ref::<GlobalRef>()
          .expect("Not a Wired NativeView ref")
          .as_obj(),
      )],
    )?;
    Ok(())
  }
  /// Do not insert a child that is already there! undefined behavior!
  fn insert_child_at(&mut self, c: &PlatformView, idx: usize) -> Result<(), Box<dyn Error>> {
    let env = self.jvm.get_env()?;
    env.call_method(
      self.native_view.lock().unwrap().as_obj(),
      "insertChildAt",
      "(Ldev/fruit/androiddemo/WiredPlatformView;I)V",
      &[
        JValue::Object(
          c.get_raw_view()?
            .lock()
            .unwrap()
            .downcast_ref::<GlobalRef>()
            .expect("Not a Wired NativeView ref")
            .as_obj(),
        ),
        JValue::Int(idx as i32),
      ],
    )?;
    Ok(())
  }
  /// should not tear down the child! since it may be placed somewhere else later
  fn remove_child(&mut self, c: &PlatformView) -> Result<(), Box<dyn Error>> {
    let env = self.jvm.get_env()?;
    env.call_method(
      self.native_view.lock().unwrap().as_obj(),
      "removeChild",
      "(Ldev/fruit/androiddemo/WiredPlatformView;)V",
      &[JValue::Object(
        c.get_raw_view()?
          .lock()
          .unwrap()
          .downcast_ref::<GlobalRef>()
          .expect("Not a Wired NativeView ref")
          .as_obj(),
      )],
    )?;
    Ok(())
  }
  /// Should not tear down the child (same as remove_child)
  fn remove_child_index(&mut self, idx: usize) -> Result<(), Box<dyn Error>> {
    let env = self.jvm.get_env()?;
    env.call_method(
      self.native_view.lock().unwrap().as_obj(),
      "removeChildIndex",
      "(I)V",
      &[JValue::Int(idx as i32)],
    )?;
    Ok(())
  }

  fn get_raw_view(&self) -> Result<Arc<Mutex<dyn Any>>, Box<dyn Error>> {
    Ok(self.native_view.clone())
  }
}
}