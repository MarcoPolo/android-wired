use crate::bindings::callback::Callback;
use crate::style;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use std::error::Error;

pub trait UpdateProp<T> {
  fn update_prop(&mut self, k: &str, v: T) -> Result<(), Box<dyn Error>>;
}

pub trait UpdatePropSignal<T> {
  fn update_prop_signal<S>(&mut self, k: &'static str, s: S) -> Result<(), Box<dyn Error>>
  where
    S: 'static + Signal<Item = T> + Send;
}

macro_rules! prop_method {
    ($i:ident, $t:ty) => {
      fn $i(mut self, f: $t) -> Self {
        self
          .update_prop(stringify!($i), f)
          .expect(stringify!("Couldn't update", stringify($i)));
        self
      }
    };
}

macro_rules! prop_method_signal {
  ($i:ident, $t:ty) => {
    paste::item! {
      fn [<$i _signal>] <S>(mut self, s: S) -> Self
      where
        S: 'static + Signal<Item = $t> + Send {
        self
          .update_prop_signal(stringify!($i), s)
          .expect(stringify!("Couldn't update from signal: ", stringify($i)));
        self
      }
    }
  };
}

pub trait Padding: UpdateProp<f32> + UpdatePropSignal<f32> + Sized {
  prop_method_signal!(pad_left, f32);
  prop_method_signal!(pad_top, f32);
  prop_method_signal!(pad_right, f32);
  prop_method_signal!(pad_bottom, f32);

  prop_method!(pad_left, f32);
  prop_method!(pad_top, f32);
  prop_method!(pad_right, f32);
  prop_method!(pad_bottom, f32);
}

pub trait SetText: UpdateProp<String> + UpdatePropSignal<String> + Sized {
  prop_method!(text, String);
  prop_method_signal!(text, String);
}

pub trait SetTextSize: UpdateProp<f32> + UpdatePropSignal<f32> + Sized {
  prop_method!(text_size, f32);
  prop_method_signal!(text_size, f32);
}

pub trait SetXY: UpdateProp<f32> + UpdatePropSignal<f32> + Sized {
  prop_method!(set_x, f32);
  prop_method!(set_y, f32);

  prop_method_signal!(set_x, f32);
  prop_method_signal!(set_y, f32);
}

pub trait OnPress: UpdateProp<Callback> + UpdatePropSignal<Callback> + Sized {
  prop_method!(on_press, Callback);
  prop_method_signal!(on_press, Callback);
}

pub trait SetHeightWidth: UpdateProp<f32> + UpdatePropSignal<f32> + Sized {
  prop_method!(height, f32);
  prop_method!(width, f32);

  prop_method_signal!(height, f32);
  prop_method_signal!(width, f32);
}

pub trait SetOrientation: UpdateProp<String> + Sized {
  fn orientation(mut self, o: style::Orientation) -> Self {
    let string: String = o.to_string();
    self
      .update_prop("orientation", string)
      .expect("Couldn't update orientation");
    self
  }
}

pub trait ParentWith: Sized {
  fn with<F>(self, f: F) -> Self
  where
    F: FnOnce();
}
