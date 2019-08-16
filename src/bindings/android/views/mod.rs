pub mod button;
use crate::android_executor::spawn_future;
use crate::ui_tree::{Composable, Composer, PlatformView, PlatformViewInner, COMPOSER};
use discard::Discard;
use futures::future::ready;
use futures_signals::signal::{Mutable, Signal, SignalExt};
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
  discard::DiscardOnDrop,
  futures::future::{BoxFuture, FutureExt},
};

use crate::bindings::android::callback::Callback;

pub use button::Button;

macro_rules! auto_compose {
  ($e:ty) => {
    impl Drop for $e {
      fn drop(&mut self) {
        COMPOSER.with(|c| {
          let mut c = c.borrow_mut();
          self.compose(&mut c);
        })
      }
    }
  };
}

auto_compose!(StackLayout);
auto_compose!(Text);
auto_compose!(Button);

pub struct ViewFactory {
  inner: GlobalRef,
  jvm: Arc<JavaVM>,
}

impl ViewFactory {
  pub fn new(inner: GlobalRef, jvm: JavaVM) -> Self {
    ViewFactory {
      inner,
      jvm: Arc::new(jvm),
    }
  }
}

thread_local! {
    pub static VIEWFACTORY: RefCell<Option<ViewFactory>> = RefCell::new(None);
}

pub struct Text {
  inner: PlatformView,
  after_remove: Vec<Box<FnOnce()>>,
}

fn wrap_native_view(g: GlobalRef) -> Arc<Mutex<GlobalRef>> {
  Arc::new(Mutex::new(g))
}

impl Default for Text {
  fn default() -> Self {
    VIEWFACTORY.with(|view_factory| {
      let mut view_factory_ref = view_factory.borrow_mut();
      let view_factory = view_factory_ref.as_mut().expect("No View Factory");
      let env = view_factory.jvm.get_env().expect("Couldn't get env");
      let native_view = env
        .call_method(
          view_factory.inner.as_obj(),
          "createTextView",
          "()Ldev/fruit/androiddemo/WiredPlatformView;",
          &[],
        )
        .unwrap();
      let wired_native_view = WiredNativeView {
        kind: "text",
        jvm: view_factory.jvm.clone(),
        native_view: wrap_native_view(env.new_global_ref(native_view.l().unwrap()).unwrap()),
      };
      Text {
        inner: PlatformView::new(wired_native_view),
        after_remove: vec![],
      }
    })
  }
}

impl Text {
  pub fn new<S>(s: S) -> Text
  where
    S: Into<String>,
  {
    let mut t = Self::default();
    t.text_mut(s.into());
    t
  }

  pub fn padding_left(mut self, f: f32) -> Self {
    self
      .inner
      .update_prop("left_pad", Box::new(f))
      .expect("Couldn't update left padding");
    self
  }

  pub fn x_pos_signal<S>(mut self, s: S) -> Self
  where
    S: 'static + Signal<Item = f32> + Send,
  {
    let mut platform_view = self.inner.clone();
    let f = s.for_each(move |i| {
      platform_view
        .update_prop("set_x", Box::new(i))
        .expect("view is there");
      ready(())
    });

    let cancel = spawn_future(f);
    let handle = DiscardOnDrop::leak(cancel);
    let cleanup = Box::new(move || {
      handle.discard();
    });
    // DiscardOnDrop::leak(cancel);
    // let id = AFTER_REMOVE_CALLBACKS.with(move |r| {
    //   let after_remove = r.borrow_mut();
    //   // after_remove.push(cleanup);
    //   after_remove.len()
    // });

    self.after_remove.push(cleanup);
    self
  }

  pub fn padding_left_signal<S>(mut self, s: S) -> Self
  where
    S: 'static + Signal<Item = f32> + Send,
  {
    let mut platform_view = self.inner.clone();
    let f = s.for_each(move |i| {
      platform_view
        .update_prop("left_pad", Box::new(i))
        .expect("view is there");
      ready(())
    });

    let cancel = spawn_future(f);
    let handle = DiscardOnDrop::leak(cancel);
    let cleanup = Box::new(move || {
      handle.discard();
    });
    // DiscardOnDrop::leak(cancel);
    // let id = AFTER_REMOVE_CALLBACKS.with(move |r| {
    //   let after_remove = r.borrow_mut();
    //   // after_remove.push(cleanup);
    //   after_remove.len()
    // });

    self.after_remove.push(cleanup);
    self
  }

  pub fn text_signal<S>(mut self, s: S) -> Self
  where
    S: 'static + Signal<Item = String> + Send,
  {
    let mut platform_view = self.inner.clone();
    let f = s.for_each(move |string| {
      platform_view
        .update_prop("text", Box::new(string.clone()))
        .expect("view is there");
      ready(())
    });

    let cancel = spawn_future(f);
    let handle = DiscardOnDrop::leak(cancel);
    let cleanup = Box::new(move || {
      handle.discard();
    });
    // DiscardOnDrop::leak(cancel);
    // let id = AFTER_REMOVE_CALLBACKS.with(move |r| {
    //   let after_remove = r.borrow_mut();
    //   // after_remove.push(cleanup);
    //   after_remove.len()
    // });

    self.after_remove.push(cleanup);
    self
  }

