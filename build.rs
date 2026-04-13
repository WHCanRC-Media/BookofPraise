#[cfg(target_os = "windows")]
use std::collections::HashSet;
use std::path::PathBuf;

/// Build script entry point. Compiles the Windows icon resource, copies runtime
/// DLLs (Windows only), and copies song data directories.
fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    println!("cargo:rerun-if-env-changed=BOP_PAT");
    let pat = std::env::var("BOP_PAT").unwrap_or_default();
    let trimmed = pat.trim();
    println!("cargo:warning=build.rs saw BOP_PAT len={} trimmed_len={}", pat.len(), trimmed.len());
    println!("cargo:rustc-env=BOP_PAT={}", trimmed);

    #[cfg(target_os = "windows")]
    {
        let out = std::env::var("OUT_DIR").unwrap();
        let icon = format!("{manifest}/assets/icon.ico").replace('\\', "/");
        let rc_path = format!("{out}/icon.rc");
        let res_path = format!("{out}/icon.res");
        std::fs::write(&rc_path, format!("IDI_ICON1 ICON \"{icon}\"\n")).unwrap();
        let status = std::process::Command::new("windres")
            .args(["--use-temp-file", &rc_path, "-O", "coff", "-o", &res_path])
            .status()
            .expect("windres not found");
        assert!(status.success(), "windres failed");
        println!("cargo:rustc-link-arg={res_path}");

        copy_dlls(&manifest);
    }

    copy_data_dirs(&manifest);
}

#[cfg(target_os = "windows")]
/// Recursively resolve and copy all required UCRT64 DLLs to the target directory
/// using `ldd` to walk transitive dependencies.
fn copy_dlls(manifest: &str) {
    let prefix = std::env::var("MSYSTEM_PREFIX")
        .unwrap_or_else(|_| "C:/msys64/ucrt64".into());
    let bin_dir = PathBuf::from(&prefix).join("bin");
    let profile = std::env::var("PROFILE").unwrap();
    let target_dir = PathBuf::from(manifest).join("target").join(&profile);

    // Seed DLLs: our direct native dependencies
    let seeds = [
        "libgtk-4-1.dll",
        "libpng16-16.dll",
        "libjpeg-8.dll",
        "libtiff-6.dll",
    ];

    let mut all_dlls = HashSet::new();
    let mut queue: Vec<String> = seeds.iter().map(|s| s.to_string()).collect();

    // Recursively resolve transitive dependencies via ldd
    while let Some(dll) = queue.pop() {
        if !all_dlls.insert(dll.clone()) {
            continue;
        }
        let dll_path = bin_dir.join(&dll);
        if !dll_path.exists() {
            continue;
        }
        if let Ok(output) = std::process::Command::new("ldd")
            .arg(&dll_path)
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                // ldd format: "name.dll => /path/to/name.dll (0x...)"
                if parts.len() >= 3 && parts[2].contains("ucrt64") {
                    let name = parts[0].to_string();
                    if !all_dlls.contains(&name) {
                        queue.push(name);
                    }
                }
            }
        }
    }

    // Copy all resolved DLLs to the target directory
    std::fs::create_dir_all(&target_dir).expect("failed to create target dir");
    for dll in &all_dlls {
        let src = bin_dir.join(dll);
        let dst = target_dir.join(dll);
        if src.exists() {
            let should_copy = match (src.metadata(), dst.metadata()) {
                (Ok(s), Ok(d)) => s.modified().ok() > d.modified().ok(),
                (Ok(_), Err(_)) => true,
                _ => false,
            };
            if should_copy {
                std::fs::copy(&src, &dst)
                    .unwrap_or_else(|e| panic!("failed to copy DLL {}: {e}", src.display()));
            }
        }
    }
}

/// Copy the `lilypond/` and `photos/` data directories into the build target
/// directory so they are available at runtime.
fn copy_data_dirs(manifest: &str) {
    let profile = std::env::var("PROFILE").unwrap();
    let manifest = PathBuf::from(manifest);
    let target_dir = manifest.join("target").join(&profile);

    // In release builds, lilypond/ and photos/ are downloaded at runtime
    // by the updater. Only bundle them for local debug builds.
    if profile == "debug" {
        for dir_name in &["lilypond", "photos"] {
            let src_dir = manifest.join(dir_name);
            if src_dir.exists() {
                copy_dir_recursive(&src_dir, &target_dir.join(dir_name));
            }
        }
    }
}


/// Recursively copy a directory tree, only overwriting files whose source is
/// newer than the destination.
fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) {
    std::fs::create_dir_all(dst)
        .unwrap_or_else(|e| panic!("failed to create dir {}: {e}", dst.display()));
    let entries = std::fs::read_dir(src)
        .unwrap_or_else(|e| panic!("failed to read dir {}: {e}", src.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("failed to read entry in {}: {e}", src.display()));
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path);
        } else {
            let should_copy = match (src_path.metadata(), dst_path.metadata()) {
                (Ok(s), Ok(d)) => s.modified().ok() > d.modified().ok(),
                (Ok(_), Err(_)) => true,
                _ => false,
            };
            if should_copy {
                std::fs::copy(&src_path, &dst_path)
                    .unwrap_or_else(|e| panic!("failed to copy {}: {e}", src_path.display()));
            }
        }
    }
}
