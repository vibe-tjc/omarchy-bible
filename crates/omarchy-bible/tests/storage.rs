//! Android settings use the platform's private app storage, not HOME.
use nsh_core::prelude::*;
use nsh_engine::{EngineConfig, Root};
use nsh_testing::{Headless, MemoryStorage};
use omarchy_bible::settings::{AppSettings, ThemePreference};
use omarchy_bible::store::SettingsFile;

#[component]
fn StorageProbe() -> impl IntoElement {
    let settings = signal(SettingsFile::AppStorage.load());
    view()
        .child(text(move || {
            format!("font:{}", settings.with(|s| s.font_size))
        }))
        .child(
            nsh_base::button()
                .on_press(move |_: &()| {
                    let next = AppSettings {
                        font_size: 24,
                        theme: ThemePreference::Light,
                        ..AppSettings::default()
                    };
                    SettingsFile::AppStorage.save(&next);
                    settings.set(SettingsFile::AppStorage.load());
                })
                .child(text("Save")),
        )
}

#[test]
fn app_storage_survives_engine_restart() -> Result<(), String> {
    let storage = MemoryStorage::new([]);
    {
        let mut app = Headless::start(EngineConfig {
            storage: Some(storage.clone()),
            ..EngineConfig::new(Root::of::<StorageProbe>())
        })
        .unwrap();
        app.wait_for("text:font:18")?;
        app.press("Save")?;
        app.wait_for("text:font:24")?;
    }
    let saved: AppSettings = serde_json::from_str(&storage.entries()["settings"]).unwrap();
    assert_eq!(saved.theme, ThemePreference::Light);
    let mut app = Headless::start(EngineConfig {
        storage: Some(storage),
        ..EngineConfig::new(Root::of::<StorageProbe>())
    })
    .unwrap();
    app.wait_for("text:font:24")?;
    Ok(())
}

#[test]
fn invalid_stored_settings_fall_back_to_defaults() -> Result<(), String> {
    let mut app = Headless::start(EngineConfig {
        storage: Some(MemoryStorage::new([(
            "settings".into(),
            "invalid JSON".into(),
        )])),
        ..EngineConfig::new(Root::of::<StorageProbe>())
    })
    .unwrap();
    app.wait_for("text:font:18")?;
    Ok(())
}
