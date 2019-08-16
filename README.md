# Wired

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
