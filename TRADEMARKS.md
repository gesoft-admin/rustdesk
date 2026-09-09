# Trademarks and brand assets

This software is licensed under AGPL-3.0. Names, logos, trademarks and brand
assets are not: the licence grants rights to the code, and grants nobody
permission to use anyone's marks.

## RustDesk

"RustDesk" and the RustDesk logo belong to their owners. This fork claims no
rights in them, and nothing here asserts any.

Where this repository, our download page and our support tool name RustDesk,
they do it to say truthfully what the software is and where its source can be
found — which is what AGPL-3.0 §5 and §6 require of us. It is not a claim of
affiliation, sponsorship or endorsement, and there is none.

### What we changed, and what we did not

We changed one product string, `APP_NAME`, from `RustDesk` to `GesoftSupport`.
The reason is technical and is recorded in `CHANGES-GESOFT.md`: `APP_NAME`
decides the configuration directory, the names of the files in it and the
process name, so leaving it as upstream's meant our client wrote into
`%APPDATA%\RustDesk` and overwrote the profile of a customer who already used
RustDesk. Renaming it moves us out of their way.

We did **not** restyle the application. The icons, the artwork, the window
chrome and every user-facing string are upstream's, unmodified. So a customer
running our build sees RustDesk's interface, under RustDesk's icon, in a
process called `GesoftSupport.exe`.

We would rather state that plainly than let it be discovered. The name change
is a namespace, not a rebrand, and it is not intended to obscure what the
software is or where it came from — which is why the download page and the
tool's own window both name RustDesk and link to both the upstream project and
our fork.

If RustDesk's owners consider any use of their name or artwork here
inappropriate, we will change it on request.

## Gesoft

Gesoft names, logos, trademarks and brand assets are **not** licensed under the
AGPL-3.0 licence covering this software.

`GesoftSupport` appears here as an application namespace. If you build from this
fork, change it to your own before distributing: it is `APP_NAME` in
`libs/hbb_common/src/config.rs`, and the payload executable name in
`.github/workflows/support-windows-build.yml` has to match it, because the
self-extractor derives its target directory from that filename. Do not present
a build as Gesoft's.

The rendezvous server and public key compiled into this fork
(`RENDEZVOUS_SERVERS`, `RS_PUB_KEY`) point at our infrastructure. Change those
too; they are not an invitation to use it.

## Amyuni

The Windows builds bundle Amyuni Technologies' `usbmmidd_v2` driver under its
own licence. See `LICENCE-COMPLIANCE.md`. No rights in Amyuni's marks are
claimed or granted here either.
