#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

use crate::{config::Config, log};
use std::process::exit;

extern "C" fn breakdown_signal_handler(sig: i32) {
    let mut stack = vec![];
    backtrace::trace(|frame| {
        backtrace::resolve_frame(frame, |symbol| {
            if let Some(name) = symbol.name() {
                stack.push(name.to_string());
            }
        });
        true // keep going to the next frame
    });
    let mut info = String::default();
    if stack.iter().any(|s| {
        s.contains(&"nouveau_pushbuf_kick")
            || s.to_lowercase().contains("nvidia")
            || s.contains("gdk_window_end_draw_frame")
    }) {
        Config::set_option("allow-always-software-render".to_string(), "Y".to_string());
        info = "Always use software rendering will be set.".to_string();
        log::info!("{}", info);
    }
    log::error!(
        "Got signal {} and exit. stack:\n{}",
        sig,
        stack.join("\n").to_string()
    );
    if !info.is_empty() {
        #[cfg(target_os = "linux")]
        linux::system_message(
            "RustDesk",
            &format!("Got signal {} and exit.{}", sig, info),
            true,
        )
        .ok();
        #[cfg(target_os = "macos")]
        macos::alert(
            "RustDesk".to_owned(),
            "critical".to_owned(),
            "Crashed".to_owned(),
            format!("Got signal {} and exit.{}", sig, info),
            ["Ok".to_owned()].to_vec(),
        )
        .ok();
    }
    exit(0);
}

pub fn register_breakdown_handler() {
    unsafe {
        libc::signal(libc::SIGSEGV, breakdown_signal_handler as _);
    }
}
