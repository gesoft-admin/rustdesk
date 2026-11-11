// Gesoft, 2026-11-11. See CHANGES-GESOFT.md.
//
// The two environments one support build serves, and how a downloaded file
// selects one. This file is the single source of the profile values: RustDesk
// (`src/gesoft_support.rs`) and the portable packer
// (`libs/portable/src/gesoft_support.rs`, through `#[path]`) both compile it, and
// the CI marker step reads the same constants. Free of RustDesk types, so its
// tests run on any host:
//   rustc --edition 2021 --test src/gesoft_profile.rs -o target/gesoft-profile && target/gesoft-profile
#![allow(dead_code)]

#[derive(Debug, PartialEq, Eq)]
pub struct Profile {
    pub name: &'static str,
    pub api_base: &'static str,
    pub rendezvous: &'static str,
    pub key: &'static str,
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
    api_base: "https://helpdesk.gesoft.ro",
    rendezvous: "remote.gesoft.ro",
    key: "X6NVbI1tw9PJpgGQvhBzKGman282By+A711jc1wnl34=",
    namespace: "GesoftSupport",
    dir: "gesoftsupport",
    technician: "GesoftSupport-Technician.exe",
};

pub const LAB: Profile = Profile {
    name: "lab",
    api_base: "http://192.168.99.110:8080",
    rendezvous: "192.168.99.110",
    key: "Ok9SZ5+ExasFXnNNw0lW5gg3gAsEr2xpEFenO5gHm2s=",
    namespace: "GesoftSupportLab",
    dir: "gesoftsupportlab",
    technician: "GesoftSupport-Technician-Lab.exe",
};

pub const PROFILES: [&Profile; 2] = [&LIVE, &LAB];

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

/// The profile whose API served `url`, by host and port.
fn profile_of_url(url: &str) -> Option<&'static Profile> {
    let origin = |u: &str| -> Option<String> {
        let (scheme, rest) = u.split_once("://")?;
        let authority = rest.split(['/', '?', '#']).next()?;
        Some(format!(
            "{}://{}",
            scheme.to_ascii_lowercase(),
            authority.to_ascii_lowercase()
        ))
    };
    let wanted = origin(url)?;
    PROFILES
        .into_iter()
        .find(|p| origin(p.api_base).as_deref() == Some(wanted.as_str()))
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
    /// The name and the download source disagree; refuse rather than guess.
    Conflict,
}

