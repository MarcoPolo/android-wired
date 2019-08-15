#![allow(dead_code)]
#![allow(unused_variables)]
use discard::DiscardOnDrop;
/// Scratch pad to think about how the api should look like
use futures::executor::{LocalPool, LocalSpawner};
use futures::prelude::*;
use futures::task::LocalSpawnExt;
use futures_signals::{cancelable_future, CancelableFutureHandle};
use futures_timer::{Delay, Interval};
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use std::time::Duration;

use futures::future::ready;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use futures_signals::signal_vec::{MutableVec, SignalVecExt};
use std::any::Any;
use std::error::Error;
use std::fmt::Debug;

thread_local! {
    #[allow(unused)]
    static EXECUTOR: RefCell<LocalPool> = RefCell::new(LocalPool::new());
    static SPAWNER: RefCell<LocalSpawner> = RefCell::new({
      EXECUTOR.with(|executor| {
        let executor = executor.borrow();
        let spawner = executor.spawner();
        spawner
      })
    });

    // static COMPOSER: RefCell<Composer> = RefCell::new(Composer::new());
}

trait Composable {
  fn compose(&mut self, composer: &mut Composer);
}

#[derive(Clone, Debug)]
struct PlatformView {
  underlying_view: Rc<dyn PlatformViewInner>,
  is_node: bool,
}

trait Prop: Debug + Any {}
trait PlatformViewInner: Debug {
  fn update_prop(&self, s: &str, v: Box<dyn Prop>) -> Result<(), Box<dyn Error>>;
  /// If you append a child that is attached somewhere else, you should move the child.
  fn append_child(&self, c: &dyn PlatformViewInner) -> Result<(), Box<dyn Error>>;
  /// Do not insert a child that is already there! undefined behavior!
  fn insert_child_at(&self, c: &dyn PlatformViewInner, idx: usize) -> Result<(), Box<dyn Error>>;
  /// should not tear down the child! since it may be placed somewhere else later
  fn remove_child(&self, c: &dyn PlatformViewInner) -> Result<(), Box<dyn Error>>;
  /// Should not tear down the child (same as remove_child)
  fn remove_child_index(&self, idx: usize) -> Result<(), Box<dyn Error>>;
}

impl Prop for String {}

fn spawn_local<F>(future: F)
where
  F: Future<Output = ()> + 'static,
{
  SPAWNER.with(move |s| {
    let mut spawner = s.borrow_mut();
    spawner.spawn_local(future).unwrap();
  });
}

#[inline]
pub(crate) fn spawn_future<F>(future: F) -> DiscardOnDrop<CancelableFutureHandle>
where
  F: Future<Output = ()> + 'static,
{
  // TODO make this more efficient ?
  let (handle, future) = cancelable_future(future, || ());

  spawn_local(future);

  handle
}

struct Button<F> {
  label: Option<String>,
  on_press: F,
  platform_view: PlatformView,
  on_remove: Vec<DiscardOnDrop<CancelableFutureHandle>>,
}

impl<F> Button<F>
where
  F: Fn(),
{
  fn new(on_press: F) -> Self {
    Button {
      label: None,
      on_press,
      platform_view: DummyPlatformView::new("Button"),
      on_remove: vec![],
    }
  }

  fn watch_label<S>(mut self, s: S) -> Self
  where
    S: 'static + Signal<Item = String>,
  {
    let platform_view = self.platform_view.clone();
    let f = s.for_each(move |i| {
      platform_view
        .underlying_view
        .update_prop("label", Box::new(i.clone()))
        .expect("view is there");
      ready(())
    });

    self.on_remove.push(spawn_future(f));
    self
  }

  fn label(&mut self, label: String) {
    self.label = Some(label)
  }

  fn press(&self) {
    (self.on_press)();
  }
}

impl<F> Composable for Button<F> {
  fn compose(&mut self, composer: &mut Composer) {
    composer.add_view(&mut self.platform_view).unwrap();
  }
}

