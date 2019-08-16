use crate::bindings::android::views::{Button, StackLayout, Text};
use crate::ui_tree::{Composable, Composer};
use std::rc::Rc;

use futures_signals::signal::{Mutable, Signal, SignalExt};

pub fn main(mut composer: Composer) -> StackLayout {
  let count: Mutable<i32> = Mutable::new(0);
  let count_handle = count.clone();
  let on_press = move || {
    let mut lock = count_handle.lock_mut();
    *lock += 1;
  };

  StackLayout::new().with(&mut composer, move |composer| {
    Text::new("Hello WIRED").size(32.0).compose(composer);
    Button::new(on_press).compose(composer);
    Text::default()
      .text_signal(
        count
          .signal()
          .map(|i| format!("You've pressed it {} times", i)),
      )
      .size(32.0)
      .compose(composer);
    Text::new("This is some other message")
      .size(32.0)
      .compose(composer);
  })
}
