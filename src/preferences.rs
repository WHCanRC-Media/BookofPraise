use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub copyright_accepted: bool,
    pub lyrics_magnification: f64,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            copyright_accepted: false,
            lyrics_magnification: 1.0,
        }
    }
}

fn path() -> PathBuf {
    crate::render_ly::data_dir().join("preference.yaml")
}

pub fn load() -> Preferences {
    match std::fs::read_to_string(path()) {
        Ok(s) => serde_yaml::from_str(&s).unwrap_or_default(),
        Err(_) => Preferences::default(),
    }
}

pub fn save(prefs: &Preferences) -> std::io::Result<()> {
    let p = path();
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let yaml = serde_yaml::to_string(prefs)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let yaml = yaml.strip_prefix("---\n").unwrap_or(&yaml).to_string();
    std::fs::write(&p, yaml)
}
