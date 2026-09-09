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

No other file in either repository is changed — five files in `rustdesk` and
one in `hbb_common`, as listed above.

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
