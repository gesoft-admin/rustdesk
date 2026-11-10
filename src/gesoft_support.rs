// Modified by Gesoft, 2026-10-02. See CHANGES-GESOFT.md.

use std::{path::PathBuf, time::Duration};

use hbb_common::config::{Config, HARD_SETTINGS, OVERWRITE_SETTINGS};

const API_BASE: &str = env!("GESOFT_SUPPORT_API_BASE");
const CODE_ALPHABET: &[u8] = b"23456789ACDEFGHJKMNPQRTVWXYZ";
const CODE_ENV: &str = "GESOFT_SUPPORT_CODE";
/// Passed by the portable packer once RustDesk has exited: report the
/// customer's close to the helpdesk and exit, without starting anything.
pub const CLOSED_ARG: &str = "--gesoft-support-closed";

pub fn prepare() -> Result<Option<String>, String> {
    // This build is attended-only. Keep RustDesk's installer code available for
    // a later installer release, but never offer it in this session build.
    HARD_SETTINGS
        .write()
        .map_err(|e| e.to_string())?
        .insert("disable-installation".into(), "Y".into());

    if let Some(first) = std::env::args_os().nth(1) {
        if first == CLOSED_ARG {
            notify_closed();
            std::process::exit(0);
        }
        return Ok(None);
    }
    let Ok(code) = std::env::var(CODE_ENV) else {
        return Ok(None);
    };
    if code.len() != 12 || !code.bytes().all(|b| CODE_ALPHABET.contains(&b)) {
        return Err("Codul sesiunii de suport este invalid.".into());
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    client
        .post(format!("{API_BASE}/api/support/{code}/claim"))
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|e| format!("Sesiunea de suport nu a fost aprobată: {e}"))?;

    // A support session only receives: RustDesk's incoming-only mode hides the
    // outgoing connect panel and recent sessions, and refuses outgoing calls.
    HARD_SETTINGS
        .write()
        .map_err(|e| e.to_string())?
        .insert("conn-type".into(), "incoming".into());
    // Support sessions carry no sound: the operator neither needs nor gets the
    // customer's audio, in the desktop or the browser viewer.
    OVERWRITE_SETTINGS
        .write()
        .map_err(|e| e.to_string())?
        .insert("enable-audio".into(), "N".into());
    load_brand(&client, &code)?;
    prepare_click_approval()?;
    Ok(Some(code))
}

/// Best effort: if the helpdesk cannot be told, it still closes the session
/// after the client's silence.
fn notify_closed() {
    let Ok(code) = std::env::var(CODE_ENV) else {
        return;
    };
    if code.len() != 12 || !code.bytes().all(|b| CODE_ALPHABET.contains(&b)) {
        return;
    }
    if let Ok(client) = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        let _ = client
            .post(format!("{API_BASE}/api/support/{code}/close"))
            .send();
    }
}

fn load_brand(client: &reqwest::blocking::Client, code: &str) -> Result<(), String> {
    let url = format!("{API_BASE}/api/support/{code}/brand");
    let response = client.get(&url).send().map_err(|e| e.to_string())?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(());
    }
    let manifest = response
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json::<serde_json::Value>()
        .map_err(|e| e.to_string())?;
    let name = manifest["name"].as_str().ok_or("Brand name missing")?;
    let accent = manifest["accent"].as_str().ok_or("Brand accent missing")?;
    if name.is_empty()
        || name.len() > 128
        || name.contains('\0')
        || accent.len() != 7
        || !accent.starts_with('#')
        || !accent[1..].bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("Invalid support branding".into());
    }
    let dir = std::env::var_os("RUSTDESK_BRAND_DIR")
        .map(PathBuf::from)
        .ok_or("Brand directory unavailable")?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for asset in ["logo_light", "logo_dark", "icon"] {
        let response = client
            .get(format!("{url}/{asset}"))
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .map_err(|e| e.to_string())?;
        let bytes = response.bytes().map_err(|e| e.to_string())?;
        if bytes.len() > 2 * 1024 * 1024 || !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Err(format!("Invalid brand image: {asset}"));
        }
        std::fs::write(dir.join(format!("{asset}.png")), bytes).map_err(|e| e.to_string())?;
    }
    std::env::set_var("RUSTDESK_BRAND_NAME", name);
    std::env::set_var("RUSTDESK_BRAND_ACCENT", accent);
    Ok(())
}

fn prepare_click_approval() -> Result<(), String> {
    let roaming = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or("Profilul Windows nu este disponibil")?;
    let dir = roaming.join("GesoftSupport").join("config");
    let file = dir.join("GesoftSupport2.toml");
    if file.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(
        file,
        "rendezvous_server = ''\nnat_type = 0\nserial = 0\n\n[options]\napprove-mode = 'click'\n",
    )
    .map_err(|e| e.to_string())
}

pub fn start(code: String) {
    let id = Config::get_id();
    std::thread::spawn(move || {
        let client = match reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
        {
            Ok(client) => client,
            Err(error) => {
                hbb_common::log::error!("support client HTTP setup failed: {error}");
                return;
            }
        };
        if let Err(error) = client
            .post(format!("{API_BASE}/api/ready"))
            .json(&serde_json::json!({"code": code, "rustdesk_id": id}))
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
        {
            hbb_common::log::error!("support ID report failed: {error}");
        }
        let status_url = format!("{API_BASE}/api/status/{code}");
        loop {
            std::thread::sleep(Duration::from_secs(2));
            let status = client
                .get(&status_url)
                .send()
                .and_then(reqwest::blocking::Response::error_for_status)
                .and_then(|response| response.json::<serde_json::Value>());
            match status {
                Ok(body) if matches!(body["status"].as_str(), Some("CLOSED" | "EXPIRED")) => {
                    std::process::exit(0);
                }
                Ok(_) => {}
                Err(error) => hbb_common::log::warn!("support status unavailable: {error}"),
            }
        }
    });
}

/// A file transfer that joins the remote control the customer accepted in the
/// same viewer session is let in without a second Accept; see
/// `gesoft_file_join.rs` for what binds the two.
pub fn file_transfer_joins_control(
    is_file_transfer: bool,
    session_id: u64,
    peer_id: &str,
    key: &crate::server::SessionKey,
) -> bool {
    use crate::server::{AuthConnType, AUTHED_CONNS};
    let conns = AUTHED_CONNS.lock().unwrap();
    crate::gesoft_file_join::joins_accepted_control(
        is_file_transfer,
        session_id,
        peer_id,
        key,
        conns
            .iter()
            .filter(|c| c.conn_type == AuthConnType::Remote)
            .map(|c| &c.session_key),
    )
}
