pub mod android_executor;
mod app;
pub mod bindings;
pub mod ui_tree;

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

use bindings::android;
use futures::future::ready;
use futures::prelude::*;
use futures_timer::{Delay, Interval};
use std::cell::RefCell;
use std::time::Duration;

use android_executor::spawn_future;
// use ui_tree::{spawn_future};

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
    // info!("Hello!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");

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

        // info!("Grabbing text view");
        // let native_text_view = env
        //     .call_method(
        //         view_factory.as_obj(),
        //         "createTextView",
        //         "()Ldev/fruit/androiddemo/WiredPlatformView;",
        //         &[],
        //     )
        //     .unwrap();

        let jvm = env.get_java_vm().unwrap();
        views::VIEWFACTORY.with(move |view_factory_ref| {
            *view_factory_ref.borrow_mut() = Some(views::ViewFactory::new(view_factory, jvm));
        });

        // env.call_method(
        //     native_text_view.l().unwrap(),
        //     "updateProp",
        //     "(Ljava/lang/String;Ljava/lang/Object;)V",
        //     &[
        //         JValue::Object(env.new_string("text").unwrap().into()),
        //         JValue::Object(env.new_string("Hello World").unwrap().into()),
        //     ],
        // )
        // .unwrap();

        // env.call_method(
        //     native_text_view.l().unwrap(),
        //     "updateProp",
        //     "(Ljava/lang/String;F)V",
        //     &[
        //         JValue::Object(env.new_string("text_size").unwrap().into()),
        //         JValue::Float(25.0),
        //     ],
        // )
        // .unwrap();

        // info!("got text view!");

        // env.call_method(
        //     root_View.as_obj(),
        //     "appendChild",
        //     "(Ldev/fruit/androiddemo/WiredPlatformView;)V",
        //     &[native_text_view],
        // )
        // .unwrap();

        // let native_text_view = env
        //     .new_global_ref(native_text_view.l().unwrap())
        //     .expect("Creating global ref should work");

        let composer = ui_tree::Composer::new();
        let app_root = app::main(composer);
        env.call_method(
            root_View.as_obj(),
            "appendChild",
            "(Ldev/fruit/androiddemo/WiredPlatformView;)V",
            &[JValue::Object(app_root.get_native_view().unwrap().as_obj())],
        )
        .unwrap();

        // let javavm = env.get_java_vm().unwrap();
        // let idx = Mutable::new(0);
        // let f = Interval::new(Duration::from_secs(1))
        //     .take(5)
        //     .for_each(move |_| {
        //         info!("Calling update prop here");
        //         let local_env = javavm.get_env().unwrap();
        //         {
        //             let mut lock = idx.lock_mut();
        //             *lock += 1;
        //         }
        //         let i = 0;
        //         local_env
        //             .call_method(
        //                 native_text_view.as_obj(),
        //                 "updateProp",
        //                 "(Ljava/lang/String;Ljava/lang/Object;)V",
        //                 &[
        //                     JValue::Object(local_env.new_string("text").unwrap().into()),
        //                     JValue::Object(
        //                         local_env
        //                             .new_string(format!("Hello World: {:?}", *idx.lock_ref()))
        //                             .unwrap()
        //                             .into(),
        //                     ),
        //                 ],
        //             )
        //             .unwrap();
        //         ready(())
        //     });

        // DiscardOnDrop::leak(spawn_future(f));
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
