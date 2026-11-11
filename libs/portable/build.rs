// Modified by Gesoft, 2026-11-11. See CHANGES-GESOFT.md.
fn main() {
    println!("cargo:rerun-if-env-changed=GESOFT_SUPPORT_BUILD");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=../../res/icon.ico");
    println!("cargo:rerun-if-changed=../../res/manifest.xml");
    #[cfg(windows)]
    {
        use std::io::Write;
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../res/icon.ico")
            .set_language(winapi::um::winnt::MAKELANGID(
                winapi::um::winnt::LANG_ENGLISH,
                winapi::um::winnt::SUBLANG_ENGLISH_US,
            ))
            .set_manifest_file("../../res/manifest.xml");
        // The support build names its own product instead of RustDesk's.
        if std::env::var("GESOFT_SUPPORT_BUILD").as_deref() == Ok("1") {
            res.set("CompanyName", "Gesoft")
                .set("ProductName", "Gesoft Support")
                .set("FileDescription", "Gesoft Support - asistenta la distanta")
                .set("InternalName", "GesoftSupport")
                .set("OriginalFilename", "GesoftSupport.exe")
                .set("LegalCopyright", "Copyright © 2026 Purslane Tech Pte. Ltd. All rights reserved. Modifications Copyright © 2026 Gesoft. AGPL-3.0.");
        }
        match res.compile() {
            Err(e) => {
                write!(std::io::stderr(), "{}", e).unwrap();
                std::process::exit(1);
            }
            Ok(_) => {}
        }
    }
}
