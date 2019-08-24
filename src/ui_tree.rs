#![allow(dead_code)]
use discard::DiscardOnDrop;
use futures::executor::{LocalPool, LocalSpawner};
use futures::prelude::*;
use futures::task::LocalSpawnExt;
use futures_signals::{cancelable_future, CancelableFutureHandle};
use std::any::Any;
use std::cell::RefCell;
use std::mem;
use std::sync::Arc;

use futures_signals::signal::Mutable;
use std::error::Error;
use std::fmt::{Debug, Formatter};

thread_local! {
    #[allow(unused)]
    pub static EXECUTOR: RefCell<LocalPool> = RefCell::new(LocalPool::new());
    static SPAWNER: RefCell<LocalSpawner> = RefCell::new({
      EXECUTOR.with(|executor| {
        let executor = executor.borrow();
        let spawner = executor.spawner();
        spawner
      })
    });

    pub static COMPOSER: RefCell<Composer> = RefCell::new(Composer::new());

    // static COMPOSER: RefCell<Composer> = RefCell::new(Composer::new());
}

pub trait Composable {
  fn compose(&mut self, composer: &mut Composer);
}

use std::sync::Mutex;
#[derive(Clone)]
pub struct PlatformView {
  pub underlying_view: Arc<Mutex<dyn PlatformViewInner>>,
}

impl PlatformView {
  pub fn new<V>(underlying_view: V) -> Self
  where
    V: PlatformViewInner + 'static,
  {
    PlatformView {
      underlying_view: Arc::new(Mutex::new(underlying_view)),
    }
  }
}

// pub trait Prop: Debug + Any {}
pub trait PlatformViewInner: Debug + Send {
  fn update_prop(&mut self, s: &str, v: Box<dyn Any + Send>) -> Result<(), Box<dyn Error>>;
  /// If you append a child that is attached somewhere else, you should move the child.
  fn append_child(&mut self, c: &PlatformView) -> Result<(), Box<dyn Error>>;
  /// Do not insert a child that is already there! undefined behavior!
  fn insert_child_at(&mut self, c: &PlatformView, idx: usize) -> Result<(), Box<dyn Error>>;
  /// should not tear down the child! since it may be placed somewhere else later
  fn remove_child(&mut self, c: &PlatformView) -> Result<(), Box<dyn Error>>;
  /// Should not tear down the child (same as remove_child)
  fn remove_child_index(&mut self, idx: usize) -> Result<(), Box<dyn Error>>;
  fn get_raw_view(&self) -> Result<Arc<Mutex<dyn Any>>, Box<dyn Error>>;
}

// impl Prop for String {}
// impl Prop for f32 {}

fn spawn_local<F>(future: F)
where
  F: Future<Output = ()> + 'static,
{
  SPAWNER.with(move |s| {
    let mut spawner = s.borrow_mut();
    spawner.spawn_local(future).unwrap();
  });
}

