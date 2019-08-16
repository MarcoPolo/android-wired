use crate::bindings::android::views::{Button, StackLayout, Text};
use crate::helpers::{if_signal};
use futures_signals::signal::{Mutable, Signal, SignalExt};

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
    Text::new("This is some other message").size(32.0);
  })
}