struct Text {
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
  fn new(text: String) -> Self {
    let t = Text {
      underlying_view: Some(DummyPlatformView::new("Text")),
    };
    t.underlying_view
      .as_ref()
      .unwrap()
      .underlying_view
      .update_prop("text", Box::new(text))
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

// Keep track of where we are, should be cheap to clone
#[derive(Clone, Debug)]
struct PositionContext {
  children_count_stack: Vec<Mutable<usize>>,
}

impl PositionContext {
  fn new() -> PositionContext {
    PositionContext {
      children_count_stack: vec![Mutable::new(0)],
    }
  }

  fn snapshot_context(&mut self) -> PositionContext {
    let snapshot = self.clone();
    self.children_count_stack.push(Mutable::new(0));
    snapshot
  }

  fn push_new_frame(&mut self) {
    self.children_count_stack.push(Mutable::new(0));
  }

  fn get_current_idx(&self) -> usize {
    self
      .children_count_stack
      .iter()
      .fold(0, |acc, v| acc + v.read_only().get())
  }

  fn inc(&mut self) {
    let mut lock = self.children_count_stack[self.children_count_stack.len() - 1].lock_mut();
    *lock += 1;
  }

  fn dec(&mut self) {
    let mut lock = self.children_count_stack[self.children_count_stack.len() - 1].lock_mut();
    *lock -= 1;
  }
}

#[derive(Clone, Debug)]
enum Transaction {
  Add(usize),
}

#[derive(Clone)]
struct Composer {
  curent_parent: Option<PlatformView>,
  position_context: PositionContext,
  transactions: Vec<Transaction>,
  in_transaction: bool,
  transaction_start_idx: PositionContext,
}

impl Composer {
  fn new() -> Composer {
    Composer {
      curent_parent: None,
      position_context: PositionContext::new(),
      transactions: vec![],
      in_transaction: false,
      transaction_start_idx: PositionContext::new(),
    }
  }

  fn start_transaction(&mut self) {
    self.in_transaction = true;
  }

  fn rewind_transaction(&mut self) {
    if !self.transactions.is_empty() {
      println!(
        "Trying to rewind these transactions: {:?}",
        self.transactions
      );
    }

    let removals: Vec<usize> = self
      .transactions
      .iter()
      .filter_map(|t| match t {
        Transaction::Add(idx) => Some(*idx),
      })
      .collect();

    for idx in removals {
      self
        .remove_view_at(self.transaction_start_idx.get_current_idx() + idx)
        .unwrap();
    }
  }

  fn end_transaction(&mut self) {
    self.in_transaction = false;
  }

  fn spawn<F: 'static + Future<Output = ()>>(&mut self, f: F) {
    SPAWNER.with(move |s| {
      let mut spawner = s.borrow_mut();
      spawner.spawn_local(f).unwrap();
    });
  }
  // fn start(&mut self, parent_view: &mut Box<dyn PlatformView>) {}
  // fn end(&mut self) {
  // self.curent_parent = self.parent_stack.pop();
  // }
  // fn with_parent<F: FnMut(&mut Box<dyn PlatformView>)>(&mut self, f: &mut F) {
  //   if let Some(curent_parent) = self.curent_parent.take() {
  //     f(curent_parent);
  //     self.curent_parent = Some(curent_parent);
  //   }
  // }

  fn add_view(&mut self, view: &mut PlatformView) -> Result<(), Box<dyn Error>> {
    if let Some(curent_parent) = self.curent_parent.take() {
      let res = if self.in_transaction {
        curent_parent.underlying_view.insert_child_at(
          &*view.underlying_view,
          self.position_context.get_current_idx(),
        )
      } else {
        curent_parent
          .underlying_view
          .append_child(&*view.underlying_view)
      };
      if !self.transactions.is_empty() {
        let current_idx = self.position_context.get_current_idx();
        // If we are modifying something before a recorded transaction, let's shift the transaction
        self.transactions = self
          .transactions
          .iter()
          .map(|t| match t {
            Transaction::Add(idx) => {
              if *idx > current_idx {
                Transaction::Add(idx + 1)
              } else {
                Transaction::Add(*idx)
              }
            }
          })
          .collect();
      }
      if self.in_transaction {
        self.transactions.push(Transaction::Add(
          self.position_context.get_current_idx() - self.transaction_start_idx.get_current_idx(),
        ));
      }
      self.position_context.inc();
      self.curent_parent = Some(curent_parent);
      return res;
    }
    Ok(())
  }

  fn remove_view_at(&mut self, idx_to_remove: usize) -> Result<(), Box<dyn Error>> {
    let parent = self
      .curent_parent
      .as_mut()
      .expect("A parent is set to work on");
    if !self.transactions.is_empty() {
      self.transactions = self
        .transactions
        .iter()
        .map(|t| match t {
          Transaction::Add(idx) => {
            if *idx > idx_to_remove {
              Transaction::Add(idx - 1)
            } else {
              Transaction::Add(*idx)
            }
          }
        })
        .collect();
    }
    parent.underlying_view.remove_child_index(idx_to_remove)?;
    self.position_context.dec();

    Ok(())
  }
}

struct StackLayout {
  underlying_view: PlatformView,
}

impl<'a> StackLayout {
  fn new() -> Self {
    let mut underlying_view = DummyPlatformView::new("StackLayout");
    underlying_view.is_node = true;
    StackLayout { underlying_view }
  }
  fn with<F>(self, composer: &mut Composer, f: F) -> Self
  where
    F: FnOnce(&mut Composer),
  {
    let last_parent = composer.curent_parent.take();
    composer.curent_parent = Some(self.underlying_view);
    f(composer);

    let to_return = StackLayout {
      underlying_view: composer.curent_parent.take().unwrap(),
    };

    composer.curent_parent = last_parent;
    to_return
  }
}

