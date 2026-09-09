# Licence compliance notes

This file records how we handle the parts of this source tree whose licence
status is not stated in the usual place. It documents a technical compliance
approach and is not a legal opinion.

The work as a whole is distributed under the GNU Affero General Public License
v3.0, in `LICENCE` at the root of this repository. Where a component is silent
about its own licence, the section below says what we do about it and why.

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
  copyright one. It is also a standing reason to check whether the feature is
  reachable at all in the support client before assuming the clause is inert.

Clause 3 forbids removing or altering the notice from any distribution, which
is why `License.txt` ships beside the driver and must keep shipping there.

### Acknowledgment

This product includes the `usbmmidd_v2` virtual display driver, Copyright
2014-2021 Amyuni Technologies Inc., <https://www.amyuni.com>, used under the
terms of its accompanying licence and redistributed unmodified.
