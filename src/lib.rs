pub mod android_executor;
mod app;
pub mod bindings;
pub mod helpers;
pub mod ui_tree;

mod slides;

#[macro_use]
extern crate log;
#[cfg(target_os = "android")]
extern crate android_logger;
#[cfg(target_os = "android")]
use android_logger::Config;

#[cfg(target_os = "android")]
use bindings::android::views;

use discard::DiscardOnDrop;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use jni::errors::Error as JNIError;
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue};
use jni::sys::jstring;
use jni::{JNIEnv, JavaVM};
use log::Level;
use std::ffi::{CStr, CString};
use std::panic::{catch_unwind, RefUnwindSafe, UnwindSafe};

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use std::sync::Arc;

use bindings::android;
use futures::future::ready;
use futures::prelude::*;
use futures_timer::{Delay, Interval};
use std::cell::RefCell;
use std::time::Duration;

use android_executor::spawn_future;
use ui_tree::PlatformView;

#[no_mangle]
pub unsafe extern "C" fn Java_dev_fruit_androiddemo_MainActivity_hello(
    env: JNIEnv,
    _: JObject,
    j_recipient: JString,
) -> jstring {
    let recipient = CString::from(CStr::from_ptr(
        env.get_string(j_recipient).unwrap().as_ptr(),
    ));

    let output = env
        .new_string("Hello ".to_owned() + recipient.to_str().unwrap())
        .unwrap();

    output.into_inner()
}

#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern "C" fn Java_dev_fruit_androiddemo_MainActivity_init(
    env: JNIEnv,
    _class: JClass,
    view_factory: JObject,
    root_view: JObject,
) {
    android_logger::init_once(
        Config::default()
            .with_min_level(Level::Trace)
            .with_tag("io.marcopolo.wired"),
    );
    info!("Started init");

    let result = catch_unwind(move || {
        let view_factory = env
            .new_global_ref(view_factory)
            .expect("Creating global ref should work");

        let root_View = env
            .new_global_ref(root_view)
            .expect("Creating global ref should work");

        let jvm = Arc::new(env.get_java_vm().unwrap());
        let jvm_clone = jvm.clone();
        views::VIEWFACTORY.with(move |view_factory_ref| {
            *view_factory_ref.borrow_mut() = Some(views::ViewFactory::new(view_factory, jvm));
        });

        let root_View = android::views::WiredNativeView {
            kind: "StackLayout",
            jvm: jvm_clone,
            native_view: android::views::wrap_native_view(root_View),
        };
        ui_tree::set_root_view(PlatformView::new(root_View));
        let app_root = slides::main();

        // env.call_method(
        //     root_View.as_obj(),
        //     "appendChild",
        //     "(Ldev/fruit/androiddemo/WiredPlatformView;)V",
        //     &[JValue::Object(app_root.get_native_view().unwrap().as_obj())],
        // )
        // .unwrap();
    });

    match result {
        Err(cause) => {
            info!("I failed");
            if let Some(reason) = cause.downcast_ref::<Box<dyn Error>>() {
                info!("Failed because err: {}", reason);
            }
            if let Some(reason) = cause.downcast_ref::<std::cell::BorrowError>() {
                info!("Failed because borrow err {}", reason);
            }
            if let Some(reason) = cause.downcast_ref::<std::cell::BorrowMutError>() {
                info!("Failed because borrow mut err {}", reason);
            }
            if let Some(reason) = cause.downcast_ref::<String>() {
                info!("Failed because str: {}", reason);
            }
            if let Some(reason) = cause.downcast_ref::<Box<dyn Display>>() {
                info!("Failed because {}", reason);
            }
        }
        _ => {}
    }
}
