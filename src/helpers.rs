#[cfg(target_os = "android")]
use crate::android_executor::spawn_future;
#[cfg(not(target_os = "android"))]
use crate::ui_tree::spawn_future;
use crate::ui_tree::{Composer, COMPOSER};
use discard::DiscardOnDrop;
use futures::future::ready;
use futures_signals::signal::{Mutable, ReadOnlyMutable, Signal, SignalExt};

pub fn if_signal<S, F>(s: S, f: F)
where
  S: Signal<Item = bool> + Send + 'static,
  F: Fn(bool) + Send + 'static,
{
  match_signal(s, f);
}

pub fn match_signal<S, F, M>(s: S, f: F)
where
  S: Signal<Item = M> + Send + 'static,
  F: Fn(M) + Send + 'static,
  M: 'static,
{
  let mut current_composer_context: Composer = COMPOSER.with(|c| {
    let mut composer = c.borrow_mut();
    let mut current_composer_context = composer.clone();
    let mut transaction_start_idx = current_composer_context.position_context.clone();
    std::mem::swap(&mut transaction_start_idx, &mut composer.position_context);
    composer.position_context.push_new_frame();
    current_composer_context.transaction_start_idx = transaction_start_idx;
    current_composer_context
  });

  let fut = s.for_each(move |v: M| {
    // let current_composer_context = current_composer_context.as_mut().unwrap();
    current_composer_context.rewind_transaction();
    current_composer_context.start_transaction();

    COMPOSER.with(|c| {
      {
        let mut composer = c.borrow_mut();
        std::mem::swap(&mut *composer, &mut current_composer_context);
      }
      f(v);
      {
        let mut composer = c.borrow_mut();
        std::mem::swap(&mut *composer, &mut current_composer_context);
      }
      current_composer_context.end_transaction();
    });

    ready(())
  });

  // TODO fix
  DiscardOnDrop::leak(spawn_future(fut));
}

pub struct ReadOnlyState<T>(ReadOnlyMutable<T>);
impl<T: Copy> ReadOnlyState<T> {
  pub fn get(&self) -> T {
    self.0.get()
  }
}

pub fn use_state<S>(initial_state: S) -> (ReadOnlyMutable<S>, impl Fn(S)) {
  let state = Mutable::new(initial_state);
  (state.read_only(), move |next_state: S| {
    let mut lock = state.lock_mut();
    *lock = next_state;
  })
}

pub fn use_state_reducer<S, R>(initial_state: S) -> (ReadOnlyMutable<S>, impl Fn(R))
where
  R: Fn(&S) -> S,
{
  let state = Mutable::new(initial_state);
  (state.read_only(), move |next_state_reducer: R| {
    let next_state = next_state_reducer(&state.lock_ref());
    let mut lock = state.lock_mut();
    *lock = next_state;
  })
}