/// What a downloaded file was served for. `name` is the file name;
/// `zone` is its `Zone.Identifier` stream, if any. The name and the zone are
/// read as two separate candidates and never mixed: a code from one source is
/// not paired with a profile from the other.
pub fn launch(name: &str, zone: Option<&str>) -> Launch {
    if let Some(profile) = PROFILES
        .into_iter()
        .find(|p| name.eq_ignore_ascii_case(p.technician))
    {
        return Launch::Technician(profile);
    }

    let host_url = zone.and_then(|z| {
        z.lines()
            .find_map(|line| line.strip_prefix("HostUrl="))
            .map(str::trim)
    });
    let zone_profile = host_url.and_then(profile_of_url);
    let zone_code = zone.and_then(|z| {
        z.lines()
            .filter(|line| line.starts_with("HostUrl=") || line.starts_with("ReferrerUrl="))
            .find_map(code_in)
    });

    if let Some(code) = code_in(name) {
        let profile = if has_lab_flag(name) { &LAB } else { &LIVE };
        // The zone only confirms. One that names another of our helpdesks, or
        // another code, means the file was renamed: refuse.
        if zone_profile.is_some_and(|p| p != profile)
            || (zone_profile.is_some() && zone_code.as_ref().is_some_and(|c| c != &code))
        {
            return Launch::Conflict;
        }
        return Launch::Session { code, profile };
    }

    // No code in the name (renamed): the download source is the whole
    // candidate. Its profile comes from the host it was served by; a host we do
    // not know gives no session.
    match (zone_code, zone_profile) {
        (Some(code), Some(profile)) => {
            if has_lab_flag(name) && profile != &LAB {
                Launch::Conflict
            } else {
                Launch::Session { code, profile }
            }
        }
        _ => Launch::Missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIVE_ZONE: &str =
        "[ZoneTransfer]\r\nZoneId=3\r\nHostUrl=https://helpdesk.gesoft.ro/d/ZVKQ4WGVFA43/download\r\n";
    const LAB_ZONE: &str =
        "[ZoneTransfer]\nZoneId=3\nHostUrl=http://192.168.99.110:8080/d/ZVKQ4WGVFA43/download\n";

    fn session(code: &str, profile: &'static Profile) -> Launch {
        Launch::Session {
            code: code.into(),
            profile,
        }
    }

    #[test]
    fn live_names_are_unchanged_and_select_live() {
        assert_eq!(
            launch("GeSoft-Asistenta-ZVKQ4WGVFA43.exe", None),
            session("ZVKQ4WGVFA43", &LIVE)
        );
        assert_eq!(
            launch(
                "GeSoft-Remote-Support-ZVKQ4WGVFA43 (1).exe",
                Some(LIVE_ZONE)
            ),
            session("ZVKQ4WGVFA43", &LIVE)
        );
    }

    #[test]
    fn the_lab_flag_selects_lab() {
        assert_eq!(
            launch("GeSoft-Asistenta-Lab-ZVKQ4WGVFA43.exe", Some(LAB_ZONE)),
            session("ZVKQ4WGVFA43", &LAB)
        );
        assert_eq!(
            launch("GeSoft-Asistenta-lab-ZVKQ4WGVFA43 (2).exe", None),
            session("ZVKQ4WGVFA43", &LAB)
        );
    }

    #[test]
    fn lab_only_as_a_whole_word() {
        assert_eq!(
            launch("GeSoft-Laborator-ZVKQ4WGVFA43.exe", None),
            session("ZVKQ4WGVFA43", &LIVE)
        );
    }

    #[test]
    fn a_name_contradicting_the_download_source_is_refused() {
        // A lab download renamed to look live, and the other way round.
        assert_eq!(
            launch("GeSoft-Asistenta-ZVKQ4WGVFA43.exe", Some(LAB_ZONE)),
            Launch::Conflict
        );
        assert_eq!(
            launch("GeSoft-Asistenta-Lab-ZVKQ4WGVFA43.exe", Some(LIVE_ZONE)),
            Launch::Conflict
        );
        // Another code than the one it was downloaded for.
        assert_eq!(
            launch("GeSoft-Asistenta-ACDEFGHJKMNP.exe", Some(LIVE_ZONE)),
            Launch::Conflict
        );
    }

    #[test]
    fn a_zone_from_an_unknown_host_is_ignored_when_the_name_has_the_code() {
        let foreign = "[ZoneTransfer]\nHostUrl=https://example.com/d/ACDEFGHJKMNP/download\n";
        assert_eq!(
            launch("GeSoft-Asistenta-ZVKQ4WGVFA43.exe", Some(foreign)),
            session("ZVKQ4WGVFA43", &LIVE)
        );
    }

    #[test]
    fn a_renamed_file_uses_the_download_source_as_a_whole() {
        assert_eq!(
            launch("GesoftSupport.exe", Some(LIVE_ZONE)),
            session("ZVKQ4WGVFA43", &LIVE)
        );
        assert_eq!(
            launch("support.exe", Some(LAB_ZONE)),
            session("ZVKQ4WGVFA43", &LAB)
        );
        assert_eq!(launch("lab-copy.exe", Some(LIVE_ZONE)), Launch::Conflict);
        // A code with no known host cannot be paired with a profile.
        let referrer_only =
            "[ZoneTransfer]\nReferrerUrl=https://helpdesk.gesoft.ro/d/ZVKQ4WGVFA43\n";
        assert_eq!(launch("support.exe", Some(referrer_only)), Launch::Missing);
        assert_eq!(launch("support.exe", None), Launch::Missing);
    }

    #[test]
    fn technician_names() {
        assert_eq!(
            launch("GesoftSupport-Technician.exe", None),
            Launch::Technician(&LIVE)
        );
        assert_eq!(
            launch("gesoftsupport-technician-lab.exe", None),
            Launch::Technician(&LAB)
        );
        assert_eq!(
            launch("GesoftSupport-Technician (1).exe", None),
            Launch::Missing
        );
    }

    #[test]
    fn two_different_codes_are_no_code() {
        assert_eq!(code_in("GeSoft-ZVKQ4WGVFA43-ACDEFGHJKMNP.exe"), None);
        assert_eq!(code_in("GesoftSupport.exe"), None);
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
        assert_ne!(LIVE.key, LAB.key);
        assert!(PROFILES
            .iter()
            .all(|p| p.dir == p.namespace.to_ascii_lowercase()));
    }
}
