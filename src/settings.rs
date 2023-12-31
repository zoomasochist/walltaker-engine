use anyhow::Context;
use serde::{Serialize, Deserialize};
use windows::Win32::UI::Shell::DWPOS_FILL;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub subscribed: Vec<usize>,
    pub method: i32,
    pub notifications: bool,
}

impl Settings {
    pub fn load_or_new() -> Self {
        let out = out_path();

        let _ = std::fs::create_dir_all(out.parent().unwrap());
        std::fs::File::open(out)
            .context("reading file")
            .and_then(|f| serde_json::from_reader(f).context("reading config"))
            .unwrap_or_default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let out = out_path();

        // to be safe
        let _ = std::fs::create_dir_all(out.parent().unwrap());
        let out = std::fs::File::create(out)?;
        serde_json::to_writer_pretty(out, &self)?;

        Ok(())
    }
}

fn out_path() -> std::path::PathBuf {
    directories::BaseDirs::new()
        .unwrap()
        .config_dir()
        .join("walltaker-engine")
        .join("walltaker-engine.json")
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            subscribed: Vec::new(),
            method: DWPOS_FILL.0,
            notifications: true,
        }
    }
}