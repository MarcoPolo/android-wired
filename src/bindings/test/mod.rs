use crate::ui_tree::{PlatformView, PlatformViewInner, with_parent, Composer, Composable, spawn_future};
use futures::future::ready;
use discard::DiscardOnDrop;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use std::any::Any;
use std::error::Error;
use futures_signals::signal::{Signal, SignalExt};
use futures_signals::{CancelableFutureHandle};

pub struct StackLayout {
  pub underlying_view: PlatformView,
}

impl StackLayout {
  pub fn new() -> Self {
    let underlying_view = DummyPlatformView::new("StackLayout");
    StackLayout { underlying_view }
  }
  pub fn with<F>(mut self, f: F) -> Self
  where
    F: FnOnce(),
  {
    with_parent(&mut self.underlying_view, f);
    self
  }
}

#[derive(Clone)]
pub struct DummyPlatformView {
  el_type: &'static str,
  props: Arc<Mutex<Vec<(String, Box<dyn Any + Send>)>>>,
  children: Vec<PlatformView>,
  raw_view: Arc<Mutex<dyn Any + Send>>,
}

impl DummyPlatformView {
  pub fn new(el_type: &'static str) -> PlatformView {
    PlatformView {
      underlying_view: Arc::new(Mutex::new(DummyPlatformView {
        el_type,
        props: Arc::new(Mutex::new(vec![])),
        children: vec![],
        raw_view: Arc::new(Mutex::new("dummy")),
      })),
    }
  }
}

impl Debug for DummyPlatformView {
  fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
    write!(
      f,
      "{} View (props = {:?})",
      self.el_type,
      self
        .props
        .lock()
        .unwrap()
        .iter()
        .map(|(k, v)| (
          k.clone(),
          v.downcast_ref::<String>().expect("Not a string").clone()
        ))
        .collect::<Vec<(String, String)>>()
    )?;
    if !self.children.is_empty() {
      write!(f, "{:#?}", self.children)?;
    }
    Ok(())
  }
}

pub struct Text {
  underlying_view: Option<PlatformView>,
}

impl Composable for Text {
  fn compose(&mut self, composer: &mut Composer) {
    if let Some(mut underlying_view) = self.underlying_view.take() {
      composer.add_view(&mut underlying_view).unwrap();
      self.underlying_view = Some(underlying_view);
    }
  }
}

impl Text {
  pub fn new<S: Into<String>>(text: S) -> Self {
    let mut t = Text {
      underlying_view: Some(DummyPlatformView::new("Text")),
    };
    t.underlying_view
      .as_mut()
      .unwrap()
      .update_prop("text", Box::new(text.into()))
      .unwrap();
    t
  }

  fn with_view(self, v: PlatformView) -> Text {
    Text {
      underlying_view: Some(v),
    }
  }

  fn with_view_mut(&mut self, v: PlatformView) {
    self.underlying_view = Some(v);
  }
}

impl PlatformViewInner for DummyPlatformView {
  fn update_prop(&mut self, s: &str, v: Box<dyn Any + Send>) -> Result<(), Box<dyn Error>> {
    println!("Updating {} on {:?} with {:?}", s, self, v);
    let mut props = self.props.lock().unwrap();
    if let Some(i) = props.iter().position(|(p, _)| p == s) {
      props[i] = (s.into(), v);
    } else {
      props.push((s.into(), v));
    }
    Ok(())
  }
  /// If you append a child that is attached somewhere else, you should move the child.
  fn append_child(&mut self, c: &PlatformView) -> Result<(), Box<dyn Error>> {
    println!("Appending Child {:?} to {:?}", c, self);
    self.children.push(c.clone());
    Ok(())
  }

  fn insert_child_at(&mut self, c: &PlatformView, idx: usize) -> Result<(), Box<dyn Error>> {
    println!("Appending Child {:?} to {:?} at idx: {}", c, self, idx);
    self.children.insert(idx, c.clone());
    Ok(())
  }

  /// should not tear down the child! since it may be placed somewhere else later
  fn remove_child(&mut self, c: &PlatformView) -> Result<(), Box<dyn Error>> {
    println!("Removing Child {:?} From {:?}", c, self);
    self.children = self
      .children
      .drain(..)
      .into_iter()
      .filter(|v| !Arc::ptr_eq(&v.underlying_view, &c.underlying_view))
      .collect();
    Ok(())
  }
  /// Should not tear down the child (same as remove_child)
  fn remove_child_index(&mut self, idx: usize) -> Result<(), Box<dyn Error>> {
    println!("Removing Child at {:?} From {:?}", idx, self);
    self.children.remove(idx);
    Ok(())
  }

  fn get_raw_view(&self) -> Result<Arc<Mutex<dyn Any>>, Box<dyn Error>> {
    Ok(self.raw_view.clone())
  }
}

pub struct Button<F> {
  label: Option<String>,
  on_press: Option<F>,
  platform_view: PlatformView,
  on_remove: Vec<DiscardOnDrop<CancelableFutureHandle>>,
}

pub struct ButtonHandle<F> {
  on_press: F,
}

impl<F> ButtonHandle<F>
where
  F: Fn(),
{
  pub fn press(&self) {
    (self.on_press)()
  }
}

impl<F> Button<F>
where
  F: Fn(),
{
  pub fn new(on_press: F) -> Self {
    Button {
      label: None,
      on_press: Some(on_press),
      platform_view: DummyPlatformView::new("Button"),
      on_remove: vec![],
    }
  }

  pub fn handle(&mut self) -> ButtonHandle<F> {
    ButtonHandle {
      on_press: self.on_press.take().unwrap(),
    }
  }

  pub fn watch_label<S>(mut self, s: S) -> Self
  where
    S: 'static + Signal<Item = String>,
  {
    let mut platform_view = self.platform_view.clone();
    let f = s.for_each(move |i| {
      platform_view
        .update_prop("label", Box::new(i.clone()))
        .expect("view is there");
      ready(())
    });

    self.on_remove.push(spawn_future(f));
    self
  }

  pub fn label(&mut self, label: String) {
    self
      .platform_view
      .update_prop("label", Box::new(label.clone()))
      .unwrap();
    self.label = Some(label)
  }
}

impl<F> Composable for Button<F> {
  fn compose(&mut self, composer: &mut Composer) {
    composer.add_view(&mut self.platform_view).unwrap();
  }
}
