use {
  discard::DiscardOnDrop,
  futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
  },
  futures_signals::{cancelable_future, CancelableFutureHandle},
  jni::{
    objects::{GlobalRef, JClass, JObject, JString, JValue},
    JNIEnv, JavaVM,
  },
  std::{
    cell::RefCell,
    future::Future,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    sync::{Arc, Mutex, MutexGuard},
    task::{Context, Poll},
    time::Duration,
  },
};

pub struct Callback {
  pub(crate) f: Arc<Box<dyn Fn() + Send + Sync>>,
}

#[no_mangle]
pub unsafe extern "C" fn Java_dev_fruit_androiddemo_RustCallback_call(
  env: JNIEnv,
  _class: JClass,
  callback_ref: JObject,
) {
  let callback: MutexGuard<Callback> = env.get_rust_field(callback_ref, "ptr").unwrap();
  debug!("Calling callback!");
  (callback.f)()
}
