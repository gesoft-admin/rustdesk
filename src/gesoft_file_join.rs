// Gesoft, 2026-11-10. See CHANGES-GESOFT.md.
//
// Whether a file-transfer login joins a remote control that the customer has
// already accepted, so that it needs no second Accept. Free of RustDesk types,
// so its tests run on any host:
//   rustc --edition 2021 --test src/gesoft_file_join.rs -o target/gesoft-file-join && target/gesoft-file-join

/// `peer_id` and the peer's name come from the peer itself. What binds the new
/// login to the accepted one is the session id: a random 63-bit value the
/// viewer chose for the accepted control and sends only inside the encrypted
/// channel. Zero is what a client sends when it has none, so it never matches.
pub fn joins_accepted_control<'a, K: PartialEq + 'a>(
    is_file_transfer: bool,
    session_id: u64,
    peer_id: &str,
    key: &K,
    mut accepted_remote_controls: impl Iterator<Item = &'a K>,
) -> bool {
    is_file_transfer
        && session_id != 0
        && !peer_id.is_empty()
        && accepted_remote_controls.any(|accepted| accepted == key)
}

#[cfg(test)]
mod tests {
    use super::joins_accepted_control;

    type Key = (&'static str, &'static str, u64);
    const CONTROL: Key = ("helpdesk-7", "Operator", 0x1234_5678_9abc_def0);

    fn joins(is_file: bool, key: Key, accepted: &[Key]) -> bool {
        joins_accepted_control(is_file, key.2, key.0, &key, accepted.iter())
    }

    #[test]
    fn a_file_transfer_of_the_accepted_session_joins() {
        assert!(joins(true, CONTROL, &[CONTROL]));
        assert!(joins(true, CONTROL, &[("other", "x", 9), CONTROL]));
    }

    #[test]
    fn no_accepted_control_means_accept_again() {
        assert!(!joins(true, CONTROL, &[]));
    }

    #[test]
    fn another_session_id_does_not_join() {
        assert!(!joins(
            true,
            ("helpdesk-7", "Operator", 0x1234_5678_9abc_def1),
            &[CONTROL]
        ));
    }

    #[test]
    fn the_same_session_id_from_another_peer_does_not_join() {
        assert!(!joins(
            true,
            ("helpdesk-8", "Operator", CONTROL.2),
            &[CONTROL]
        ));
        assert!(!joins(
            true,
            ("helpdesk-7", "Someone", CONTROL.2),
            &[CONTROL]
        ));
    }

    #[test]
    fn session_id_zero_never_joins() {
        let none = ("helpdesk-7", "Operator", 0);
        assert!(!joins(true, none, &[none]));
    }

    #[test]
    fn an_empty_peer_id_never_joins() {
        let anonymous = ("", "", 42);
        assert!(!joins(true, anonymous, &[anonymous]));
    }

    #[test]
    fn only_file_transfer_joins() {
        // Control, camera, terminal and port forward logins pass false.
        assert!(!joins(false, CONTROL, &[CONTROL]));
    }
}
