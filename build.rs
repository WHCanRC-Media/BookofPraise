use std::collections::HashSet;
use std::path::PathBuf;

fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();

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
    fetch_lilypond(&manifest);
}

#[cfg(target_os = "windows")]
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
    let _ = std::fs::create_dir_all(&target_dir);
    let mut copied = 0;
    for dll in &all_dlls {
        let src = bin_dir.join(dll);
        let dst = target_dir.join(dll);
        if src.exists() {
            // Only copy if source is newer or dest doesn't exist
            let should_copy = match (src.metadata(), dst.metadata()) {
                (Ok(s), Ok(d)) => {
                    s.modified().ok() > d.modified().ok()
                }
                (Ok(_), Err(_)) => true,
                _ => false,
            };
            if should_copy {
                if let Ok(_) = std::fs::copy(&src, &dst) {
                    copied += 1;
                }
            }
        }
    }
    println!("cargo:warning=Copied {copied} DLLs ({} total resolved) to {}", all_dlls.len(), target_dir.display());
}

fn copy_data_dirs(manifest: &str) {
    let profile = std::env::var("PROFILE").unwrap();
    let manifest = PathBuf::from(manifest);
    let target_dir = manifest.join("target").join(&profile);

    let mut copied = 0;
    for dir_name in &["lilypond", "photos"] {
        let src_dir = manifest.join(dir_name);
        if src_dir.exists() {
            copied += copy_dir_recursive(&src_dir, &target_dir.join(dir_name));
        }
    }
    println!("cargo:warning=Copied {copied} data files (lilypond + photos) to {}", target_dir.display());
}

fn fetch_lilypond(manifest: &str) {
    const VERSION: &str = "2.24.4";

    let profile = std::env::var("PROFILE").unwrap();
    let target_dir = PathBuf::from(manifest).join("target").join(&profile);
    let dest = target_dir.join("lilypond-bin");

    // Skip if already present
    if dest.join("bin").exists() {
        println!("cargo:warning=LilyPond already present at {}", dest.display());
        return;
    }

    #[cfg(target_os = "windows")]
    {
        let url = format!(
            "https://gitlab.com/lilypond/lilypond/-/releases/v{VERSION}/downloads/lilypond-{VERSION}-mingw-x86_64.zip"
        );
        let zip_path = target_dir.join("lilypond.zip");
        let status = std::process::Command::new("curl")
            .args(["-L", "-o"])
            .arg(&zip_path)
            .arg(&url)
            .status();
        if !status.is_ok_and(|s| s.success()) {
            println!("cargo:warning=Failed to download LilyPond");
            return;
        }
        let status = std::process::Command::new("unzip")
            .args(["-q", "-o"])
            .arg(&zip_path)
            .arg("-d")
            .arg(&target_dir)
            .status();
        if !status.is_ok_and(|s| s.success()) {
            println!("cargo:warning=Failed to extract LilyPond");
            return;
        }
        let extracted = target_dir.join(format!("lilypond-{VERSION}"));
        if extracted.exists() {
            let _ = std::fs::rename(&extracted, &dest);
        }
        let _ = std::fs::remove_file(&zip_path);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let url = format!(
            "https://gitlab.com/lilypond/lilypond/-/releases/v{VERSION}/downloads/lilypond-{VERSION}-linux-x86_64.tar.gz"
        );
        let tar_path = target_dir.join("lilypond.tar.gz");
        let status = std::process::Command::new("curl")
            .args(["-L", "-o"])
            .arg(&tar_path)
            .arg(&url)
            .status();
        if !status.is_ok_and(|s| s.success()) {
            println!("cargo:warning=Failed to download LilyPond");
            return;
        }
        let status = std::process::Command::new("tar")
            .args(["xzf"])
            .arg(&tar_path)
            .arg("-C")
            .arg(&target_dir)
            .status();
        if !status.is_ok_and(|s| s.success()) {
            println!("cargo:warning=Failed to extract LilyPond");
            return;
        }
        let extracted = target_dir.join(format!("lilypond-{VERSION}"));
        if extracted.exists() {
            let _ = std::fs::rename(&extracted, &dest);
        }
        let _ = std::fs::remove_file(&tar_path);
    }

    println!("cargo:warning=LilyPond {VERSION} installed to {}", dest.display());
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> usize {
    let _ = std::fs::create_dir_all(dst);
    let mut copied = 0;
    let entries = match std::fs::read_dir(src) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    for entry in entries.flatten() {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copied += copy_dir_recursive(&src_path, &dst_path);
        } else {
            let should_copy = match (src_path.metadata(), dst_path.metadata()) {
                (Ok(s), Ok(d)) => s.modified().ok() > d.modified().ok(),
                (Ok(_), Err(_)) => true,
                _ => false,
            };
            if should_copy {
                if std::fs::copy(&src_path, &dst_path).is_ok() {
                    copied += 1;
                }
            }
        }
    }
    copied
}