#[derive(Clone)]
struct DummyPlatformView {
  el_type: &'static str,
  props: Rc<RefCell<Vec<(String, Box<dyn Prop>)>>>,
  children: Vec<Rc<Box<dyn PlatformViewInner>>>,
}

impl DummyPlatformView {
  fn new(el_type: &'static str) -> PlatformView {
    PlatformView {
      underlying_view: Rc::new(DummyPlatformView {
        el_type,
        props: Rc::new(RefCell::new(vec![])),
        children: vec![],
      }),
      is_node: false,
    }
  }
}

use std::fmt::Formatter;
impl Debug for DummyPlatformView {
  fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
    write!(f, "{} View {{{:?}}}", self.el_type, self.props.borrow())
  }
}

impl PlatformViewInner for DummyPlatformView {
  fn update_prop(&self, s: &str, v: Box<dyn Prop>) -> Result<(), Box<dyn Error>> {
    println!("Updating {} on {:?} with {:?}", s, self, v);
    let mut props = self.props.borrow_mut();
    if let Some(i) = props.iter().position(|(p, _)| p == s) {
      props[i] = (s.into(), v);
    } else {
      props.push((s.into(), v));
    }
    Ok(())
  }
  /// If you append a child that is attached somewhere else, you should move the child.
  fn append_child(&self, c: &dyn PlatformViewInner) -> Result<(), Box<dyn Error>> {
    println!("Appending Child {:?} to {:?}", c, self);
    Ok(())
  }

  fn insert_child_at(&self, c: &dyn PlatformViewInner, idx: usize) -> Result<(), Box<dyn Error>> {
    println!("Appending Child {:?} to {:?} at idx: {}", c, self, idx);
    Ok(())
  }

  /// should not tear down the child! since it may be placed somewhere else later
  fn remove_child(&self, c: &dyn PlatformViewInner) -> Result<(), Box<dyn Error>> {
    println!("Removing Child {:?} From {:?}", c, self);
    Ok(())
  }
  /// Should not tear down the child (same as remove_child)
  fn remove_child_index(&self, idx: usize) -> Result<(), Box<dyn Error>> {
    println!("Removing Child at {:?} From {:?}", idx, self);
    Ok(())
  }
}

pub fn demo() {
  let mut composer = Composer::new();

  let my_state = Mutable::new(5);

  let my_state_clone = my_state.clone();
  let mut button = Button::new(move || {
    let mut lock = my_state_clone.lock_mut();
    *lock += 1;
  })
  .watch_label(my_state.signal().map(|n| format!("Counter is: {}", n)));

  // Style A
  {
    let root = StackLayout::new().with(&mut composer, |composer| {
      Text::new("Hello World".into()).compose(composer);
      button.compose(composer);

      let my_vec: MutableVec<u32> = MutableVec::new_with_values(vec![1, 2, 3]);
      let f = my_vec.signal_vec().for_each(|change| {
        println!("Here in signal vec with {:?}", change);
        // Text::new(format!("Hello no.{}!", i)).compose(composer);
        ready(())
      });
      composer.spawn(Box::new(f));
    });
  }

  let my_state_clone = my_state.clone();
  let f = Interval::new(Duration::from_secs(1))
    .take(5)
    .for_each(move |_| {
      println!("Pressing Button");
      button.press();
      ready(())
    });

  let cancel = spawn_future(f);

  EXECUTOR.with(|executor| {
    let mut executor = executor.borrow_mut();
    executor
      .run_until(Delay::new(Duration::from_secs(5)))
      .unwrap();
  })

  // Style B
  // {
  //   let root = StackLayout::new().with(&mut composer, |composer| {
  //     Text::new("Hello World".into()).compose(composer);
  //     let mut button = Button::new(|| {
  //       counter_state.set(counter_state.get() + 1);
  //     });
  //     button.watch_label_mut(&counter_state.map(|n| format!("Counter is: {}", n)));
  //     button.compose(composer);
  //   });
  // }

  // Style c
  // {
  //   let root = StackLayout!(
  //     Text!(name = "Foo")
  //   )
  // }
}

