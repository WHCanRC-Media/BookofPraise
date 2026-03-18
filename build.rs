fn main() {
    #[cfg(target_os = "windows")]
    {
        let out = std::env::var("OUT_DIR").unwrap();
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let icon = format!("{manifest}/assets/icon.ico").replace('\\', "/");
        let rc_path = format!("{out}/icon.rc");
        let res_path = format!("{out}/icon.res");
        std::fs::write(&rc_path, format!("IDI_ICON1 ICON \"{icon}\"\n")).unwrap();
        let status = std::process::Command::new("windres")
            .args([&rc_path, "-O", "coff", "-o", &res_path])
            .status()
            .expect("windres not found");
        assert!(status.success(), "windres failed");
        println!("cargo:rustc-link-arg={res_path}");
    }
}
