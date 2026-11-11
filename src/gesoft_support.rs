// Modified by Gesoft, 2026-10-02 and 2026-11-11. See CHANGES-GESOFT.md.

use std::{path::PathBuf, sync::OnceLock, time::Duration};

use hbb_common::config::{
    Config, APP_NAME, BUILTIN_SETTINGS, EXE_RENDEZVOUS_SERVER, HARD_SETTINGS, OVERWRITE_SETTINGS,
};

use crate::gesoft_profile::{self, Profile};

const CODE_ENV: &str = "GESOFT_SUPPORT_CODE";
/// Set by the packer for the technician's exe: outgoing connections, no
/// customer restrictions. The technician never elevates, so its helpers all
/// inherit it; a process without it is treated as a customer's.
const TECHNICIAN_ENV: &str = "GESOFT_SUPPORT_TECHNICIAN";
/// Passed by the portable packer once RustDesk has exited: report the
/// customer's close to the helpdesk and exit, without starting anything.
pub const CLOSED_ARG: &str = "--gesoft-support-closed";

static PROFILE: OnceLock<&'static Profile> = OnceLock::new();

fn profile() -> &'static Profile {
    // `prepare()` sets it before anything else runs, or the process exits.
    PROFILE.get().copied().unwrap_or(&gesoft_profile::LIVE)
}

/// The profile follows the directory the packer extracted RustDesk into. Every
/// process of the session runs that same executable, including an elevated
/// helper, which does not inherit the environment.
fn select_profile() -> Result<&'static Profile, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe
        .parent()
        .and_then(|d| d.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    gesoft_profile::from_dir(dir).ok_or_else(|| {
        "Gesoft Support pornește numai din descărcarea de suport. Descărcați din nou clientul din pagina de suport."
            .to_owned()
    })
}

/// Runs first in every process, with or without arguments, before any read of
/// the configuration: the compiled profile is the only source of the helpdesk,
/// the rendezvous server and its key, whatever a toml on this PC says.
fn apply_profile(profile: &'static Profile) -> Result<(), String> {
    let _ = PROFILE.set(profile);
    *APP_NAME.write().map_err(|e| e.to_string())? = profile.namespace.to_owned();
    *EXE_RENDEZVOUS_SERVER.write().map_err(|e| e.to_string())? = profile.rendezvous.to_owned();
    let mut overwrite = OVERWRITE_SETTINGS.write().map_err(|e| e.to_string())?;
    for (k, v) in [
        ("custom-rendezvous-server", profile.rendezvous),
        ("key", profile.key),
        // Empty: the relay is the one the rendezvous server names.
        ("relay-server", ""),
        ("api-server", ""),
    ] {
        overwrite.insert(k.into(), v.into());
    }
    drop(overwrite);
    // No device registration, so no heartbeat to an API server derived from
    // the rendezvous host (`hbbs_http/sync.rs`).
    BUILTIN_SETTINGS
        .write()
        .map_err(|e| e.to_string())?
        .insert("register-device".into(), "N".into());
    // This build is attended-only. Keep RustDesk's installer code available for
    // a later installer release, but never offer it in this session build.
    HARD_SETTINGS
        .write()
        .map_err(|e| e.to_string())?
        .insert("disable-installation".into(), "Y".into());
    Ok(())
}

/// A customer's support session, in every one of its processes (the main
/// window, the connection manager, the server, an elevated helper):
/// incoming-only, no sound, and the customer's click to accept, whatever an
/// existing toml says.
fn apply_customer_restrictions() -> Result<(), String> {
    // RustDesk's incoming-only mode hides the outgoing connect panel and
    // recent sessions, and refuses outgoing calls.
    HARD_SETTINGS
        .write()
        .map_err(|e| e.to_string())?
        .insert("conn-type".into(), "incoming".into());
    let mut overwrite = OVERWRITE_SETTINGS.write().map_err(|e| e.to_string())?;
    // The operator neither needs nor gets the customer's audio, in the desktop
    // or the browser viewer.
    overwrite.insert("enable-audio".into(), "N".into());
    overwrite.insert("approve-mode".into(), "click".into());
    Ok(())
}

fn is_technician() -> bool {
    std::env::var(TECHNICIAN_ENV).is_ok_and(|v| v == "1")
}

