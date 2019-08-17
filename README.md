# Wired

## Sample - Hello World

```rust
fn HelloWorld() {
  Text::new(Hello World)
}
```

## Sample - Presentation Slides

```rust
struct BasicSlideInfo {
  title: &'static str,
  reasons: Vec<&'static str>,
}

pub fn main() {
  let slide_number: Mutable<usize> = Mutable::new(0);
  let slide_sig = slide_number.signal();
  let slides: Vec<BasicSlideInfo> = build_slides();

  // Necesarry since we move this in the closures below
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
        BasicSlide(&slides[(slide_idx % slides.len())]);
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

fn BasicSlide(info: &BasicSlideInfo) {
  Text::new(info.title)
    .size(32.0)
    .pad_left(20.0)
    .pad_top(20.0);
  for reason in info.reasons.iter() {
    Text::new(*reason).size(20.0).pad_top(20.0).pad_left(20.0);
  }
}
```

### Setup (For cross platform)

[Source](https://medium.com/visly/rust-on-android-19f34a2fb43)

#### NDK Setup

(TODO this is no longer necessary you can use the toolchain directly)

```
mkdir ~/.NDK

$(ANDROID_HOME)/ndk-bundle/build/tools/make_standalone_toolchain.py --api 26 --arch arm64 --install-dir ~/.NDK/arm64;
$(ANDROID_HOME)/ndk-bundle/build/tools/make_standalone_toolchain.py --api 26 --arch arm --install-dir ~/.NDK/arm;
$(ANDROID_HOME)/ndk-bundle/build/tools/make_standalone_toolchain.py --api 26 --arch x86 --install-dir ~/.NDK/x86;
```

### Cargo config

(TODO can we move this to cargo.toml?)

add this to your `~/.cargo/config`

```
[target.aarch64-linux-android]
ar = ".NDK/arm64/bin/aarch64-linux-android-ar"
linker = ".NDK/arm64/bin/aarch64-linux-android-clang"

[target.armv7-linux-androideabi]
ar = ".NDK/arm/bin/arm-linux-androideabi-ar"
linker = ".NDK/arm/bin/arm-linux-androideabi-clang"

[target.i686-linux-android]
ar = ".NDK/x86/bin/i686-linux-android-ar"
linker = ".NDK/x86/bin/i686-linux-android-clang"

```
