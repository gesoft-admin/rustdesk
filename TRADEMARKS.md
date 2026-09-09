# Trademarks and brand assets

This repository is distributed under the GNU Affero General Public License
version 3 (AGPL-3.0).

The AGPL governs copyright permissions in the covered software. It does not,
by itself, grant permission to use trade names, trademarks or service marks.
Trademark rights, where applicable, are separate from the copyright licence.

This notice is informational. It does not add restrictions to the AGPL licence
for the covered software.

## RustDesk

"RustDesk" and the RustDesk logo are names and marks associated with the
upstream RustDesk project. This fork claims no ownership in them and makes no
claim of affiliation, sponsorship or endorsement by the RustDesk project or
its owners.

References to RustDesk in this repository, our source documentation and our
download/support documentation identify the upstream software from which this
fork is derived and provide accurate provenance for the modified software.

The upstream project is available at:

https://github.com/rustdesk/rustdesk

The source of this fork is available at:

https://github.com/gesoft-admin/rustdesk

### What we changed

Our build changes the application namespace `APP_NAME` from:

`RustDesk`

to:

`GesoftSupport`

The technical reason is documented in `CHANGES-GESOFT.md`.

`APP_NAME` affects application configuration paths and related runtime names.
Using a distinct namespace prevents our support client from writing into the
configuration namespace of an independently installed RustDesk client on the
same computer.

The portable payload executable name is changed correspondingly because the
portable self-extractor derives its extraction directory from that filename.

This namespace change is intended to isolate the two installations. It is not
intended to conceal the origin of the software.

### Upstream user interface and artwork

Unless otherwise documented in `CHANGES-GESOFT.md`, this fork does not
currently attempt a complete visual rebranding of the RustDesk user interface.

Some icons, artwork, interface elements and user-facing strings remain derived
from upstream RustDesk.

Copyright permissions applicable to files contained in the covered RustDesk
source distribution are governed by their applicable licences and notices.
Any trademark significance associated with names, logos or other identifiers
is a separate matter and no additional trademark rights are claimed by this
fork.

The modified nature and provenance of the software should remain clear to
recipients.

## Gesoft

Names and marks associated with Gesoft, including `Gesoft` and
`GesoftSupport`, may be used in this repository to identify our fork,
application namespace or deployment.

Their presence in AGPL-licensed source code does not remove that source code
from the AGPL and does not restrict the rights granted by the AGPL with
respect to the covered software.

However, no separate licence to use Gesoft trademarks, logos or other
Gesoft-specific brand assets as trademarks is granted by the AGPL software
licence.

Third-party distributors of this fork should use branding and identifiers
appropriate to their own distribution and should not represent their builds
as official Gesoft products or services unless separately authorized.

The application namespace is configured in:

`libs/hbb_common/src/config.rs`

and the portable payload executable name is configured by the corresponding
Windows build/packaging process.

## Gesoft-operated infrastructure

This fork may contain configuration values referring to Gesoft-operated
infrastructure, including rendezvous server addresses and the corresponding
public server key.

Their presence in the source code is required to reproduce the corresponding
build and does not constitute authorization to use Gesoft-operated services or
infrastructure.

Distributors operating their own deployment should replace:

* `RENDEZVOUS_SERVERS`
* `RS_PUB_KEY`

with values for infrastructure they are authorized to operate or use.

The public server key is not a secret. No server private keys, credentials or
other infrastructure secrets are included in this repository.

## Amyuni

Windows distributions may include the Amyuni Technologies `usbmmidd_v2`
component under its own applicable licence and notices.

See `LICENCE-COMPLIANCE.md` for the exact component and distribution
information applicable to our builds.

No ownership of or additional rights in Amyuni names or marks are claimed by
this project.
