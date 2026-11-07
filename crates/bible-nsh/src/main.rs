//! Runs the reader on the machine's native backend (GTK 4 on Linux,
//! AppKit on macOS).

#[cfg(target_os = "linux")]
use nsh_platform_linux as platform;
#[cfg(target_os = "macos")]
use nsh_platform_macos as platform;

/// The application id: `nsh.json` `"id"`, GTK's and Wayland's.
#[cfg(target_os = "linux")]
const APP_ID: &str = "io.github.vibe_tjc.OmarchyBible";

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn main() -> impl std::process::Termination {
    env_logger::init();
    platform::run_app(
        platform::AppSource::Rust(platform::Root::of::<bible_nsh::App>()),
        platform::AppOptions {
            title: "聖經".to_owned(),
            // GTK needs a reverse-DNS id; it is also the Wayland app id
            // (the .desktop entry's StartupWMClass).
            #[cfg(target_os = "linux")]
            app_id: APP_ID.to_owned(),
            size: (980, 760),
            ..Default::default()
        },
    )
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
compile_error!("omarchy-bible runs on Linux and macOS.");
