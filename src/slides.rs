#![allow(dead_code)]
use crate::android_executor::spawn_future;
use crate::bindings::android::views::{Button, StackLayout, Text};
use crate::helpers::{if_signal, match_signal};
use discard::DiscardOnDrop;
use futures::future::ready;
use futures::prelude::*;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use futures_timer::{Delay, Interval};
use std::time::Duration;

#[derive(Copy, Clone)]
enum Slide {
  One,
  Two,
}

pub fn main() -> StackLayout {
  let slide_number: Mutable<Slide> = Mutable::new(Slide::One);
  let slide_sig = slide_number.signal();

  let slide_number_clone = slide_number.clone();
  let on_next = move || {
    let mut lock = slide_number_clone.lock_mut();
    *lock = lock.next();
  };
  let on_prev = move || {
    let mut lock = slide_number.lock_mut();
    *lock = lock.prev();
  };

  StackLayout::new().with(move || {
    match_signal(slide_sig, |slide| match slide {
      Slide::One => {
        Text::new("Hello Slide 1").size(32.0);
      }
      Slide::Two => {
        Text::new("Hello Slide 2").size(32.0);
      }
    });
    Button::new(on_prev).label("Previous");
    Button::new(on_next).label("Next");
  })
}

impl Slide {
  fn next(self) -> Slide {
    match self {
      Slide::One => Slide::Two,
      Slide::Two => Slide::Two,
    }
  }
  fn prev(self) -> Slide {
    match self {
      Slide::One => Slide::One,
      Slide::Two => Slide::One,
    }
  }
}
