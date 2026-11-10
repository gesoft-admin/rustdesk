# Changes made by Gesoft

This file exists to satisfy AGPL-3.0 §5(a): the work must carry prominent
notices stating that it was modified, and giving a relevant date. It lists
every change this fork makes to upstream RustDesk, and every change our
`hbb_common` fork makes to upstream `hbb_common`, because the two are
distributed together as one work.

Upstream is <https://github.com/rustdesk/rustdesk> (branch `master`) and
<https://github.com/rustdesk/hbb_common> (branch `main`). Everything below is
the complete difference; anything not listed here is upstream's, unmodified.

Modified by **Gesoft** — <https://github.com/gesoft-admin/rustdesk>.

## `rustdesk`

### `libs/portable/src/gesoft_support.rs` — added 2026-10-02

The Gesoft portable packer reads the code from the download name, extracts
RustDesk, passes the code to it and removes only directories created by that
run. It does not present a separate guidance or extraction window. The code is
compiled only with the `gesoft-support` feature.

Modified 2026-11-09: cleanup after the session retries removing the extracted
directories for up to 60 seconds, because RustDesk's helper processes can keep
the DLLs open briefly after the main process exits; the error dialog appears
only if removal still fails.

Modified 2026-11-09: when RustDesk exits, the packer runs it once more with
`--gesoft-support-closed` so the Helpdesk learns at once that the customer
closed the session, before cleanup starts.

### `src/gesoft_support.rs` — added 2026-10-02

The Windows RustDesk application claims the session before startup, sets
attended click approval, reports its ID and observes remote close. Its
`gesoft-support` feature suppresses the installation offer for this temporary
build. RustDesk's installer code remains available for a future installer
build. Optional runtime branding is fetched from the Helpdesk session and
written beside the extracted application; no GeSoft symbol is compiled into
RustDesk.

Modified 2026-11-08: once the session is claimed, it also sets RustDesk's
existing incoming-only mode (`conn-type = incoming`), which hides the outgoing
connect panel and recent sessions and refuses outgoing connections. Without a
support code the application is unchanged.

Modified 2026-11-08: a claimed session also overrides RustDesk's existing
`enable-audio` option to `N`, so the customer's audio is not sent to the
operator (desktop or browser viewer). Without a support code it is unchanged.

Modified 2026-11-09: with `--gesoft-support-closed` and a support code in the
environment, it reports the close to the Helpdesk (best effort, 5 s) and exits
before starting anything else.

Modified 2026-11-10: `file_transfer_joins_control` lets a file-transfer login in
without a second Accept when the customer has already accepted a remote control
of the same viewer session (same peer ID, name and non-zero session ID, which
the viewer sends only inside the encrypted channel). The pure decision is in
the new `src/gesoft_file_join.rs`, with tests that run on any host. Any other
login, and a file transfer without such a control, still asks for Accept.

### `src/server/connection.rs` — modified 2026-11-10

One feature-gated hook in the click-approval branch of the login handler calls
`gesoft_support::file_transfer_joins_control`; when it holds, the connection is
authorized as the customer's Accept of that control was (audited as a click,
no 2FA prompt, `send_logon_response_and_keep_alive`), and the connection manager
is told it is authorized, so the customer sees it and can disconnect it. A file
transfer can join a control in the brief moment between its close and its
removal from the list of authorized connections; the browser path is closed by
the Helpdesk gateway, which ends the file transfer with the control. Builds
without `gesoft-support` are unchanged.

### `flutter/lib/common.dart`, `flutter/lib/desktop/pages/desktop_home_page.dart`, `flutter/windows/runner/main.cpp` — modified 2026-10-02

These read a runtime display name, accent colour, logo and icon from the
session environment. With no brand configuration the existing application name,
blue accent and generic artwork remain. The supporting platform-specific
readers are new `flutter/lib/native/brand.dart` and `flutter/lib/web/brand.dart`.

