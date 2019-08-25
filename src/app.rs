#![allow(dead_code)]
use crate::android_executor::spawn_future;
use crate::bindings::android::views::{Button, StackLayout, Text};
use crate::helpers::if_signal;
use discard::DiscardOnDrop;
use futures::future::ready;
use futures::prelude::*;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use futures_timer::{Delay, Interval};
use std::time::Duration;

pub fn main() -> StackLayout {
  let count: Mutable<i32> = Mutable::new(0);
  let count_handle = count.clone();
  let on_press = move || {
    let mut lock = count_handle.lock_mut();
    *lock += 1;
  };

  StackLayout::new().with(move || {
    Text::new("Hello WIRED").size(32.0);
    Button::new(on_press);
    Text::default()
      .text_signal(
        count
          .signal()
          .map(|i| format!("You've pressed it {} times", i)),
      )
      .size(32.0);
    if_signal(count.signal().map(|i| i % 2 == 0), |is_even| {
      if is_even {
        Text::new("Number is even!");
      }
    });
    marquee("WEEE").size(22.0);
    Text::new("This is some other message").size(22.0);
  })
}

fn marquee<S>(text: S) -> Text
where
  S: Into<String>,
{
  let count: Mutable<f32> = Mutable::new(0.0);
  let count_signal = count.signal();
  let f = Interval::new(Duration::from_millis(15))
    .take(900)
    .for_each(move |_| {
      let mut lock = count.lock_mut();
      *lock = (*lock + 20.0) % 1400.0;
      ready(())
    });

  let cancel = spawn_future(f);
  DiscardOnDrop::leak(cancel);

  Text::new(text).x_pos_signal(count_signal.map(|f| f - 200.0))
}
