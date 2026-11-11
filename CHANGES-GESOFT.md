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

Modified 2026-11-11: one build serves the live and the lab Helpdesk. The packer
reads the profile from the download name (a `Lab` word selects the lab; live
names carry none) and from the download source Windows records beside the file
(`Zone.Identifier`), as two separate candidates that must agree, and extracts
RustDesk into the profile's own directory (`%LOCALAPPDATA%\gesoftsupport` or
`gesoftsupportlab`), which is how RustDesk learns its profile. The technician's
exe (`GesoftSupport-Technician.exe`, `…-Technician-Lab.exe`) is extracted the
same way and started with `GESOFT_SUPPORT_TECHNICIAN=1` and a branding
directory. The address marker compiled into the packer is gone; CI appends a
profile marker instead (below).

Modified 2026-11-11 (later): the packer reads only the download name; the
`Zone.Identifier` fallback and the name/source conflict check are gone, so a
file renamed without its code asks for a new download. Before extracting a
lab session or the lab technician, it checks that the test machine's
`%ProgramData%\GesoftSupportLab\lab.conf` is in place and valid, and stops
with a message otherwise.

### `src/gesoft_profile.rs` — added 2026-11-11

The two profiles one build serves, `live` and `lab`: the configuration
namespace (`APP_NAME`), the extraction directory and the technician's name,
plus the pure decision of which profile a download name selects. Live's
Helpdesk API base, rendezvous host and public key are compiled; lab's are read
from `%ProgramData%\GesoftSupportLab\lab.conf` on the test machine (modified
later on 2026-11-11: no lab address is compiled any more, and the name alone
selects the profile). Compiled into RustDesk and, through
`#[path]`, into the portable packer; read by the CI marker step. Its tests run
on any host.

### `src/gesoft_support.rs` — added 2026-10-02

The Windows RustDesk application claims the session before startup, sets
attended click approval, reports its ID and observes remote close. Its
`gesoft-support` feature suppresses the installation offer for this temporary
build. RustDesk's installer code remains available for a future installer
build. Optional runtime branding is fetched from the Helpdesk session and
written beside the extracted application; no GeSoft artwork is compiled into
RustDesk (since 2026-11-11 the support build's version info names Gesoft
Support; see the build files below).

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

Modified 2026-11-11: the Helpdesk API base, the rendezvous server and its key
come from the compiled profile, no longer from a build-time address. Every
process of the build (with or without arguments, including an elevated helper,
which does not inherit the environment) first selects the profile from the
directory it runs in and refuses to start outside the two known ones; then sets
`APP_NAME` to the profile's namespace, `EXE_RENDEZVOUS_SERVER`, and overrides
`custom-rendezvous-server`, `key`, `relay-server` and `api-server` so that a
toml on the PC cannot steer the server, the key or the relay, and turns device
registration off (no heartbeat). Unless started as the technician, every process
also applies the customer restrictions (incoming-only, no audio, and
`approve-mode = click` as an override, where before it was written to a new toml
only), not only the main window. The technician's RustDesk fetches the
Helpdesk's public branding and shows it with „— Tehnician”; if it cannot, it
starts with the stock look.

Modified 2026-11-10: `file_transfer_joins_control` lets a file-transfer login in
without a second Accept when the customer has already accepted a remote control
of the same viewer session (same peer ID, name and non-zero session ID, which
the viewer sends only inside the encrypted channel). The pure decision is in
the new `src/gesoft_file_join.rs`, with tests that run on any host. Any other
login, and a file transfer without such a control, still asks for Accept.
A control on which the customer switched File off in the connection manager is
not joined (Accept again), and when the last control of a viewer session ends,
the file transfers of that session are closed with it.

### `src/server/connection.rs` — modified 2026-11-10

