//! The `eframe::NativeOptions` a rux application on Android starts with.

use eframe::egui_wgpu::{wgpu, WgpuSetup};
use std::sync::Arc;
use winit::platform::android::activity::AndroidApp;

/// Options for `eframe::run_native` on Android: the app handle, and a wgpu device any phone can create.
///
/// Two changes from eframe's defaults, each found by running on a device:
///
/// - **No wgpu debug flags.** wgpu turns on `InstanceFlags::DEBUG` by itself in a build with debug
///   assertions, which names every object it creates, and the emulator's Vulkan driver
///   (`vulkan.ranchu.so`) segfaults inside `vkSetDebugUtilsObjectNameEXT` the first time it is called.
/// - **Limits the adapter can give.** eframe asks for `wgpu::Limits::default()` on Vulkan, which includes
///   a 64 KiB uniform buffer binding. SwiftShader (the emulator's software Vulkan when it has no window)
///   allows 16 KiB, so device creation failed with "Limit 'max_uniform_buffer_binding_size' value 65536 is
///   better than allowed 16384" and the application exited before its first frame. egui needs none of
///   the larger defaults, so the downlevel set is asked for, with the texture size the adapter supports.
/// @param app - the `AndroidApp` `android_main` was given
/// @param title - the window title
pub fn native_options(app: AndroidApp, title: &str) -> eframe::NativeOptions {
    let mut options = eframe::NativeOptions {
        android_app: Some(app),
        viewport: egui::ViewportBuilder::default().with_title(title),
        ..Default::default()
    };
    if let WgpuSetup::CreateNew(setup) = &mut options.wgpu_options.wgpu_setup {
        setup.instance_descriptor.flags = wgpu::InstanceFlags::empty();
        setup.device_descriptor = Arc::new(|adapter| {
            let supported = adapter.limits();
            let limits = wgpu::Limits::downlevel_defaults().using_resolution(supported.clone());
            wgpu::DeviceDescriptor {
                label: Some("rux wgpu device"),
                required_limits: wgpu::Limits { max_texture_dimension_2d: supported.max_texture_dimension_2d.min(8192), ..limits },
                ..Default::default()
            }
        });
    }
    options
}