fn if_signal<S, F>(composer: &mut Composer, s: S, mut f: F)
where
  S: Signal<Item = bool> + 'static,
  F: FnMut(&mut Composer, bool) + 'static,
{
  let mut current_composer_context = composer.clone();
  current_composer_context.position_context.push_new_frame();
  let mut transaction_start_idx = current_composer_context.position_context.clone();
  mem::swap(&mut transaction_start_idx, &mut composer.position_context);
  current_composer_context.transaction_start_idx = transaction_start_idx;

  let fut = s.for_each(move |v: bool| {
    current_composer_context.rewind_transaction();
    current_composer_context.start_transaction();
    f(&mut current_composer_context, v);
    current_composer_context.end_transaction();
    ready(())
  });
  // TODO fix
  DiscardOnDrop::leak(spawn_future(fut));
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn check_button_presses() {
    let mut composer = Composer::new();

    let my_state = Mutable::new(0);

    let my_state_clone = my_state.clone();
    let mut button = Button::new(move || {
      let mut lock = my_state_clone.lock_mut();
      *lock += 1;
    })
    .watch_label(my_state.signal().map(|n| format!("Counter is: {}", n)));

    let root = StackLayout::new().with(&mut composer, |composer| {
      Text::new("Hello World".into()).compose(composer);
      button.compose(composer);
    });

    // Press the button 3 times
    button.press();
    button.press();
    button.press();

    EXECUTOR.with(|executor| {
      let mut executor = executor.borrow_mut();
      executor.run_until_stalled();
    });

    assert_eq!(*my_state.lock_ref(), 3);
    assert_eq!(
      format!("{:?}", button.platform_view.underlying_view),
      "Button View {[(\"label\", \"Counter is: 3\")]}"
    );
  }

  #[test]
  fn handle_removal() {
    let mut composer = Composer::new();

    let my_state = Mutable::new(true);

    let my_state_clone = my_state.clone();
    let my_state_clone2 = my_state.clone();
    let btn_press = move || {
      let mut lock = my_state_clone.lock_mut();
      *lock = false;
    };

    // let mut button = Button::new(btn_press);
    let mut button = Button::new(|| {});
    button.label("Press me to get rid of me!".into());

    let root = StackLayout::new().with(&mut composer, move |composer| {
      Text::new("Hello World".into()).compose(composer);
      if_signal(composer, my_state.signal(), move |composer, showing| {
        if showing {
          assert_eq!(composer.position_context.get_current_idx(), 1);
          Text::new("Breaking news,".into()).compose(composer);
          Text::new("It was true".into()).compose(composer);
        } else {
          Text::new("It was not true".into()).compose(composer);
        }
      });
      if_signal(composer, my_state.signal(), move |composer, showing| {
        if showing {
          button.compose(composer);
        }
      });

      if_signal(composer, my_state.signal(), move |composer, showing| {
        if !showing {
          Text::new("This will only show if false".into()).compose(composer);
        }
      });
    });

    assert_eq!(*my_state_clone2.lock_ref(), true);

    EXECUTOR.with(|executor| {
      let mut executor = executor.borrow_mut();
      executor.run_until_stalled();
    });

    println!("{:?}", composer.position_context);
    println!("tx {:?}", composer.transactions);
    assert_eq!(composer.position_context.get_current_idx(), 4);

    btn_press();

    EXECUTOR.with(|executor| {
      let mut executor = executor.borrow_mut();
      executor.run_until_stalled();
    });

    println!("Root is {:?}", root.underlying_view);

    assert_eq!(*my_state_clone2.lock_ref(), false);
    assert_eq!(composer.position_context.get_current_idx(), 3);

    // assert_eq!(
    //   format!("{:?}", button.platform_view.underlying_view),
    //   "Button View {[(\"label\", \"Counter is: 3\")]}"
    // );
  }

}
