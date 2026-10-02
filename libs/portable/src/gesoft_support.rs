// Modified by Gesoft, 2026-10-02. See CHANGES-GESOFT.md.
// The API is fixed at build time and checked by the helpdesk before download.

use std::process::{Command, Stdio};

const ALPHABET: &[u8] = b"23456789ACDEFGHJKMNPQRTVWXYZ";

#[used]
static API_BASE_MARKER: &str = concat!("HELPDESK-API-BASE:", env!("GESOFT_SUPPORT_API_BASE"), "\0");

pub fn session_code() -> Option<String> {
    let path = std::env::current_exe().ok()?;
    let name = path.file_name()?.to_str()?;
    code_in(name).or_else(|| {
        let mut stream = path.as_os_str().to_owned();
        stream.push(":Zone.Identifier");
        let zone = std::fs::read_to_string(stream).ok()?;
        zone.lines()
            .filter(|line| line.starts_with("HostUrl=") || line.starts_with("ReferrerUrl="))
            .filter_map(code_in)
            .next()
    })
}

pub fn is_technician_executable() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .is_some_and(|name| name.eq_ignore_ascii_case("GesoftSupport-Technician.exe"))
}

pub fn missing_session() -> Result<(), String> {
    let error = "Lipsesc detaliile sesiunii. Descărcați din nou clientul din pagina de suport.";
    native_windows_gui::simple_message("Gesoft Support", error);
    Err(error.to_string())
}

fn code_in(text: &str) -> Option<String> {
    let mut found: Option<String> = None;
    for part in text.split(|c: char| !c.is_ascii_alphanumeric()) {
        let token = part.to_ascii_uppercase();
        if token.len() != 12 || !token.bytes().all(|b| ALPHABET.contains(&b)) {
            continue;
        }
        if found.as_ref().is_some_and(|prior| prior != &token) {
            return None;
        }
        found = Some(token);
    }
    found
}

pub fn run(code: &str) -> Result<(), String> {
    let result = run_session(code);
    if let Err(error) = &result {
        native_windows_gui::simple_message(
            "Gesoft Support",
            &format!("Sesiunea nu a putut porni.\n{error}"),
        );
    }
    result
}

fn run_session(code: &str) -> Result<(), String> {
    let reader = super::BinaryReader::new()?;
    let local = dirs::data_local_dir()
        .ok_or("LocalAppData unavailable")?
        .join(super::app_dir_name(&reader.exe));
    let local_existed = local.exists();
    let roaming = dirs::config_dir()
        .ok_or("AppData unavailable")?
        .join("GesoftSupport");
    let roaming_existed = roaming.exists();
    let mut ui = false;
    // setup shows its separate extraction window for empty args. This
    // support handoff keeps the customer's first visible window in RustDesk.
    let no_progress_window = vec!["--gesoft-support".to_owned()];
    let payload = super::setup(reader, None, false, &no_progress_window, &mut ui)
        .ok_or("Could not extract RustDesk")?;
    let brand_dir = local.join(format!("branding-{}", std::process::id()));
    std::fs::create_dir_all(&brand_dir).map_err(|e| e.to_string())?;
    let mut child = Command::new(&payload)
        .env("RUSTDESK_APPNAME", "GesoftSupport.exe")
        .env("GESOFT_SUPPORT_CODE", code)
        .env("RUSTDESK_BRAND_DIR", &brand_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    child.wait().map_err(|e| e.to_string())?;
    let mut cleanup_errors = Vec::new();
    if let Err(error) = std::fs::remove_dir_all(&brand_dir) {
        cleanup_errors.push(format!("{}: {error}", brand_dir.display()));
    }
    for (preexisting, path) in [(local_existed, local), (roaming_existed, roaming)] {
        if !preexisting && path.exists() {
            if let Err(error) = std::fs::remove_dir_all(&path) {
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

#[cfg(test)]
mod tests {
    use super::code_in;
    #[test]
    fn one_valid_code_in_download_name() {
        assert_eq!(
            code_in("GeSoft-Asistenta-ZVKQ4WGVFA43 (1).exe"),
            Some("ZVKQ4WGVFA43".into())
        );
        assert_eq!(code_in("GesoftSupport.exe"), None);
        assert_eq!(code_in("GeSoft-ZVKQ4WGVFA43-ACDEFGHJKMNP.exe"), None);
    }
}
