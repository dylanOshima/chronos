use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JobMeta {
    pub id: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
}

/// TOML requires string keys for tables, so `at` jobs are stored with their
/// job number serialised as a string (e.g. `"42"`) and converted on access.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Sidecar {
    #[serde(default)]
    pub cron: HashMap<String, JobMeta>,
    #[serde(default)]
    pub at: HashMap<String, JobMeta>,
}

impl Sidecar {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let sidecar: Sidecar = toml::from_str(&content)?;
        Ok(sidecar)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn set_cron_meta(&mut self, raw_line: &str, meta: JobMeta) {
        self.cron.insert(raw_line.to_string(), meta);
    }

    pub fn get_cron_meta(&self, raw_line: &str) -> Option<&JobMeta> {
        self.cron.get(raw_line)
    }

    pub fn set_at_meta(&mut self, job_number: u32, meta: JobMeta) {
        self.at.insert(job_number.to_string(), meta);
    }

    pub fn get_at_meta(&self, job_number: u32) -> Option<&JobMeta> {
        self.at.get(&job_number.to_string())
    }

    pub fn prune(&mut self, active_cron_lines: &[String], active_at_jobs: &[u32]) {
        self.cron.retain(|key, _| active_cron_lines.contains(key));
        let active_str: Vec<String> = active_at_jobs.iter().map(|n| n.to_string()).collect();
        self.at.retain(|key, _| active_str.contains(key));
    }

    pub fn find_cron_by_id(&self, id: &str) -> Option<String> {
        self.cron
            .iter()
            .find(|(_, meta)| meta.id.as_deref() == Some(id))
            .map(|(key, _)| key.clone())
    }

    pub fn find_at_by_id(&self, id: &str) -> Option<u32> {
        self.at
            .iter()
            .find(|(_, meta)| meta.id.as_deref() == Some(id))
            .and_then(|(key, _)| key.parse().ok())
    }
}

pub fn sidecar_path() -> PathBuf {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config")));
    config_dir.join("chronos").join("meta.toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_empty_sidecar() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("meta.toml");
        let sidecar = Sidecar::load(&path).unwrap();
        assert!(sidecar.cron.is_empty());
        assert!(sidecar.at.is_empty());
    }

    #[test]
    fn test_add_and_retrieve_cron_metadata() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("meta.toml");
        let mut sidecar = Sidecar::load(&path).unwrap();
        sidecar.set_cron_meta(
            "0 8 * * * echo hello",
            JobMeta {
                id: Some("daily".to_string()),
                description: Some("Daily job".to_string()),
                source: Some("claude".to_string()),
            },
        );
        sidecar.save(&path).unwrap();

        let reloaded = Sidecar::load(&path).unwrap();
        let meta = reloaded.get_cron_meta("0 8 * * * echo hello").unwrap();
        assert_eq!(meta.id.as_deref(), Some("daily"));
        assert_eq!(meta.source.as_deref(), Some("claude"));
    }

    #[test]
    fn test_add_and_retrieve_at_metadata() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("meta.toml");
        let mut sidecar = Sidecar::load(&path).unwrap();
        sidecar.set_at_meta(
            42,
            JobMeta {
                id: Some("retry".to_string()),
                description: Some("Retry deploy".to_string()),
                source: Some("droshima".to_string()),
            },
        );
        sidecar.save(&path).unwrap();

        let reloaded = Sidecar::load(&path).unwrap();
        let meta = reloaded.get_at_meta(42).unwrap();
        assert_eq!(meta.id.as_deref(), Some("retry"));
    }

    #[test]
    fn test_prune_stale_cron_entries() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("meta.toml");
        let mut sidecar = Sidecar::load(&path).unwrap();
        sidecar.set_cron_meta("0 8 * * * echo hello", JobMeta {
            id: Some("keep".to_string()),
            description: None,
            source: None,
        });
        sidecar.set_cron_meta("0 9 * * * echo gone", JobMeta {
            id: Some("stale".to_string()),
            description: None,
            source: None,
        });

        let active_lines = vec!["0 8 * * * echo hello".to_string()];
        let active_at: Vec<u32> = vec![];
        sidecar.prune(&active_lines, &active_at);

        assert!(sidecar.get_cron_meta("0 8 * * * echo hello").is_some());
        assert!(sidecar.get_cron_meta("0 9 * * * echo gone").is_none());
    }

    #[test]
    fn test_find_by_id() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("meta.toml");
        let mut sidecar = Sidecar::load(&path).unwrap();
        sidecar.set_cron_meta("0 8 * * * echo hello", JobMeta {
            id: Some("daily".to_string()),
            description: None,
            source: None,
        });
        let result = sidecar.find_cron_by_id("daily");
        assert_eq!(result, Some("0 8 * * * echo hello".to_string()));
    }
}
