// Modified by Gesoft, 2026-10-02 and 2026-11-11. See CHANGES-GESOFT.md.
// One build serves the live and the lab helpdesk; the download name selects the
// profile (`src/gesoft_profile.rs`), and the helpdesk checks the build's profile
// marker before serving it.

use std::process::{Command, Stdio};

#[path = "../../../src/gesoft_profile.rs"]
mod gesoft_profile;

pub use gesoft_profile::{Launch, Profile};

/// What this download was served for, from its name alone.
pub fn launch() -> Launch {
    let Some(path) = std::env::current_exe().ok() else {
        return Launch::Missing;
    };
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return Launch::Missing;
    };
    gesoft_profile::launch(name)
}

/// The lab profile runs only where `lab.conf` is in place; checked before
/// anything is extracted, so the customer sees one message, not RustDesk's
/// and the close run's.
fn check_endpoints(profile: &'static Profile) -> Result<(), String> {
    gesoft_profile::endpoints(profile).map(|_| ()).map_err(|error| {
        native_windows_gui::simple_message("Gesoft Support", &error);
        error
    })
}

pub fn missing_session() -> Result<(), String> {
    let error = "Lipsesc detaliile sesiunii. Descărcați din nou clientul din pagina de suport.";
    native_windows_gui::simple_message("Gesoft Support", error);
    Err(error.to_string())
}

/// The technician's exe: RustDesk for outgoing connections, extracted into the
/// profile's directory (which selects the profile in RustDesk) and branded from
/// the profile's helpdesk when it answers.
pub fn run_technician(profile: &'static Profile) -> Result<(), String> {
    check_endpoints(profile)?;
    let reader = super::BinaryReader::new()?;
    let local = dirs::data_local_dir()
        .ok_or("LocalAppData unavailable")?
        .join(profile.dir);
    let no_progress_window = vec!["--gesoft-support".to_owned()];
    let mut ui = false;
    let payload = super::setup(
        reader,
        Some(local.clone()),
        false,
        &no_progress_window,
        &mut ui,
    )
    .ok_or("Could not extract RustDesk")?;
    Command::new(&payload)
        .env("RUSTDESK_APPNAME", "GesoftSupport.exe")
        .env("GESOFT_SUPPORT_TECHNICIAN", "1")
        .env_remove("GESOFT_SUPPORT_CODE")
        .env("RUSTDESK_BRAND_DIR", local.join("branding-technician"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn run(code: &str, profile: &'static Profile) -> Result<(), String> {
    check_endpoints(profile)?;
    let result = run_session(code, profile);
    if let Err(error) = &result {
        native_windows_gui::simple_message(
            "Gesoft Support",
            &format!("Sesiunea nu a putut porni.\n{error}"),
        );
    }
    result
}

fn run_session(code: &str, profile: &'static Profile) -> Result<(), String> {
    let reader = super::BinaryReader::new()?;
    let local = dirs::data_local_dir()
        .ok_or("LocalAppData unavailable")?
        .join(profile.dir);
    let local_existed = local.exists();
    let roaming = dirs::config_dir()
        .ok_or("AppData unavailable")?
        .join(profile.namespace);
    let roaming_existed = roaming.exists();
    let mut ui = false;
    // setup shows its separate extraction window for empty args. This
    // support handoff keeps the customer's first visible window in RustDesk.
    let no_progress_window = vec!["--gesoft-support".to_owned()];
    let payload = super::setup(
        reader,
        Some(local.clone()),
        false,
        &no_progress_window,
        &mut ui,
    )
    .ok_or("Could not extract RustDesk")?;
    let brand_dir = local.join(format!("branding-{}", std::process::id()));
    std::fs::create_dir_all(&brand_dir).map_err(|e| e.to_string())?;
    let mut child = Command::new(&payload)
        .env("RUSTDESK_APPNAME", "GesoftSupport.exe")
        .env("GESOFT_SUPPORT_CODE", code)
        .env_remove("GESOFT_SUPPORT_TECHNICIAN")
        .env("RUSTDESK_BRAND_DIR", &brand_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    child.wait().map_err(|e| e.to_string())?;
    // The payload carries the HTTP client; with this argument it reports the
    // close and exits before any UI. Done before cleanup so the technician
    // learns at once instead of after the silence timeout.
    let _ = Command::new(&payload)
        .arg("--gesoft-support-closed")
        .env("RUSTDESK_APPNAME", "GesoftSupport.exe")
        .env("GESOFT_SUPPORT_CODE", code)
        .env_remove("GESOFT_SUPPORT_TECHNICIAN")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let mut cleanup_errors = Vec::new();
    if let Err(error) = remove_dir_with_retry(&brand_dir) {
        cleanup_errors.push(format!("{}: {error}", brand_dir.display()));
    }
    for (preexisting, path) in [(local_existed, local), (roaming_existed, roaming)] {
        if !preexisting && path.exists() {
            if let Err(error) = remove_dir_with_retry(&path) {
                cleanup_errors.push(format!("{}: {error}", path.display()));
            }
        }
    }
    if !cleanup_errors.is_empty() {
        native_windows_gui::simple_message(
            "Gesoft Support",
            &format!("Curățarea nu s-a încheiat:\n{}", cleanup_errors.join("\n")),
        );
    }
    Ok(())
}

// RustDesk's helper processes (connection manager, server) can outlive the
// main process by a few seconds and keep the extracted DLLs open, so a single
// attempt right after it exits fails. Retry quietly before reporting.
fn remove_dir_with_retry(path: &std::path::Path) -> std::io::Result<()> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        match std::fs::remove_dir_all(path) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) if std::time::Instant::now() >= deadline => return Err(error),
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(500)),
        }
    }
}