Feature-gated hooks, all calling `gesoft_support`: in the click-approval branch
of the login handler (`file_transfer_joins_control`), on the customer's File
switch (`note_file_permission`), as an authorized connection is dropped
(`end_joined_file_transfers`), and a `Close` arm on the authorized-connection
channel that ends a joined file transfer. When the join holds, the connection is
authorized as the customer's Accept of that control was (audited as a click,
no 2FA prompt, `send_logon_response_and_keep_alive`), and the connection manager
is told it is authorized, so the customer sees it and can disconnect it. A file
transfer can join a control in the brief moment between its close and its
removal from the list of authorized connections; it is closed as that control
is dropped. Builds without `gesoft-support` are unchanged.

### `flutter/lib/common.dart`, `flutter/lib/desktop/pages/desktop_home_page.dart`, `flutter/windows/runner/main.cpp` — modified 2026-10-02

These read a runtime display name, accent colour, logo and icon from the
session environment. With no brand configuration the existing application name,
blue accent and generic artwork remain. The supporting platform-specific
readers are new `flutter/lib/native/brand.dart` and `flutter/lib/web/brand.dart`.

### `Cargo.toml`, `build.py`, `src/lib.rs`, `src/core_main.rs` — modified 2026-10-02

These opt the RustDesk application into the support feature only when the
Helpdesk API base is set at build time (since 2026-11-11: when
`GESOFT_SUPPORT_BUILD=1`; `src/lib.rs` also declares `gesoft_profile`). The shared startup path has a thin
feature-gated hook; builds without the feature follow the previous path.
`src/lib.rs` also declares `gesoft_file_join` under the same feature (2026-11-10).

### `libs/portable/build.rs`, `flutter/windows/runner/Runner.rc` (via the support workflow) — modified 2026-11-11

Under `GESOFT_SUPPORT_BUILD=1` the portable executable's version info names
the product: CompanyName `Gesoft`, ProductName `Gesoft Support`,
OriginalFilename `GesoftSupport.exe`, and a copyright line that keeps
upstream's notice and adds Gesoft's for its modifications. The support
workflow rewrites the same six identity strings of `Runner.rc` before the
Flutter build, so the extracted `GesoftSupport.exe` says the same; version
numbers, binary name and icons stay upstream's. Other builds are unchanged.
`build.rs` also declares its inputs for Cargo's change detection.

### `libs/portable/Cargo.toml`, `generate.py`, `src/main.rs`, build workflow — modified 2026-10-02

These opt the portable packer into the support handoff when a Helpdesk API base
is provided at build time (since 2026-11-11: when `GESOFT_SUPPORT_BUILD=1`; the
workflow builds once for both profiles and appends
`HELPDESK-PROFILES:v2;live=<api>|<host>|<key>;lab=local` (v1 with the lab values until later on 2026-11-11) to the portable exe after
checking that the DLL carries every profile value). Builds without the feature continue through the
upstream portable entry point.

### `src/platform/windows.rs` — modified 2026-11-11

`get_license_from_exe_name` returns an error in builds with `gesoft-support`, so
a file name such as `rustdesk-host=…,key=….exe` cannot replace the compiled
profile's server, key or API server (it is read by `bootstrap()`, `get_key()`
and `get_api_server()`). Builds without the feature are unchanged.

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

Worth being precise about branding. `APP_NAME` is a product string, and since
2026-11-11 the support build's version info (both executables) names Gesoft
Support, keeping upstream's copyright notice beside Gesoft's for its
modifications. Those are the compiled product strings changed. The icons, the
artwork, the window chrome, the translation catalogues and every other
user-facing string are upstream's, unmodified. The `LICENCE` file at the root of this
repository is upstream's AGPL-3.0, unaltered.

Where the upstream name still appears in the interface, that is upstream's
text, not a claim by us. Where our name appears, it is a namespace for
configuration and process names rather than a rebrand of the application. See
`LICENCE-COMPLIANCE.md` for how we treat the source obligations that follow
from distributing this.
