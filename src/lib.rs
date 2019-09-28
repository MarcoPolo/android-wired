// #![feature(trace_macros)]
#![allow(dead_code, unused_imports)]
pub mod ui_tree;

#[macro_use]
mod macros {
    macro_rules! auto_compose {
        ($e:ty) => {
            impl Drop for $e {
                fn drop(&mut self) {
                    crate::ui_tree::COMPOSER.with(|c| {
                        let mut c = c.borrow_mut();
                        crate::ui_tree::Composable::compose(self, &mut c)
                    })
                }
            }
        };
    }

    macro_rules! auto_compose_T {
        ($e:ty) => {
            impl<T> Drop for $e {
                fn drop(&mut self) {
                    crate::ui_tree::COMPOSER.with(|c| {
                        let mut c = c.borrow_mut();
                        crate::ui_tree::Composable::compose(self, &mut c)
                    })
                }
            }
        };
    }
}

pub mod android_executor;
mod app;
pub mod bindings;
pub mod helpers;
pub mod style;

mod slides;

#[macro_use]
extern crate update_prop_derive;

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
use std::ops::Deref;

use std::sync::Arc;

use bindings::android;
use futures::future::ready;
use futures::prelude::*;
use futures_timer::{Delay, Interval};
use std::cell::RefCell;
use std::time::Duration;

use android_executor::spawn_future;
use ui_tree::PlatformView;

use jni_android_sys::android::content::Context;
use jni_android_sys::android::view::KeyEvent;
use jni_android_sys::android::view::View;
use jni_android_sys::android::view::ViewGroup;
use jni_android_sys::android::widget::TextView;
use jni_glue::{jchar, Argument, AsJValue, Env};
use jni_sys::{jboolean, jobject, JNI_TRUE};

#[no_mangle]
pub extern "system" fn Java_dev_fruit_androiddemo_MainActivity_dispatchKeyEvent(
    env: &Env,
    _this: jobject,
    key_event: Argument<KeyEvent>,
) -> jboolean {
    let key_event = unsafe { key_event.with_unchecked(env) }; // Unsafe boilerplate not yet autogenerated.

    // Err = Java exception was thrown.
    // Ok(None) = Java object is null.
    // Ok(Some(...)) = Real java object!
    if let Some(key_event) = key_event {
        let is_enter = if let Ok(r) = key_event.getKeyCode() {
            r == KeyEvent::KEYCODE_ENTER
        } else {
            false
        };
        let is_down = if let Ok(r) = key_event.getAction() {
            r == KeyEvent::ACTION_DOWN
        } else {
            false
        };
        if is_enter && is_down {
            println!("ENTER pressed"); // Not that you can see this...
        }
    }

    JNI_TRUE // JNI boilerplate not yet autogenerated
}

#[no_mangle]
pub unsafe extern "system" fn Java_dev_fruit_androiddemo_MainActivity_something(
    env: &Env,
    _this: JObject,
    ctx: Argument<Context>,
    root_view: Argument<ViewGroup>,
) {
    let ctx = ctx.with_unchecked(env); // Unsafe boilerplate not yet autogenerated.
    let root_view = root_view.with_unchecked(env); // Unsafe boilerplate not yet autogenerated.

    let ctx = ctx.unwrap();
    let root_view = root_view.unwrap();

    let text_view = TextView::new_Context(env, Some(&*ctx)).expect("Create textview");
    let chars: Vec<jchar> = "Foooooooooo"
        .as_bytes()
        .iter()
        .map(|b| jchar(*b as u16))
        .collect();
    let char_array: jni_glue::Local<jni_glue::CharArray> =
        jni_glue::PrimitiveArray::from(env, &chars);
    text_view
        .setText_char_array_int_int(Some(&*char_array), 0, chars.len() as i32)
        .expect("Set text");
    let text_view = jni_glue::Local::leak(text_view);
    let text_view_view: &View = &text_view;
    root_view.addView_View(text_view_view).expect("Should work");
}

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

        let root_view = env
            .new_global_ref(root_view)
            .expect("Creating global ref should work");

        let jvm = Arc::new(env.get_java_vm().unwrap());
        let jvm_clone = jvm.clone();
        views::VIEWFACTORY.with(move |view_factory_ref| {
            *view_factory_ref.borrow_mut() = Some(views::ViewFactory::new(view_factory, jvm));
        });

        let root_view = android::views::WiredNativeView {
            kind: "StackLayout",
            jvm: jvm_clone,
            native_view: android::views::wrap_native_view(root_view),
        };
        ui_tree::set_root_view(PlatformView::new(root_view));
        let _app_root = slides::main();

        // env.call_method(
        //     root_view.as_obj(),
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
