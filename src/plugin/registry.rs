use std::collections::HashMap;

use anyhow::{Result, bail};

pub trait Plugin {
    fn name(&self) -> &str;

    fn install(&self, version: Option<&str>) -> Result<String>;

    fn uninstall(&self, version: &str) -> Result<()>;

    fn list_installed(&self) -> Result<Vec<String>>;

    fn use_version(&self, version: &str, set_default: bool) -> Result<()>;

    fn current_version(&self) -> Result<Option<String>> {
        Ok(None)
    }

    fn binary_path(&self, version: &str) -> Result<String>;

    fn latest_version(&self) -> Result<String>;

    fn list_remote_versions(&self) -> Result<Vec<String>> {
        bail!("Remote version listing is not supported for this plugin")
    }

    fn format_installed(&self, versions: &[String]) -> Result<Vec<String>> {
        Ok(versions.to_vec())
    }

    fn post_install(&self, _version: &str) -> Result<()> {
        Ok(())
    }
}

pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        PluginRegistry {
            plugins: HashMap::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.insert(plugin.name().to_string(), plugin);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(Box::as_ref)
    }

    pub fn list_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.plugins.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }
}
