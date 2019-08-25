#!/bin/sh
JNI_LIBS=./android/app/src/main/jniLibs

cargo build --target i686-linux-android # --release

exit_status=$?
if [ $exit_status -eq 0 ]; then
  rm -rf $JNI_LIBS
  mkdir -p $JNI_LIBS

  mkdir -p $JNI_LIBS/arm64-v8a
  mkdir -p $JNI_LIBS/armeabi-v7a
  mkdir -p $JNI_LIBS/x86

  cp target/i686-linux-android/debug/librust.so $JNI_LIBS/x86/librust.so

  adb -e shell 'am force-stop dev.fruit.androiddemo'
  adb -e shell 'su root mkdir /data/data/dev.fruit.androiddemo/lib'
  adb -e push target/i686-linux-android/debug/librust.so /sdcard/librust.so
  adb -e shell 'su root mv /sdcard/librust.so /data/data/dev.fruit.androiddemo/lib/librust.so'
  adb -e shell 'su root chown system.system /data/data/dev.fruit.androiddemo/lib/librust.so'
  adb -e shell 'su root chmod a+rx /data/data/dev.fruit.androiddemo/lib/librust.so'
  # adb -e push target/i686-linux-android/debug/librust.so /data/data/dev.fruit.androiddemo/lib/librust.so
  adb -e shell 'am start -n dev.fruit.androiddemo/.MainActivity'
fi
exit $exit_status