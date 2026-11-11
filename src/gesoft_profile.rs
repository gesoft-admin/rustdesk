// Gesoft, 2026-11-11. See CHANGES-GESOFT.md.
//
// The two environments one support build serves, and how a downloaded file
// selects one. RustDesk (`src/gesoft_support.rs`) and the portable packer
// (`libs/portable/src/gesoft_support.rs`, through `#[path]`) both compile this
// file, and the CI marker step reads `LIVE_ENDPOINTS` from it. Free of RustDesk
// types, so its tests run on any host:
//   rustc --edition 2021 --test src/gesoft_profile.rs -o target/gesoft-profile && target/gesoft-profile
#![allow(dead_code)]

use std::path::PathBuf;

/// Who a profile is on Windows. Its addresses are separate: compiled for live,
/// read from the test machine for lab (`endpoints`).
#[derive(Debug, PartialEq, Eq)]
pub struct Profile {
    pub name: &'static str,
    /// RustDesk's `APP_NAME`: the configuration directory, the toml files and
    /// the IPC names, so one profile never reads the other's configuration.
    pub namespace: &'static str,
    /// The directory under `%LOCALAPPDATA%` the packer extracts into. RustDesk
    /// derives its profile from it, because an elevated helper does not inherit
    /// the environment but runs the same extracted executable.
    pub dir: &'static str,
    /// The technician's download name for this profile.
    pub technician: &'static str,
}

pub const LIVE: Profile = Profile {
    name: "live",
    namespace: "GesoftSupport",
    dir: "gesoftsupport",
    technician: "GesoftSupport-Technician.exe",
};

pub const LAB: Profile = Profile {
    name: "lab",
    namespace: "GesoftSupportLab",
    dir: "gesoftsupportlab",
    technician: "GesoftSupport-Technician-Lab.exe",
};

pub const PROFILES: [&Profile; 2] = [&LIVE, &LAB];

/// The helpdesk, rendezvous server and key a profile talks to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoints {
    pub api_base: String,
    pub rendezvous: String,
    pub key: String,
}

/// Live's addresses, compiled: no file on a customer's PC can change them.
pub const LIVE_API_BASE: &str = "https://helpdesk.gesoft.ro";
pub const LIVE_RENDEZVOUS: &str = "remote.gesoft.ro";
pub const LIVE_KEY: &str = "X6NVbI1tw9PJpgGQvhBzKGman282By+A711jc1wnl34=";

pub fn live_endpoints() -> Endpoints {
    Endpoints {
        api_base: LIVE_API_BASE.to_owned(),
        rendezvous: LIVE_RENDEZVOUS.to_owned(),
        key: LIVE_KEY.to_owned(),
    }
}

pub const LAB_CONFIG_MISSING: &str = "Profilul de test nu este configurat pe acest calculator.";

/// Lab's addresses live only on our test machines, in a file every process of
/// a session reads alike, the elevated and SYSTEM ones included.
pub fn lab_config_path() -> PathBuf {
    let base = std::env::var_os("ProgramData")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
    base.join(LAB.namespace).join("lab.conf")
}

/// `api_base=`, `rendezvous=` and `key=` lines; blank lines and `#` comments
/// are ignored. Anything else, a missing or repeated key, or a value of the
/// live profile is refused.
pub fn parse_lab_config(text: &str) -> Result<Endpoints, String> {
    let (mut api_base, mut rendezvous, mut key) = (None, None, None);
    // Notepad may save a UTF-8 byte order mark.
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, value) = line
            .split_once('=')
            .ok_or_else(|| format!("line without '=': {line}"))?;
        let slot = match name.trim() {
            "api_base" => &mut api_base,
            "rendezvous" => &mut rendezvous,
            "key" => &mut key,
            other => return Err(format!("unknown key {other}")),
        };
        if slot.is_some() {
            return Err(format!("{} given twice", name.trim()));
        }
        *slot = Some(value.trim().to_owned());
    }
    let api_base = api_base.ok_or("api_base missing")?;
    let rendezvous = rendezvous.ok_or("rendezvous missing")?;
    let key = key.ok_or("key missing")?;
    let api_ok = (api_base.starts_with("http://") || api_base.starts_with("https://"))
        && api_base.len() > "https://".len()
        && !api_base.chars().any(char::is_whitespace);
    if !api_ok {
        return Err("api_base must be an http(s) address".into());
    }
    if rendezvous.is_empty()
        || rendezvous.contains("://")
        || rendezvous.chars().any(char::is_whitespace)
    {
        return Err("rendezvous must be a host".into());
    }
    let key_ok = key.len() == 44
        && key.ends_with('=')
        && key[..43]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'/');
    if !key_ok {
        return Err("key must be a base64 public key".into());
    }
    if api_base.trim_end_matches('/') == LIVE_API_BASE
        || rendezvous.eq_ignore_ascii_case(LIVE_RENDEZVOUS)
        || key == LIVE_KEY
    {
        return Err("the lab configuration names the live helpdesk".into());
    }
    Ok(Endpoints {
        api_base: api_base.trim_end_matches('/').to_owned(),
        rendezvous,
        key,
    })
}

