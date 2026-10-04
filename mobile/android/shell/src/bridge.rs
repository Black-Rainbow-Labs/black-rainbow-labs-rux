//! The JNI half: the activity, `RuxBridge`'s static methods, and the one native method Kotlin calls.
//!
//! Every call goes through [`with_activity`], which attaches the calling thread, opens a local reference
//! frame and clears any Java exception. The frame matters: the Rust thread that draws the application is
//! attached for its whole life, so a local reference made once a frame and never freed would fill the
//! JNI local reference table within minutes.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

use jni::objects::{GlobalRef, JClass, JIntArray, JObject, JValue};
use jni::{JNIEnv, JavaVM, NativeMethod};
use winit::platform::android::activity::AndroidApp;

use crate::Edges;

/// The application handle `android_main` was given.
static APP: OnceLock<AndroidApp> = OnceLock::new();
/// The egui context [`native_wake`] asks for a frame.
static CTX: OnceLock<egui::Context> = OnceLock::new();
/// `com.blackrainbowlabs.rux.RuxBridge`, loaded through the activity's class loader.
static BRIDGE: OnceLock<GlobalRef> = OnceLock::new();
/// The last value sent to `setHandlesBack`: 0 unknown, 1 off, 2 on. Saves a JNI call a frame.
static HANDLES_BACK: AtomicU8 = AtomicU8::new(0);

const BRIDGE_CLASS: &str = "com.blackrainbowlabs.rux.RuxBridge";

/// Connects the Rust side to `RuxBridge`. Call once, from the eframe creation closure.
///
/// It keeps the app and the context, loads the bridge class, registers `nativeWake` and only then tells
/// Kotlin it may call it.
/// @param app - the `AndroidApp` `android_main` was given
/// @param ctx - the egui context to wake when Kotlin changes something
pub fn install(app: &AndroidApp, ctx: &egui::Context) -> Result<(), String> {
    let _ = APP.set(app.clone());
    let _ = CTX.set(ctx.clone());
    let class = with_activity(|env, activity| {
        let class = load_class(env, activity, BRIDGE_CLASS)?;
        let wake = NativeMethod { name: "nativeWake".into(), sig: "()V".into(), fn_ptr: native_wake as *mut std::ffi::c_void };
        env.register_native_methods(<&JClass>::from(class.as_obj()), &[wake])?;
        env.call_static_method(<&JClass>::from(class.as_obj()), "setNativeReady", "(Z)V", &[JValue::Bool(1)])?;
        Ok(class)
    })?;
    let _ = BRIDGE.set(class);
    Ok(())
}

/// The `AndroidApp` [`install`] was given, if it has been called.
pub(crate) fn app() -> Option<AndroidApp> {
    APP.get().cloned()
}

/// Runs `f` with a JNI environment and the activity object, inside a local reference frame.
///
/// A Java exception thrown inside `f` is described to the log and cleared, and comes back as an error,
/// so a failed call never leaves the thread unable to make the next one.
/// @param f - the work, given the environment and the activity
pub fn with_activity<R>(f: impl FnOnce(&mut JNIEnv, &JObject) -> jni::errors::Result<R>) -> Result<R, String> {
    let app = APP.get().ok_or("rux_android::install has not been called")?;
    // SAFETY: both pointers come from android-activity and live as long as the activity does.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.map_err(|e| e.to_string())?;
    let mut env = vm.attach_current_thread_permanently().map_err(|e| e.to_string())?;
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
    let result = env.with_local_frame(16, |env| f(env, &activity));
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_describe();
        let _ = env.exception_clear();
    }
    result.map_err(|e| e.to_string())
}

