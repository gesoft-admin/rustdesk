# Licence compliance notes

This file records how we handle the parts of this source tree whose licence
status is not stated in the usual place. It documents a technical compliance
approach and is not a legal opinion.

The work as a whole is distributed under the GNU Affero General Public License
v3.0, in `LICENCE` at the root of this repository. Where a component is silent
about its own licence, the section below says what we do about it and why.

## Distributing the binaries

We serve compiled Windows builds of this tree to customers, over the web, from
our own helpdesk service. That is *conveying* object code under AGPL-3.0, and
it carries two obligations that the licence text alone does not discharge.

**§5(a) — say that you modified it, and when.** `CHANGES-GESOFT.md` at the root
of this repository lists every change this fork and our `hbb_common` fork make
to upstream, with dates, and states that nothing else is changed. The two
source files we actually modified — `src/common.rs` and
`libs/hbb_common/src/config.rs` — each carry a short notice at the top pointing
at it, so a reader who opens only the file still sees it.

**§6 — tell the recipient where the source is.** Publishing the fork is not by
itself enough: the person receiving the binary has to be told. So the offer
appears where the binary is actually handed over, in the customer's own
language:

* On the download page they click, as a footer naming RustDesk, the AGPL-3.0,
  and `github.com/gesoft-admin/rustdesk`.
* In the support tool's own window, printed under the banner before anything
  else happens, because a customer may keep the executable and not the page.

Neither is a repository link buried in documentation the customer would never
open. If either is removed, the obligation is not met.

**Keeping the offer true.** The source we point at has to be the source that
built the binary the customer received. That is why builds are pinned by
sha256 alongside the commits they came from, and why the pin does not move for
a documentation change — see the note on the gitlink below.

## `hbb_common`

In this tree, `hbb_common` is consumed as a Git submodule at `libs/hbb_common`
and, inside Cargo, as a path dependency and workspace member. In our builds and
distributions it is compiled into the RustDesk binaries this repository
produces, and we do not distribute it as a standalone product. That is a
statement about how we use it, not about `hbb_common` in general.

**The observed facts, as of 2026-09-09:**

* Upstream `rustdesk/hbb_common` contains no `LICENSE` file, no `COPYING`, and
  no `NOTICE`. GitHub's licence detection reports `NONE` for it.
* Its `Cargo.toml` declares no `license` or `license-file` key. The only
  attribution it carries is `authors = ["open-trade <info@opentradesolutions.com>"]`.
* No source file in it carries a copyright header or an SPDX identifier.
* By contrast, `rustdesk/rustdesk` is AGPL-3.0, stated in its `LICENCE` file
  and detected as such by GitHub. That repository's `Cargo.toml` also declares
  no `license` key, so the file is the operative statement there too.

**What we do.** We treat `hbb_common` operationally as part of the RustDesk
source distribution: same licence, same obligations. We publish the exact
source used for our builds, at `https://github.com/gesoft-admin/hbb_common`,
and we keep our fork public for as long as we distribute binaries built from
it.

We preserve what upstream attribution exists — which is the `authors` field in
`Cargo.toml`, and nothing else, because there is nothing else. We do not add
copyright headers of our own to files we did not write.

Modifications we author in our fork are made available under AGPL-3.0, to the
extent that we hold the rights to those modifications. This does not purport to
relicense upstream code for which we do not hold copyright, and we do not claim
that the absence of a licence file independently establishes the licence status
of all historical `hbb_common` code.

**Recording the pin.** The exact `hbb_common` commit used by a distributed
build must be recorded together with the RustDesk commit and the source URL.
The commit is **not** in `.gitmodules` — that file records only the path and
the URL. It is the gitlink in the parent tree:

    git ls-tree HEAD libs/hbb_common
    # 160000 commit 2635e2a376b251efa80088187356c7d1478e6932	libs/hbb_common

    git rev-parse HEAD                     # the RustDesk commit
    git config -f .gitmodules submodule.libs/hbb_common.url