/// The addresses of `profile`: live's compiled ones, never a file; lab's from
/// `lab_config_path()`, or an error that stops the client.
pub fn endpoints(profile: &Profile) -> Result<Endpoints, String> {
    if profile == &LIVE {
        return Ok(live_endpoints());
    }
    let path = lab_config_path();
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("{LAB_CONFIG_MISSING}\n{}: {e}", path.display()))?;
    parse_lab_config(&text).map_err(|e| format!("{LAB_CONFIG_MISSING}\n{}: {e}", path.display()))
}

const CODE_ALPHABET: &[u8] = b"23456789ACDEFGHJKMNPQRTVWXYZ";

/// The profile of an extracted RustDesk, from the name of the directory it runs
/// in. Anything else is not a support installation: `None`, and the caller
/// refuses to start.
pub fn from_dir(dir_name: &str) -> Option<&'static Profile> {
    PROFILES
        .into_iter()
        .find(|p| p.dir.eq_ignore_ascii_case(dir_name))
}

pub fn valid_code(code: &str) -> bool {
    code.len() == 12 && code.bytes().all(|b| CODE_ALPHABET.contains(&b))
}

fn tokens(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
}

/// The one session code in `text`; none when there are two different ones.
pub fn code_in(text: &str) -> Option<String> {
    let mut found: Option<String> = None;
    for token in tokens(text) {
        let token = token.to_ascii_uppercase();
        if !valid_code(&token) {
            continue;
        }
        if found.as_ref().is_some_and(|prior| prior != &token) {
            return None;
        }
        found = Some(token);
    }
    found
}

/// `Lab` as a whole word of the name. Its absence means live: the live names
/// carry no flag, so a customer's download is unchanged.
fn has_lab_flag(name: &str) -> bool {
    tokens(name).any(|t| t.eq_ignore_ascii_case("lab"))
}