/// Loads an application class by name through the activity's class loader.
///
/// `FindClass` on a thread that native code attached sees only the system classes, so an application's
/// own Kotlin has to be reached through the loader that loaded the activity.
/// @param env - the JNI environment
/// @param activity - the activity object
/// @param name - the class name with dots, such as `com.example.Thing`
pub fn load_class(env: &mut JNIEnv, activity: &JObject, name: &str) -> jni::errors::Result<GlobalRef> {
    let loader = env.call_method(activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?.l()?;
    let name = env.new_string(name)?;
    let class = env.call_method(&loader, "loadClass", "(Ljava/lang/String;)Ljava/lang/Class;", &[JValue::Object(&name)])?.l()?;
    env.new_global_ref(class)
}

/// Reads `RuxBridge.insets()` and converts it from pixels to points.
/// @param pixels_per_point - egui's scale
pub(crate) fn read_edges(pixels_per_point: f32) -> Option<Edges> {
    let class = BRIDGE.get()?;
    let pixels = with_activity(|env, _| {
        let array = env.call_static_method(<&JClass>::from(class.as_obj()), "insets", "()[I", &[])?.l()?;
        let mut values = [0i32; 5];
        env.get_int_array_region(&JIntArray::from(array), 0, &mut values)?;
        Ok(values)
    })
    .map_err(|problem| log::warn!("reading RuxBridge.insets failed: {problem}"))
    .ok()?;
    let scale = pixels_per_point.max(0.01);
    let [top, right, bottom, left, keyboard] = pixels.map(|value| value.max(0) as f32 / scale);
    Some(Edges { safe: rux::Insets::new(top, right, bottom, left), keyboard })
}

/// The keyboard editor's text as `RuxActivity.stateChanged` last recorded it.
pub(crate) fn keyboard_text() -> Option<String> {
    let class = BRIDGE.get()?;
    with_activity(|env, _| {
        let text = env.call_static_method(<&JClass>::from(class.as_obj()), "keyboardText", "()Ljava/lang/String;", &[])?.l()?;
        let text: String = env.get_string(&text.into())?.into();
        Ok(text)
    })
    .ok()
}

/// How many times the keyboard's action key was pressed since the last call.
pub(crate) fn take_editor_actions() -> u32 {
    call_count("takeEditorActions")
}

/// Empties the keyboard's editor, here and on the UI thread.
pub(crate) fn clear_keyboard() {
    let Some(class) = BRIDGE.get() else { return };
    let _ = with_activity(|env, _| {
        env.call_static_method(<&JClass>::from(class.as_obj()), "clearKeyboard", "()V", &[])?;
        Ok(())
    });
}

/// Calls a `RuxBridge` method that returns a count and resets it.
/// @param method - the static method name, signature `()I`
fn call_count(method: &str) -> u32 {
    let Some(class) = BRIDGE.get() else { return 0 };
    with_activity(|env, _| env.call_static_method(<&JClass>::from(class.as_obj()), method, "()I", &[])?.i())
        .map(|count| count.max(0) as u32)
        .unwrap_or(0)
}

/// How many back gestures Kotlin counted since the last call.
pub(crate) fn take_back_presses() -> u32 {
    call_count("takeBackPresses")
}

/// Tells Kotlin whether the next back gesture belongs to the application. Only calls JNI on a change.
/// @param handles - true while the application has a screen to go back from
pub fn set_handles_back(handles: bool) {
    let wanted = if handles { 2 } else { 1 };
    if HANDLES_BACK.swap(wanted, Ordering::Relaxed) == wanted {
        return;
    }
    let Some(class) = BRIDGE.get() else { return };
    let result = with_activity(|env, _| {
        env.call_static_method(<&JClass>::from(class.as_obj()), "setHandlesBack", "(Z)V", &[JValue::Bool(handles as u8)])?;
        Ok(())
    });
    if let Err(problem) = result {
        HANDLES_BACK.store(0, Ordering::Relaxed);
        log::warn!("RuxBridge.setHandlesBack failed: {problem}");
    }
}

/// The last value sent to `setLightBars`: 0 unknown, 1 dark theme, 2 light theme.
static LIGHT_BARS: AtomicU8 = AtomicU8::new(0);

/// Makes the system bar icons readable on the theme's ground. Only calls JNI on a change.
/// @param light - true for a light theme
pub fn set_light_bars(light: bool) {
    let wanted = if light { 2 } else { 1 };
    if LIGHT_BARS.swap(wanted, Ordering::Relaxed) == wanted {
        return;
    }
    let Some(class) = BRIDGE.get() else {
        LIGHT_BARS.store(0, Ordering::Relaxed);
        return;
    };
    let result = with_activity(|env, _| {
        env.call_static_method(<&JClass>::from(class.as_obj()), "setLightBars", "(Z)V", &[JValue::Bool(light as u8)])?;
        Ok(())
    });
    if result.is_err() {
        LIGHT_BARS.store(0, Ordering::Relaxed);
    }
}

/// `RuxBridge.nativeWake`: Kotlin changed something, so draw a frame.
extern "system" fn native_wake(_env: JNIEnv, _class: JClass) {
    if let Some(ctx) = CTX.get() {
        ctx.request_repaint();
    }
}