pub fn prepare() -> Result<Option<String>, String> {
    let profile = select_profile()?;
    apply_profile(profile)?;
    let technician = is_technician();
    if !technician {
        apply_customer_restrictions()?;
    }

    if let Some(first) = std::env::args_os().nth(1) {
        if first == CLOSED_ARG {
            notify_closed();
            std::process::exit(0);
        }
        return Ok(None);
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    if technician {
        // Cosmetic for the technician: never stop it from starting.
        if let Err(error) = load_brand(
            &client,
            &format!("{}/api/brand", profile.api_base),
            " — Tehnician",
        ) {
            hbb_common::log::warn!("technician branding unavailable: {error}");
        }
        return Ok(None);
    }
    let Ok(code) = std::env::var(CODE_ENV) else {
        return Ok(None);
    };
    if !gesoft_profile::valid_code(&code) {
        return Err("Codul sesiunii de suport este invalid.".into());
    }

    client
        .post(format!("{}/api/support/{code}/claim", profile.api_base))
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|e| format!("Sesiunea de suport nu a fost aprobată: {e}"))?;
    load_brand(
        &client,
        &format!("{}/api/support/{code}/brand", profile.api_base),
        "",
    )?;
    Ok(Some(code))
}

/// Best effort: if the helpdesk cannot be told, it still closes the session
/// after the client's silence.
fn notify_closed() {
    let Ok(code) = std::env::var(CODE_ENV) else {
        return;
    };
    if !gesoft_profile::valid_code(&code) {
        return;
    }
    if let Ok(client) = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        let _ = client
            .post(format!("{}/api/support/{code}/close", profile().api_base))
            .send();
    }
}

/// `url` serves the brand manifest, and each image under it. 404: no
/// branding, RustDesk keeps its own look.
fn load_brand(
    client: &reqwest::blocking::Client,
    url: &str,
    name_suffix: &str,
) -> Result<(), String> {
    let response = client.get(url).send().map_err(|e| e.to_string())?;
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
    std::env::set_var("RUSTDESK_BRAND_NAME", format!("{name}{name_suffix}"));
    std::env::set_var("RUSTDESK_BRAND_ACCENT", accent);
    Ok(())
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
            .post(format!("{}/api/ready", profile().api_base))
            .json(&serde_json::json!({"code": code, "rustdesk_id": id}))
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
        {
            hbb_common::log::error!("support ID report failed: {error}");
        }
        let status_url = format!("{}/api/status/{code}", profile().api_base);
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

/// Remote controls on which the customer switched File off in the connection
/// manager: a file transfer does not join those, it asks for Accept.
static CONTROLS_WITHOUT_FILES: std::sync::Mutex<Vec<i32>> = std::sync::Mutex::new(Vec::new());

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
    let without_files = CONTROLS_WITHOUT_FILES.lock().unwrap().clone();
    let conns = AUTHED_CONNS.lock().unwrap();
    crate::gesoft_file_join::joins_accepted_control(
        is_file_transfer,
        session_id,
        peer_id,
        key,
        conns
            .iter()
            .filter(|c| c.conn_type == AuthConnType::Remote && !without_files.contains(&c.conn_id))
            .map(|c| &c.session_key),
    )
}

/// The customer's File switch on a connection in the connection manager.
pub fn note_file_permission(conn_id: i32, enabled: bool) {
    let mut off = CONTROLS_WITHOUT_FILES.lock().unwrap();
    off.retain(|c| *c != conn_id);
    if !enabled {
        off.push(conn_id);
    }
}

/// Called as an authorized connection goes. When it is the last remote
/// control of its viewer session, the file transfers of that session end with
/// it: they were let in because of it.
pub fn end_joined_file_transfers(conn_id: i32, conn_type: crate::server::AuthConnType) {
    use crate::server::{AuthConnType, AUTHED_CONNS};
    CONTROLS_WITHOUT_FILES
        .lock()
        .unwrap()
        .retain(|c| *c != conn_id);
    if conn_type != AuthConnType::Remote {
        return;
    }
    let conns = AUTHED_CONNS.lock().unwrap();
    let Some(key) = conns
        .iter()
        .find(|c| c.conn_id == conn_id)
        .map(|c| &c.session_key)
    else {
        return;
    };
    if conns.iter().any(|c| {
        c.conn_id != conn_id && c.conn_type == AuthConnType::Remote && &c.session_key == key
    }) {
        return;
    }
    for c in conns
        .iter()
        .filter(|c| c.conn_type == AuthConnType::FileTransfer && &c.session_key == key)
    {
        hbb_common::log::info!("file transfer ends with the remote control");
        c.sender.send(crate::ipc::Data::Close).ok();
    }
}
