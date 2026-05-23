//! In-memory registry of installed projects with state tracking.
//!
//! On host startup, scans `<data_dir>/projects/<id>/project.yaml` and loads
//! descriptors into the registry. State transitions (Installed → Starting →
//! Running → Stopping → Stopped → Failed) are tracked in memory.

use std::collections::HashMap;
use std::sync::RwLock;

use ygg_core::project::{ProjectDescriptor, ProjectId, ProjectState};

#[derive(Default)]
pub struct ProjectRegistry {
    inner: RwLock<HashMap<ProjectId, ProjectEntry>>,
}

#[derive(Clone, Debug)]
pub struct ProjectEntry {
    pub descriptor: ProjectDescriptor,
    pub state: ProjectState,
}

impl ProjectRegistry {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    /// Register a project (typically called when an installed project's
    /// project.yaml is read at host startup).
    pub fn register(&self, descriptor: ProjectDescriptor) -> anyhow::Result<()> {
        descriptor.validate()?;
        let mut guard = self.inner.write().expect("project registry poisoned");
        guard.insert(
            descriptor.project.id.clone(),
            ProjectEntry {
                descriptor,
                state: ProjectState::Installed,
            },
        );
        Ok(())
    }

    pub fn unregister(&self, id: &ProjectId) -> Option<ProjectEntry> {
        self.inner
            .write()
            .expect("project registry poisoned")
            .remove(id)
    }

    pub fn get(&self, id: &ProjectId) -> Option<ProjectEntry> {
        self.inner
            .read()
            .expect("project registry poisoned")
            .get(id)
            .cloned()
    }

    pub fn list(&self) -> Vec<ProjectEntry> {
        self.inner
            .read()
            .expect("project registry poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn set_state(&self, id: &ProjectId, state: ProjectState) -> anyhow::Result<()> {
        let mut guard = self.inner.write().expect("project registry poisoned");
        let entry = guard
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("project '{}' not registered", id))?;
        entry.state = state;
        Ok(())
    }

    /// Discover and load all projects under <data_dir>/projects/.
    pub fn load_from_disk(&self) -> anyhow::Result<usize> {
        let projects_dir = ygg_core::paths::projects_dir()?;
        if !projects_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in std::fs::read_dir(&projects_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            // Skip .archived and other dot-dirs.
            let name = match path.file_name().and_then(|s| s.to_str()) {
                Some(n) => n,
                None => continue,
            };
            if name.starts_with('.') {
                continue;
            }

            let descriptor_path = path.join("project.yaml");
            if !descriptor_path.exists() {
                continue;
            }

            let yaml = std::fs::read_to_string(&descriptor_path)?;
            let descriptor: ProjectDescriptor = serde_yaml::from_str(&yaml).map_err(|e| {
                anyhow::anyhow!("invalid project.yaml at {}: {e}", descriptor_path.display())
            })?;

            self.register(descriptor)?;
            count += 1;
        }
        Ok(count)
    }
}
