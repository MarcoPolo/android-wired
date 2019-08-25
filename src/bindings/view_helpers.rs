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
    ($i:ident) => {
      fn $i(mut self, f: f32) -> Self {
        self
          .update_prop(stringify!($i), f)
          .expect(stringify!("Couldn't update", stringify($i)));
        self
      }
    };
}

macro_rules! prop_method_signal {
    ($i:ident) => {
      paste::item! {
        fn [<$i _signal>] <S>(mut self, s: S) -> Self
        where
          S: 'static + Signal<Item = f32> + Send {
          self
            .update_prop_signal(stringify!($i), s)
            .expect(stringify!("Couldn't update from signal: ", stringify($i)));
          self
        }
      }
    };
}

pub trait Padding: UpdateProp<f32> + UpdatePropSignal<f32> + Sized {
  // trace_macros!(true);
  prop_method_signal!(pad_left);
  // trace_macros!(false);
  prop_method_signal!(pad_top);
  prop_method_signal!(pad_right);
  prop_method_signal!(pad_bottom);

  prop_method!(pad_left);
  prop_method!(pad_top);
  prop_method!(pad_right);
  prop_method!(pad_bottom);

  // fn pad(self, left: f32, top: f32, right: f32, bottom: f32) -> Self {
  //   self
  //     .update_prop_4("pad_bottom", (left, top, right, bottom))
  //     .expect("Couldn't update bottom padding");
  //   self
  // }

  // fn pad_signal<S>(self, s: S) -> Self
  // where
  //   S: 'static + Signal<Item = (f32, f32, f32, f32)> + Send {
  //   self
  //     .update_prop_4_signal("pad", s)
  //     .expect("Couldn't update padding signal");
  //   self
  // }
}