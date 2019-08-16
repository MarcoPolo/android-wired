use crate::android_executor::spawn_future;
use crate::ui_tree::{Composable, Composer, PlatformView, PlatformViewInner, COMPOSER};
use discard::DiscardOnDrop;
use futures::future::ready;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use std::borrow::BorrowMut;

pub fn if_signal<S, F>(s: S, mut f: F)
where
  S: Signal<Item = bool> + Send + 'static,
  F: Fn(bool) + Send + 'static,
{
  let mut current_composer_context: Composer = COMPOSER.with(|c| {
    info!("HERE!!!! grabbing current context");

    let mut composer = c.borrow_mut();
    let mut current_composer_context = composer.clone();
    current_composer_context.position_context.push_new_frame();
    let mut transaction_start_idx = current_composer_context.position_context.clone();
    std::mem::swap(&mut transaction_start_idx, &mut composer.position_context);
    current_composer_context.transaction_start_idx = transaction_start_idx;
    info!("HERE!!!! grabbed current context");
    current_composer_context
  });

  let fut = s.for_each(move |v: bool| {
    // let current_composer_context = current_composer_context.as_mut().unwrap();
    info!("HERE!!!! rewinding transaction");
    current_composer_context.rewind_transaction();
    info!("HERE!!!! starting transaction");
    current_composer_context.start_transaction();
    info!("HERE!!!! started transaction");

    COMPOSER.with(|c| {
      {
        let mut composer = c.borrow_mut();
        info!("HERE!!!! swapping composer");
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