  pub fn size(mut self, f: f32) -> Self {
    self.size_mut(f);
    self
  }

  pub fn text_mut(&mut self, s: String) {
    self
      .inner
      .update_prop("text", Box::new(s))
      .expect("Couldn't update text");
  }

  pub fn size_mut(&mut self, f: f32) {
    self.inner.update_prop("text_size", Box::new(f)).unwrap();
  }
}

impl Composable for Text {
  fn compose(&mut self, composer: &mut Composer) {
    composer
      .add_view(&mut self.inner)
      .expect("Couldn't add view");
  }
}

impl Composable for StackLayout {
  fn compose(&mut self, composer: &mut Composer) {
    composer
      .add_view(&mut self.inner)
      .expect("Couldn't add view");
  }
}

pub struct StackLayout {
  inner: PlatformView,
}

impl Default for StackLayout {
  fn default() -> Self {
    StackLayout::new()
  }
}

impl StackLayout {
  pub fn new() -> Self {
    VIEWFACTORY.with(|view_factory| {
      let mut view_factory_ref = view_factory.borrow_mut();
      let view_factory = view_factory_ref.as_mut().expect("No View Factory");
      let env = view_factory.jvm.get_env().expect("Couldn't get env");
      let native_view = env
        .call_method(
          view_factory.inner.as_obj(),
          "createStackLayoutView",
          "()Ldev/fruit/androiddemo/WiredPlatformView;",
          &[],
        )
        .unwrap();
      let underlying_view = WiredNativeView {
        kind: "StackLayout",
        jvm: view_factory.jvm.clone(),
        native_view: wrap_native_view(env.new_global_ref(native_view.l().unwrap()).unwrap()),
      };
      StackLayout {
        inner: PlatformView::new(underlying_view),
      }
    })
  }

  pub fn with<F>(self, f: F) -> Self
  where
    F: FnOnce(),
  {
    let last_parent = COMPOSER.with(|composer| {
      let mut composer = composer.borrow_mut();
      let last_parent = composer.curent_parent.take();
      composer.curent_parent = Some(self.inner.clone());
      last_parent
    });

    f();

    let prev_parent = COMPOSER.with(move |composer| {
      let mut composer = composer.borrow_mut();
      let prev_parent = composer.curent_parent.take().unwrap();
      composer.curent_parent = last_parent;
      prev_parent
    });

    StackLayout { inner: prev_parent }
  }

  pub(crate) fn get_native_view(&self) -> Result<GlobalRef, Box<dyn Error>> {
    self.inner.get_raw_view().map(move |v| {
      v.lock()
        .expect("Couldn't get lock")
        .downcast_ref::<GlobalRef>()
        .expect("Not a global ref")
        .clone()
    })
  }
}

struct WiredNativeView {
  kind: &'static str,
  jvm: Arc<JavaVM>,
  native_view: Arc<Mutex<GlobalRef>>,
}

impl fmt::Debug for WiredNativeView {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "WiredNativeView [{}]", self.kind)
  }
}

impl PlatformViewInner for WiredNativeView {
  fn update_prop(&mut self, s: &str, mut v: Box<dyn Any + Send>) -> Result<(), Box<dyn Error>> {
    let env = self.jvm.get_env()?;
    if let Some(string) = v.downcast_ref::<String>() {
      env.call_method(
        self.native_view.lock().unwrap().as_obj(),
        "updateProp",
        "(Ljava/lang/String;Ljava/lang/Object;)V",
        &[
          JValue::Object(env.new_string(s).unwrap().into()),
          JValue::Object(env.new_string(&string).unwrap().into()),
        ],
      )?;
    } else if let Some(float) = v.downcast_ref::<f32>() {
      env.call_method(
        self.native_view.lock().unwrap().as_obj(),
        "updateProp",
        "(Ljava/lang/String;F)V",
        &[
          JValue::Object(env.new_string(s).unwrap().into()),
          JValue::Float(*float),
        ],
      )?;
    } else if let Some(cb) = v.downcast_mut::<Option<Callback>>() {
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
      info!("COULDN'T UPDATE");
    }
    Ok(())
  }
  /// If you append a child that is attached somewhere else, you should move the child.
  fn append_child(&mut self, c: &PlatformView) -> Result<(), Box<dyn Error>> {
    let env = self.jvm.get_env()?;
    {
      let kind = c
        .get_raw_view()?
        .lock()
        .unwrap()
        .downcast_ref::<GlobalRef>()
        .expect("here");
      info!("Appending {} ", self.kind);
    }
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