### `Cargo.toml`, `build.py`, `src/lib.rs`, `src/core_main.rs` — modified 2026-10-02

These opt the RustDesk application into the support feature only when the
Helpdesk API base is set at build time. The shared startup path has a thin
feature-gated hook; builds without the feature follow the previous path.
`src/lib.rs` also declares `gesoft_file_join` under the same feature (2026-11-10).

### `libs/portable/Cargo.toml`, `generate.py`, `src/main.rs`, build workflow — modified 2026-10-02

These opt the portable packer into the support handoff when a Helpdesk API base
is provided at build time. Builds without the feature continue through the
upstream portable entry point.

### `src/common.rs` — modified 2026-09-06

Two changes, both about one thing: a build compiled against a self-hosted
rendezvous server must not be treated as if it were talking to RustDesk's
public infrastructure.

* `check_software_update()` returns early unless the build is on a public
  server, so a self-hosted client does not contact the public RustDesk update
  endpoint on startup.
* `using_public_server()` now classifies the rendezvous server actually in use
  — `Config::get_rendezvous_server()` — instead of only checking whether the
  `custom-rendezvous-server` option is set. Upstream's version reports a custom
  *compiled-in* fallback as public, which is exactly the case this fork
  creates.
* A unit test was added covering both the public and the compiled-custom case.

### `.gitmodules` and the `libs/hbb_common` gitlink — modified 2026-09-06

The submodule URL points at <https://github.com/gesoft-admin/hbb_common> and
the gitlink pins a commit on its `support` branch, rather than upstream's. The
pinned commit for any given build is recorded in the parent tree; see
`LICENCE-COMPLIANCE.md` for how to read it.

### `.github/workflows/support-windows-build.yml` — added 2026-09-06, modified 2026-09-08

A new workflow, written by Gesoft, producing an unsigned Windows x64 portable
build. On 2026-09-08 it was changed to rename the packed payload executable
from `rustdesk.exe` to `GesoftSupport.exe`, because the self-extractor names
its target directory after the payload's stem: packing it under the upstream
name would extract into `%LOCALAPPDATA%\rustdesk` and overwrite a customer's
own portable RustDesk installation.

### Files added by Gesoft

`LICENCE-COMPLIANCE.md`, `CHANGES-GESOFT.md`. These are ours; they modify no
upstream code.

## `hbb_common`

Distributed as a submodule of the above. Upstream `rustdesk/hbb_common`, branch
`main`.

### `src/config.rs` — modified 2026-09-06 and 2026-09-08

Three constants, and nothing else in the file or the repository:

| line | upstream | this fork | date |
|---|---|---|---|
| `RENDEZVOUS_SERVERS` | `["rs-ny.rustdesk.com"]` | `["remote.gesoft.ro"]` | 2026-09-06 |
| `RS_PUB_KEY` | RustDesk's public key | our server's public key | 2026-09-06 |
| `APP_NAME` | `"RustDesk"` | `"GesoftSupport"` | 2026-09-08 |

`APP_NAME` decides the configuration directory and the names of the files in
it, so this is what keeps our client out of `%APPDATA%\RustDesk` and out of a
customer's own RustDesk profile.

### Files added by Gesoft

`COMPLIANCE.md`, which explains why that repository carries no licence file of
its own. It modifies no upstream code.

## Not modified

No other runtime code in either repository is changed beyond the files listed
above. The build's exact dependency versions are recorded in `Cargo.lock`.

Worth being precise about branding, since `APP_NAME` is a product string and
changing it is a branding change: it is the only one. The icons, the artwork,
the window chrome, the translation catalogues and every other user-facing
string are upstream's, unmodified. The `LICENCE` file at the root of this
repository is upstream's AGPL-3.0, unaltered.

Where the upstream name still appears in the interface, that is upstream's
text, not a claim by us. Where our name appears, it is a namespace for
configuration and process names rather than a rebrand of the application. See
`LICENCE-COMPLIANCE.md` for how we treat the source obligations that follow
from distributing this.
