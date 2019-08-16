use crate::bindings::android::views::{StackLayout, Text};
use crate::ui_tree::{Composable, Composer};

use futures_signals::signal::{Mutable, Signal, SignalExt};

pub fn main(mut composer: Composer) -> StackLayout {
  let text: Mutable<String> = Mutable::new("Hello from a signal!")
  StackLayout::new().with(&mut composer, move |composer| {
    Text::new("Hello World from WIRED")
      .size(32.0)
      .compose(composer);
    Text::default().text_signal(text.signal()).size(32.0).compose(composer);
    Text::new("This is some other message")
      .size(32.0)
      .compose(composer);
  })
}