pub fn with_parent<F>(parent: &mut PlatformView, f: F)
where
  F: FnOnce(),
{
  COMPOSER.with(|composer| {
    let mut composer = composer.borrow_mut();
    std::mem::swap(parent, composer.curent_parent.as_mut().unwrap());
  });

  f();

  COMPOSER.with(|composer| {
    let mut composer = composer.borrow_mut();
    std::mem::swap(parent, composer.curent_parent.as_mut().unwrap());
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
// Keep track of where we are, should be cheap to clone
#[derive(Clone, Debug)]
pub struct PositionContext {
  children_count_stack: Vec<Mutable<usize>>,
}

pub fn set_root_view(view: PlatformView) {
  COMPOSER.with(|c| {
    let mut composer = c.borrow_mut();
    composer.curent_parent.replace(view);
  })
}

pub fn swap_composer_with_active(other_composer: &mut Composer) {
  // Going to switch these
  COMPOSER.with(|c| {
    let mut active_composer = c.borrow_mut();
    mem::swap(other_composer, &mut active_composer);
  })
}

pub fn with_composer<F>(mut composer: Composer, f: F)
where
  F: FnOnce(),
{
  swap_composer_with_active(&mut composer);
  f();
  swap_composer_with_active(&mut composer);
}

fn current_idx() -> usize {
  COMPOSER.with(|c| {
    let active_composer = c.borrow();
    active_composer.position_context.get_current_idx()
  })
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

  pub(crate) fn push_new_frame(&mut self) {
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
pub enum Transaction {
  Add(usize),
}

#[derive(Clone)]
pub struct Composer {
  pub(crate) curent_parent: Option<PlatformView>,
  pub(crate) position_context: PositionContext,
  pub(crate) transactions: Vec<Transaction>,
  pub(crate) in_transaction: bool,
  pub(crate) transaction_start_idx: PositionContext,
}

impl Default for Composer {
  fn default() -> Self {
    Composer::new()
  }
}

impl Composer {
  pub fn new() -> Composer {
    Composer {
      curent_parent: None,
      position_context: PositionContext::new(),
      transactions: vec![],
      in_transaction: false,
      transaction_start_idx: PositionContext::new(),
    }
  }

  pub(crate) fn start_transaction(&mut self) {
    self.in_transaction = true;
  }

  pub(crate) fn rewind_transaction(&mut self) {
    if !self.transactions.is_empty() {
      debug!(
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

    let total_item_count = self.transactions.len();
    if !self.transactions.is_empty() {
      // self.position_context.dec();
    }

    self.transactions = vec![];
    for idx in 0..total_item_count {
      info!(
        "Removing view at {} + {}",
        self.transaction_start_idx.get_current_idx(),
        self.position_context.get_current_idx(),
      );

      self
        .remove_view_at(
          self.transaction_start_idx.get_current_idx() + self.position_context.get_current_idx()
            - 1,
        )
        .unwrap();
    }
  }

  pub(crate) fn end_transaction(&mut self) {
    self.in_transaction = false;
  }

  fn spawn<F: 'static + Future<Output = ()>>(&mut self, f: F) {
    SPAWNER.with(move |s| {
      let mut spawner = s.borrow_mut();
      spawner.spawn_local(f).unwrap();
    });
  }

  pub fn add_view(&mut self, view: &mut PlatformView) -> Result<(), Box<dyn Error>> {
    if let Some(mut curent_parent) = self.curent_parent.take() {
      let res = if self.in_transaction {
        debug!(
          "Inserting CHILD AT {} + {}",
          self.transaction_start_idx.get_current_idx(),
          self.position_context.get_current_idx()
        );
        curent_parent.insert_child_at(view, self.position_context.get_current_idx())
      } else {
        curent_parent.append_child(view)
      };

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
    debug!("Got parent");
    debug!("Calling remove child index {}", idx_to_remove);
    debug!(
      "Position context is {}",
      self.position_context.get_current_idx()
    );

    debug!("Calling remove child index");
    parent.remove_child_index(idx_to_remove)?;
    self.position_context.dec();
    debug!(
      "Removed child!! Position context is now {}",
      self.position_context.get_current_idx()
    );

    Ok(())
  }
}

impl Debug for PlatformView {
  fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
    write!(f, "{:?}", self.underlying_view.lock().unwrap())?;
    Ok(())
  }
}

impl PlatformViewInner for PlatformView {
  fn update_prop(&mut self, s: &str, v: Box<dyn Any + Send>) -> Result<(), Box<dyn Error>> {
    self.underlying_view.lock().unwrap().update_prop(s, v)
  }
  /// If you append a child that is attached somewhere else, you should move the child.
  fn append_child(&mut self, c: &PlatformView) -> Result<(), Box<dyn Error>> {
    self.underlying_view.lock().unwrap().append_child(c)
  }

  fn insert_child_at(&mut self, c: &PlatformView, idx: usize) -> Result<(), Box<dyn Error>> {
    self.underlying_view.lock().unwrap().insert_child_at(c, idx)
  }

  /// should not tear down the child! since it may be placed somewhere else later
  fn remove_child(&mut self, c: &PlatformView) -> Result<(), Box<dyn Error>> {
    self.underlying_view.lock().unwrap().remove_child(c)
  }
  /// Should not tear down the child (same as remove_child)
  fn remove_child_index(&mut self, idx: usize) -> Result<(), Box<dyn Error>> {
    self.underlying_view.lock().unwrap().remove_child_index(idx)
  }

  fn get_raw_view(&self) -> Result<Arc<Mutex<dyn Any>>, Box<dyn Error>> {
    // TODO fix
    let tmp = self.underlying_view.clone();
    let underlying_view = tmp.lock().unwrap();
    Ok(underlying_view.get_raw_view()?.clone())

    // let ptr = Arc::into_raw(underlying_view);
    // unsafe { (*ptr).lock().unwrap().get_raw_view() }
  }
}

// fn if_signal<S, F>(composer: &mut Composer, s: S, mut f: F)
// where
//   S: Signal<Item = bool> + 'static,
//   F: FnMut(&mut Composer, bool) + 'static,
// {
//   let mut current_composer_context = composer.clone();
//   current_composer_context.position_context.push_new_frame();
//   let mut transaction_start_idx = current_composer_context.position_context.clone();
//   mem::swap(&mut transaction_start_idx, &mut composer.position_context);
//   current_composer_context.transaction_start_idx = transaction_start_idx;

//   let fut = s.for_each(move |v: bool| {
//     current_composer_context.rewind_transaction();
//     current_composer_context.start_transaction();
//     f(&mut current_composer_context, v);
//     current_composer_context.end_transaction();
//     ready(())
//   });
//   // TODO fix
//   DiscardOnDrop::leak(spawn_future(fut));
// }

#[cfg(test)]
mod tests {
  use super::*;
  use crate::bindings::test::*;
  use crate::helpers::if_signal;
  use futures::future::ready;
  use futures_timer::{Delay, Interval};
  use std::panic::{catch_unwind, RefUnwindSafe, UnwindSafe};
  use std::rc::Rc;
  use std::time::Duration;

  use futures_signals::signal::{Mutable, Signal, SignalExt};
  use futures_signals::signal_vec::{MutableVec, SignalVecExt};

  #[test]
  fn check_button_presses() {
    set_root_view(DummyPlatformView::new("Root"));

    let my_state = Mutable::new(0);

    let my_state_clone = my_state.clone();
    let mut button = Button::new(move || {
      let mut lock = my_state_clone.lock_mut();
      *lock += 1;
    })
    .watch_label(my_state.signal().map(|n| format!("Counter is: {}", n)));

    let button_handle = button.handle();

    let root = StackLayout::new().with(move || {
      Text::new("Hello World");
      button;
    });

    // Press the button 3 times
    button_handle.press();
    button_handle.press();
    button_handle.press();

    EXECUTOR.with(|executor| {
      let mut executor = executor.borrow_mut();
      executor.run_until_stalled();
    });

    assert_eq!(*my_state.lock_ref(), 3);
    // assert_eq!(
    //   format!("{:?}", button.platform_view.underlying_view.lock().unwrap()),
    //   "Button View (props = [(\"label\", \"Counter is: 3\")])"
    // );
  }

  #[test]
  fn handle_removal() {
    let my_state = Mutable::new(true);

    let my_state_clone = my_state.clone();
    let my_state_clone2 = my_state.clone();
    let btn_press = move || {
      let mut lock = my_state_clone.lock_mut();
      *lock = false;
    };

    // let mut button = Button::new(btn_press);
    // let mut button = Button::new(|| {});
    // button.label("Press me to get rid of me!".into());

    let root = StackLayout::new().with(move || {
      Text::new("Hello World");
      if_signal(my_state.signal(), move |showing| {
        if showing {
          assert_eq!(current_idx(), 1);
          Text::new("Breaking news,");
          Text::new("It was true");
        } else {
          Text::new("It was not true");
        }
      });
      if_signal(my_state.signal(), move |showing| {
        if showing {
          Button::new(|| {}).label("Press me to get rid of me!".into());
        }
      });

      if_signal(my_state.signal(), move |showing| {
        if !showing {
          Text::new("This will only show if false");
        }
      });
    });

    assert_eq!(*my_state_clone2.lock_ref(), true);

    EXECUTOR.with(|executor| {
      let mut executor = executor.borrow_mut();
      executor.run_until_stalled();
    });

    assert_eq!(
      "StackLayout View (props = [])[\n    Text View (props = [(\"text\", \"Hello World\")]),\n    Text View (props = [(\"text\", \"Breaking news,\")]),\n    Text View (props = [(\"text\", \"It was true\")]),\n    Button View (props = [(\"label\", \"Press me to get rid of me!\")]),\n]",
      format!("{:?}", root.underlying_view)
    );
    assert_eq!(current_idx(), 4);

    btn_press();

    EXECUTOR.with(|executor| {
      let mut executor = executor.borrow_mut();
      executor.run_until_stalled();
    });

    println!("Root is {:?}", root.underlying_view);

    assert_eq!(*my_state_clone2.lock_ref(), false);
    assert_eq!(current_idx(), 3);
    assert_eq!(
      "StackLayout View (props = [])[\n    Text View (props = [(\"text\", \"Hello World\")]),\n    Text View (props = [(\"text\", \"It was not true\")]),\n    Text View (props = [(\"text\", \"This will only show if false\")]),\n]",
      format!("{:?}", root.underlying_view)
    );
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
      let root = StackLayout::new().with(|| {
        Text::new("Hello World");
        button;

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
        // button.press();
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

}