#[derive(Debug, PartialEq, Eq)]
pub enum Launch {
    Session {
        code: String,
        profile: &'static Profile,
    },
    Technician(&'static Profile),
    /// No usable session: the packer tells the customer to download again.
    Missing,
}

/// What a downloaded file was served for, from its name alone. A file renamed
/// without its code is not a session: the customer downloads it again.
pub fn launch(name: &str) -> Launch {
    if let Some(profile) = PROFILES
        .into_iter()
        .find(|p| name.eq_ignore_ascii_case(p.technician))
    {
        return Launch::Technician(profile);
    }
    match code_in(name) {
        Some(code) => Launch::Session {
            code,
            profile: if has_lab_flag(name) { &LAB } else { &LIVE },
        },
        None => Launch::Missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAB_CONF: &str = "# lab helpdesk\r\napi_base=http://10.0.0.5:8080/\r\n\r\nrendezvous = 10.0.0.5\r\nkey=Ok9SZ5+ExasFXnNNw0lW5gg3gAsEr2xpEFenO5gHm2s=\r\n";

    fn session(code: &str, profile: &'static Profile) -> Launch {
        Launch::Session {
            code: code.into(),
            profile,
        }
    }

    #[test]
    fn live_names_are_unchanged_and_select_live() {
        assert_eq!(
            launch("GeSoft-Asistenta-ZVKQ4WGVFA43.exe"),
            session("ZVKQ4WGVFA43", &LIVE)
        );
        assert_eq!(
            launch("GeSoft-Remote-Support-ZVKQ4WGVFA43 (1).exe"),
            session("ZVKQ4WGVFA43", &LIVE)
        );
        assert_eq!(
            launch("gesoft-asistenta-zvkq4wgvfa43 - Copy.EXE"),
            session("ZVKQ4WGVFA43", &LIVE)
        );
    }

    #[test]
    fn the_lab_flag_selects_lab() {
        assert_eq!(
            launch("GeSoft-Asistenta-Lab-ZVKQ4WGVFA43.exe"),
            session("ZVKQ4WGVFA43", &LAB)
        );
        assert_eq!(
            launch("GeSoft-Asistenta-lab-ZVKQ4WGVFA43 (2).exe"),
            session("ZVKQ4WGVFA43", &LAB)
        );
    }

    #[test]
    fn lab_only_as_a_whole_word() {
        assert_eq!(
            launch("GeSoft-Laborator-ZVKQ4WGVFA43.exe"),
            session("ZVKQ4WGVFA43", &LIVE)
        );
    }

    #[test]
    fn a_name_without_one_code_is_no_session() {
        assert_eq!(launch("GesoftSupport.exe"), Launch::Missing);
        assert_eq!(launch("support-lab.exe"), Launch::Missing);
        assert_eq!(
            launch("GeSoft-ZVKQ4WGVFA43-ACDEFGHJKMNP.exe"),
            Launch::Missing
        );
    }

    #[test]
    fn technician_names() {
        assert_eq!(
            launch("GesoftSupport-Technician.exe"),
            Launch::Technician(&LIVE)
        );
        assert_eq!(
            launch("gesoftsupport-technician-lab.exe"),
            Launch::Technician(&LAB)
        );
        assert_eq!(launch("GesoftSupport-Technician (1).exe"), Launch::Missing);
    }

    #[test]
    fn directories_map_to_profiles_and_nothing_else() {
        assert_eq!(from_dir("gesoftsupport"), Some(&LIVE));
        assert_eq!(from_dir("GesoftSupportLab"), Some(&LAB));
        assert_eq!(from_dir("rustdesk"), None);
        assert_eq!(from_dir(""), None);
    }

    #[test]
    fn profiles_are_distinct() {
        assert_ne!(LIVE.namespace, LAB.namespace);
        assert_ne!(LIVE.dir, LAB.dir);
        assert!(PROFILES
            .iter()
            .all(|p| p.dir == p.namespace.to_ascii_lowercase()));
    }

    #[test]
    fn live_never_reads_a_file() {
        std::env::set_var("ProgramData", "/nonexistent-gesoft-test");
        assert_eq!(endpoints(&LIVE), Ok(live_endpoints()));
        assert!(endpoints(&LAB)
            .unwrap_err()
            .starts_with(LAB_CONFIG_MISSING));
    }

    #[test]
    fn a_lab_config_reads() {
        assert_eq!(
            parse_lab_config(LAB_CONF),
            Ok(Endpoints {
                api_base: "http://10.0.0.5:8080".into(),
                rendezvous: "10.0.0.5".into(),
                key: "Ok9SZ5+ExasFXnNNw0lW5gg3gAsEr2xpEFenO5gHm2s=".into(),
            })
        );
    }

    #[test]
    fn a_lab_config_saved_with_a_byte_order_mark_reads() {
        assert!(parse_lab_config(&format!("\u{feff}{LAB_CONF}")).is_ok());
    }

    #[test]
    fn a_bad_lab_config_is_refused() {
        let key = "key=Ok9SZ5+ExasFXnNNw0lW5gg3gAsEr2xpEFenO5gHm2s=";
        for text in [
            "",
            "api_base=http://10.0.0.5:8080\nrendezvous=10.0.0.5\n",
            &format!("rendezvous=10.0.0.5\n{key}\n"),
            &format!("api_base=ftp://10.0.0.5\nrendezvous=10.0.0.5\n{key}\n"),
            &format!("api_base=http://\nrendezvous=10.0.0.5\n{key}\n"),
            &format!("api_base=http://10.0.0.5:8080\nrendezvous=http://10.0.0.5\n{key}\n"),
            "api_base=http://10.0.0.5:8080\nrendezvous=10.0.0.5\nkey=short=\n",
            &format!("api_base=http://10.0.0.5:8080\nrendezvous=10.0.0.5\n{key}\n{key}\n"),
            &format!("api_base=http://10.0.0.5:8080\nrendezvous=10.0.0.5\n{key}\nrelay=x\n"),
            &format!("api_base=http://10.0.0.5:8080\nrendezvous=10.0.0.5\n{key}\njunk\n"),
        ] {
            assert!(parse_lab_config(text).is_err(), "accepted: {text:?}");
        }
    }

    #[test]
    fn a_lab_config_naming_live_is_refused() {
        let key = "key=Ok9SZ5+ExasFXnNNw0lW5gg3gAsEr2xpEFenO5gHm2s=";
        for text in [
            format!("api_base={LIVE_API_BASE}/\nrendezvous=10.0.0.5\n{key}\n"),
            format!("api_base=http://10.0.0.5:8080\nrendezvous=REMOTE.gesoft.ro\n{key}\n"),
            format!("api_base=http://10.0.0.5:8080\nrendezvous=10.0.0.5\nkey={LIVE_KEY}\n"),
        ] {
            assert!(parse_lab_config(&text).is_err(), "accepted: {text:?}");
        }
    }
}
