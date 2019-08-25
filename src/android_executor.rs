#![allow(dead_code)]
use {
  discard::DiscardOnDrop,
  futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
  },
  futures_signals::{cancelable_future, CancelableFutureHandle},
  jni::{
    objects::{JClass, JObject, JString, JValue},
    JNIEnv,
  },
  std::{
    cell::RefCell,
    future::Future,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    sync::{Arc, Mutex, MutexGuard},
    task::{Context, Poll},
  },
};

thread_local! {
    static SPAWNER: RefCell<Option<Spawner>> = RefCell::new(None);
}

/// A future that can reschedule itself to be polled by an `Executor`.
struct Task {
  /// In-progress future that should be pushed to completion.
  ///
  /// The `Mutex` is not necessary for correctness, since we only have
  /// one thread executing tasks at once. However, Rust isn't smart
  /// enough to know that `future` is only mutated from one thread,
  /// so we need use the `Mutex` to prove thread-safety. A production
  /// executor would not need this, and could use `UnsafeCell` instead.
  future: Mutex<Option<BoxFuture<'static, ()>>>,

  /// Handle to place the task itself back onto the task queue.
  task_sender: SyncSender<Arc<Task>>,
}

impl ArcWake for Task {
  fn wake_by_ref(arc_self: &Arc<Self>) {
    // Implement `wake` by sending this task back onto the task channel
    // so that it will be polled again by the executor.
    let cloned = arc_self.clone();
    arc_self
      .task_sender
      .send(cloned)
      .expect("too many tasks queued");
  }
}

pub struct AndroidExecutor {
  ready_queue: Receiver<Arc<Task>>,
  staging_future: Mutex<Option<BoxFuture<'static, ()>>>,
  staging_task: Mutex<Option<Arc<Task>>>,
}

#[no_mangle]
pub unsafe extern "C" fn Java_dev_fruit_androiddemo_Executor_setup(
  env: JNIEnv,
  _class: JClass,
  executor_ref: JObject,
) {
  let (executor, spawner) = new_executor_and_spawner();
  env.set_rust_field(executor_ref, "ptr", executor).unwrap();

  SPAWNER.with(|s| {
    s.borrow_mut().replace(spawner);
  });
}

#[no_mangle]
pub unsafe extern "C" fn Java_dev_fruit_androiddemo_Executor_recv(
  env: JNIEnv,
  _class: JClass,
  executor_ref: JObject,
) {
  let mut executor: MutexGuard<AndroidExecutor> = env.get_rust_field(executor_ref, "ptr").unwrap();
  info!("Waiting for task");

  if let Ok(task) = executor.ready_queue.recv() {
    info!("Got task!");
    executor.staging_task = Mutex::new(Some(task));
  } else {
    panic!("Bail")
  }
}

#[no_mangle]
pub unsafe extern "C" fn Java_dev_fruit_androiddemo_Executor_poll(
  env: JNIEnv,
  _class: JClass,

  executor_ref: JObject,
) {
  info!("Starting poll");
  let executor: MutexGuard<AndroidExecutor> = env.get_rust_field(executor_ref, "ptr").unwrap();
  let task = executor.staging_task.lock().unwrap().take();
  if let Some(task) = task {
    let mut future_slot = task.future.lock().unwrap();
    if let Some(mut future) = future_slot.take() {
      info!("Got future");
      // Create a `LocalWaker` from the task itself
      let waker = waker_ref(&task);
      let context = &mut Context::from_waker(&*waker);
      // `BoxFuture<T>` is a type alias for
      // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
      // We can get a `Pin<&mut dyn Future + Send + 'static>`
      // from it by calling the `Pin::as_mut` method.
      if let Poll::Pending = future.as_mut().poll(context) {
        info!("Was pending");
        // We're not done processing the future, so put it
        // back in its task to be run again in the future.
        *future_slot = Some(future);
      } else {
        info!("Was ready");
      }
    }
  }
}

impl AndroidExecutor {
  fn new() -> Self {
    panic!("todo")
  }

  fn run(&self) {
    while let Ok(task) = self.ready_queue.recv() {
      // Take the future, and if it has not yet completed (is still Some),
      // poll it in an attempt to complete it.
      let mut future_slot = task.future.lock().unwrap();
      if let Some(mut future) = future_slot.take() {
        // Create a `LocalWaker` from the task itself
        let waker = waker_ref(&task);
        let context = &mut Context::from_waker(&*waker);
        // `BoxFuture<T>` is a type alias for
        // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
        // We can get a `Pin<&mut dyn Future + Send + 'static>`
        // from it by calling the `Pin::as_mut` method.
        if let Poll::Pending = future.as_mut().poll(context) {
          // We're not done processing the future, so put it
          // back in its task to be run again in the future.
          *future_slot = Some(future);
        }
      }
    }
  }
}

/// `Spawner` spawns new futures onto the task channel.
#[derive(Clone)]
struct Spawner {
  task_sender: SyncSender<Arc<Task>>,
}

fn new_executor_and_spawner() -> (AndroidExecutor, Spawner) {
  // Maximum number of tasks to allow queueing in the channel at once.
  // This is just to make `sync_channel` happy, and wouldn't be present in
  // a real executor.
  const MAX_QUEUED_TASKS: usize = 10_000;
  let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
  (
    AndroidExecutor {
      ready_queue,
      staging_future: Mutex::new(None),
      staging_task: Mutex::new(None),
    },
    Spawner { task_sender },
  )
}

impl Spawner {
  fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
    let future = future.boxed();
    let task = Arc::new(Task {
      future: Mutex::new(Some(future)),
      task_sender: self.task_sender.clone(),
    });
    debug!("running task first");
    let mut already_done = false;
    {
      let mut future_slot = task.future.lock().unwrap();
      if let Some(mut future) = future_slot.take() {
        // Create a `LocalWaker` from the task itself
        let waker = waker_ref(&task);
        let context = &mut Context::from_waker(&*waker);
        // `BoxFuture<T>` is a type alias for
        // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
        // We can get a `Pin<&mut dyn Future + Send + 'static>`
        // from it by calling the `Pin::as_mut` method.
        if let Poll::Pending = future.as_mut().poll(context) {
          // We're not done processing the future, so put it
          // back in its task to be run again in the future.
          *future_slot = Some(future);
        } else {
          already_done = true;
        }
      }
    }
    if !already_done {
      self.task_sender.send(task).expect("too many tasks queued");
    }
  }
}

fn spawn_local(future: impl Future<Output = ()> + 'static + Send) {
  SPAWNER.with(move |spawner| {
    if let Some(spawner) = spawner.borrow_mut().as_mut() {
      spawner.spawn(future);
    }
  });
}

#[inline]
pub(crate) fn spawn_future<F>(future: F) -> DiscardOnDrop<CancelableFutureHandle>
where
  F: Future<Output = ()> + 'static + Send,
{
  // TODO make this more efficient ?
  let (handle, future) = cancelable_future(future, || ());

  spawn_local(future);

  handle
}
