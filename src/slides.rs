#![allow(dead_code)]
use crate::bindings::android::views::*;
use crate::bindings::view_helpers::*;
use crate::helpers::match_signal;
use crate::style::Orientation;
use futures_signals::signal::Mutable;

fn build_slides() -> Vec<BasicSlideInfo> {
  (vec![
    expand_slides(BasicSlideInfo {
      title: "Why",
      reasons: vec![
        " * React Native is ineffecient, where it matters. Mobile.",
        " * Keep it native, keep it sync, keep it simple",
        " * Zero Cost Abstractions",
        " * Rust is a nice environment",
        " * VDOM is work, no matter how \"fast\" it is",
        " * David vs Goliath; because I can!",
        " * Android - I had a week!",
      ],
    }),
    expand_slides(BasicSlideInfo {
      title: "How",
      reasons: vec![
        " * FRP/Observables backed by Rust Futures",
        " * JNI bindings to Java Views",
        " * Cross compile Rust to all Arm + linux combinations",
      ],
    }),
  ])
  .into_iter()
  .flatten()
  .collect()
}

pub fn main() {
  let slide_number: Mutable<usize> = Mutable::new(0);
  let slide_sig = slide_number.signal();
  let slides = build_slides();

  let slide_number_clone = slide_number.clone();
  let on_next = move || {
    let mut lock = slide_number_clone.lock_mut();
    *lock += 1;
  };
  let on_prev = move || {
    let mut lock = slide_number.lock_mut();
    if *lock > 0 {
      *lock -= 1;
    }
  };

  PhysicsLayout::new()
    .with(move || {
      match_signal(slide_sig, move |slide_idx| {
        basic_slide(&slides[(slide_idx % slides.len())]);
      });
      StackLayout::new()
        .with(|| {
          Button::new(on_prev).label("Previous");
          Button::new(on_next).label("Next");
        })
        .orientation(Orientation::Horizontal);
    })
    .orientation(Orientation::Vertical)
    .height(1820.0)
    .width(1080.0);
}

fn basic_slide(info: &BasicSlideInfo) {
  Text::new(info.title)
    .size(32.0)
    .pad_left(20.0)
    .pad_top(20.0);
  for reason in info.reasons.iter() {
    Text::new(*reason).size(20.0).pad_top(20.0).pad_left(20.0);
  }
}

// UTIL

fn expand_slides(info: BasicSlideInfo) -> Vec<BasicSlideInfo> {
  let title = info.title;
  let mut out = vec![];
  for i in 0..info.reasons.len() {
    out.push(BasicSlideInfo {
      title,
      reasons: info.reasons[0..i].into(),
    })
  }
  out
}

struct BasicSlideInfo {
  title: &'static str,
  reasons: Vec<&'static str>,
}
