# Gesoft Support and Microsoft Defender: what not to put back

Unsigned support builds were flagged by Defender's machine-learning detections
(`Trojan:Win32/Wacatac.B!ml`, `Trojan:Win32/Bearfoos.A!ml`) from 8 to 11
November 2026. ESET found nothing in the same files. The first build that came
out clean was run 43027535071 (commit ef47ec179, sha256 `07ea4b6e…`), scanned
on 11 November 2026 on the win10 lab VM with real-time protection on, no
exclusion, a `ZoneId=3` mark of the web and an on-demand scan, next to the
flagged single build `8ebf8da2` as the positive control.

Several things changed between the last flagged build and the clean one, so no
single one is proven to be the trigger. They are all kept as safeguards, and the
support workflow's "Mark and verify" step fails a build that brings one back.

| Safeguard | Flagged builds | Clean build | CI check |
|---|---|---|---|
| No private address (lab IP) compiled into the packer; the lab profile reads its endpoints from `%ProgramData%\GesoftSupportLab\lab.conf` on the test machine | `192.168.99.110` in plain text | none | no RFC 1918 IPv4 literal in the packer exe |
| The packer does not read `Zone.Identifier` (the profile comes from the file name only) | read it | does not | no `Zone.Identifier` string in the packer exe |
| Version info names Gesoft Support, not RustDesk (`CompanyName` Gesoft, `ProductName` Gesoft Support, `OriginalFilename` GesoftSupport.exe), on both the packer and the payload exe | ProductName RustDesk, OriginalFilename rustdesk.exe, no CompanyName | Gesoft Support | `Gesoft Support` and `GesoftSupport.exe` present, no UTF-16 `RustDesk` or `rustdesk.exe` in the packer exe |

Observations that did not explain the detections, kept for the record:

- The old production build `f7e4200b` was clean while still reading
  `Zone.Identifier` and carrying RustDesk version info, but without a private
  address. Every recent flagged build carried the lab address.
- The payload files of a flagged build (`GesoftSupport.exe`, `librustdesk.dll`,
  `flutter_windows.dll`) scanned clean on their own: the detections were on the
  packed outer exe.
- An uncompressed ("stored") packing of the clean build was also clean. It was
  built once, in run 43027535071, as a comparison; the workflow no longer builds
  it. `generate.py --stored` and the reader's support for it stay in the code,
  unused by CI, because the clean binary was compiled with them.

## The clean verdict did not last

The clean build `07ea4b6e` was flagged too, at 19:50 the same day
(`Wacatac.B!ml`), with the same local signatures (1.463.236.0) that had passed
it at 16:55, and under any name: a copy saved as `GesoftSupport.exe`, without a
mark of the web and not through Chrome, was flagged at once. The verdict moved
in Defender's cloud, not on the VM. In the hours before, the lab VM (cloud
protection on, automatic sample submission on) had downloaded the same exe
dozens of times under different names, by Chrome, `Invoke-WebRequest` and a
local HTTP server, and started, killed, elevated and run it as SYSTEM through
scheduled tasks. That is the pattern most likely to have sent it for cloud
analysis; it is not proven to be the trigger.

So for the lab VM:

- Cloud-delivered protection and automatic sample submission stay off, set by
  hand in Windows Security: tests must not submit our builds.
- Real-time protection is off while functional tests run, switched by hand,
  never by a script, and back on afterwards.
- The Defender check is a separate, single scan of the candidate build at the
  end. It shows what the local engine says, not what the cloud will decide
  later: only code signing builds reputation that carries across builds.

Before shipping a new support build:

1. Let CI's safeguard checks pass; do not weaken them to get a build through.
2. Scan the exe on the lab VM as above, next to a known flagged build, without
   an exclusion. The visual end-to-end test also runs without one.
3. Do not add Defender exclusions from scripts: Defender itself flags a script
   that adds one (`Trojan:Win32/PowExcEnv.ZA!MTB`).
4. If a build is flagged anyway, report it to Microsoft as a false positive and
   compare it with the last clean build before changing anything else.
