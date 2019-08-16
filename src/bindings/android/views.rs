use crate::ui_tree::{Composable, Composer, PlatformView, PlatformViewInner};
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
use jni::{JNIEnv, JavaVM};
use std::any::Any;
use std::cell::RefCell;
use std::error::Error;
use std::fmt::{self, Formatter};
use std::rc::Rc;
use {
  discard::DiscardOnDrop,
  futures::future::{BoxFuture, FutureExt},
};

pub struct ViewFactory {
  inner: GlobalRef,
  jvm: Rc<JavaVM>,
}

impl ViewFactory {
  pub fn new(inner: GlobalRef, jvm: JavaVM) -> Self {
    ViewFactory {
      inner,
      jvm: Rc::new(jvm),
    }
  }
}

thread_local! {
    pub static VIEWFACTORY: RefCell<Option<ViewFactory>> = RefCell::new(None);
}

pub struct Text {
  inner: PlatformView,
  // after_remove: Vec<BoxFuture>
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
        native_view: env.new_global_ref(native_view.l().unwrap()).unwrap(),
      };
      Text {
        inner: PlatformView::new(wired_native_view),
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

  pub fn text(mut self, s: String) -> Self {
    self.text_mut(s);
    self
  }

  pub fn text_signal(mut self, s: String) -> Self {
    self.text_mut(s);
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
        native_view: env.new_global_ref(native_view.l().unwrap()).unwrap(),
      };
      StackLayout {
        inner: PlatformView::new(underlying_view),
      }
    })
  }

  pub fn with<F>(self, composer: &mut Composer, f: F) -> Self
  where
    F: FnOnce(&mut Composer),
  {
    let last_parent = composer.curent_parent.take();
    composer.curent_parent = Some(self.inner);
    f(composer);

    let to_return = StackLayout {
      inner: composer.curent_parent.take().unwrap(),
    };

    composer.curent_parent = last_parent;
    to_return
  }

  pub(crate) fn get_native_view(&self) -> &GlobalRef {
    self
      .inner
      .get_raw_view()
      .unwrap()
      .downcast_ref::<GlobalRef>()
      .expect("Couldn't get raw view")
  }
}

struct WiredNativeView {
  kind: &'static str,
  jvm: Rc<JavaVM>,
  native_view: GlobalRef,
}

impl fmt::Debug for WiredNativeView {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "WiredNativeView [{}]", self.kind)
  }
}

impl PlatformViewInner for WiredNativeView {
  fn update_prop(&mut self, s: &str, v: Box<dyn Any>) -> Result<(), Box<dyn Error>> {
    let env = self.jvm.get_env()?;
    if let Some(string) = v.downcast_ref::<String>() {
      env.call_method(
        self.native_view.as_obj(),
        "updateProp",
        "(Ljava/lang/String;Ljava/lang/Object;)V",
        &[
          JValue::Object(env.new_string(s).unwrap().into()),
          JValue::Object(env.new_string(&string).unwrap().into()),
        ],
      )?;
    } else if let Some(float) = v.downcast_ref::<f32>() {
      env.call_method(
        self.native_view.as_obj(),
        "updateProp",
        "(Ljava/lang/String;F)V",
        &[
          JValue::Object(env.new_string(s).unwrap().into()),
          JValue::Float(*float),
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
      let kind = c.get_raw_view()?.downcast_ref::<GlobalRef>().expect("here");
      info!("Appending {} ", self.kind);
    }
    env.call_method(
      self.native_view.as_obj(),
      "appendChild",
      "(Ldev/fruit/androiddemo/WiredPlatformView;)V",
      &[JValue::Object(
        c.get_raw_view()?
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
      self.native_view.as_obj(),
      "insertChildAt",
      "(Ldev/fruit/androiddemo/WiredPlatformView;I)V",
      &[
        JValue::Object(
          c.get_raw_view()?
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
      self.native_view.as_obj(),
      "removeChild",
      "(Ldev/fruit/androiddemo/WiredPlatformView;)V",
      &[JValue::Object(
        c.get_raw_view()?
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
      self.native_view.as_obj(),
      "removeChild",
      "(I)V",
      &[JValue::Int(idx as i32)],
    )?;
    Ok(())
  }

  fn get_raw_view(&self) -> Result<&dyn Any, Box<dyn Error>> {
    Ok(&self.native_view)
  }
}