Anyone honouring a source request for one of our builds needs all three, and
`deploy/PHASE-1.2.md` in the helpdesk repository is where they are pinned per
release alongside the artifact's sha256.

`hbb_common` also carries a short `COMPLIANCE.md` pointing back here, for
whoever lands on that repository looking for a licence and finds none. That
note is a commit *ahead* of the gitlink above, and deliberately so: the pin
describes the source a released binary was built from, and it does not move
for a documentation change. A checkout of the pinned commit therefore has no
`COMPLIANCE.md` — this file is the one that travels with the distribution.

**If this changes.** Should upstream publish an explicit licence statement
covering the relevant `hbb_common` history, update this section rather than
leaving it to stand as the record.

## `usbmmidd_v2` (Amyuni virtual display driver)

The Windows builds bundle Amyuni Technologies' `usbmmidd_v2` virtual display
driver, redistributed unmodified with its own `License.txt` in the same
directory the payload extracts to. That file is the operative statement. It is
not AGPL-3.0 and is not covered by the licence of the work as a whole.

It reads like the zlib licence but differs from it in two ways that matter, so
do not treat it as zlib by analogy:

* **The acknowledgment is required, not appreciated.** zlib's clause 1 says an
  acknowledgment in the product documentation "would be appreciated". Amyuni's
  says it "would be required". We therefore carry one — see below — rather
  than relying on the bundled `License.txt` alone.
* **There is an advertising clause.** "By using this software, you accept to be
  prompted with an advertisement page which is not always under the control of
  the author. A fully commercial version is available for users who do not wisth [sic]
  to see those advertisements." This is a term our customers accept when the
  virtual display is used, so it is a disclosure obligation, not just a
  copyright one — so we checked whether our client can reach it at all. It
  cannot, as it is configured and run; see below.

Clause 3 forbids removing or altering the notice from any distribution, which
is why `License.txt` ships beside the driver and must keep shipping there.

### Whether the advertising clause can be triggered by our client

Read from the source at `5cd67169`, not observed on Windows. Three independent
reasons, any one of which is sufficient:

1. **The driver cannot be installed without administrator rights.**
   `amyuni_idd::check_install_driver` tries `deviceinstaller64.exe` first, which
   our artifact does not contain — the packed `usbmmidd_v2` directory holds only
   `usbmmIdd.inf`, `usbmmidd.cat`, `x64/usbmmIdd.dll`, `idd_instructions.txt`
   and `License.txt`. It then falls back to installing the `.inf` through
   SetupAPI, which requires elevation. Our session is attended, portable and
   unelevated: the customer presses the plain **Accept**, not "Accept and
   elevate". The call fails and is logged.
2. **Nothing in our flow asks for a virtual display.** `plug_in_headless` runs
   only when the machine reports no displays at all, which is not a customer
   laptop; `plug_in_monitor` runs only when an operator explicitly requests an
   extra display.
3. **The capability is never advertised.** `get_platform_additions` returns
   early unless `is_self_service_running()`, which checks for a Windows service
   named after `APP_NAME` — `GesoftSupport`. Our client is portable and installs
   no service, so the operator's client is never told virtual displays are
   available.

One case escapes all three: a customer whose machine **already** has the Amyuni
driver installed by some other product. There `check_install_driver` returns
early and the existing driver is used. That customer accepted Amyuni's terms
when they installed it, independently of us.

None of this reduces the redistribution obligation. We ship the files, so
`License.txt` must keep travelling with them and the acknowledgment below
stands. If the support client ever gains an elevated or installed mode, this
conclusion has to be re-checked before it ships.

### Acknowledgment

This product includes the `usbmmidd_v2` virtual display driver, Copyright
2014-2021 Amyuni Technologies Inc., <https://www.amyuni.com>, used under the
terms of its accompanying licence and redistributed unmodified.
