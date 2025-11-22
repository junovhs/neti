use crate::error::Result;
use std::collections::HashSet;
use std::fmt;
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum BuildSystemType {
    Rust,
    Node,
    Python,
    Go,
    CMake,
    Conan,
}

impl fmt::Display for BuildSystemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

pub struct Detector;

impl Detector {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Detects build systems in the file list.
    ///
    /// # Errors
    ///
    /// Returns error if underlying detection logic fails.
    pub fn detect_build_systems(
        &self,
        files: &[std::path::PathBuf],
    ) -> Result<Vec<BuildSystemType>> {
        let mut detected = HashSet::new();

        for file in files {
            if Self::is_cargo(file) {
                detected.insert(BuildSystemType::Rust);
            }
            if Self::is_npm(file) {
                detected.insert(BuildSystemType::Node);
            }
            if Self::is_python(file) {
                detected.insert(BuildSystemType::Python);
            }
            if Self::is_go(file) {
                detected.insert(BuildSystemType::Go);
            }
            if Self::is_cmake(file) {
                detected.insert(BuildSystemType::CMake);
            }
            if Self::is_conan(file) {
                detected.insert(BuildSystemType::Conan);
            }
        }

        Ok(detected.into_iter().collect())
    }

    fn is_cargo(path: &Path) -> bool {
        path.ends_with("Cargo.toml")
    }
    fn is_npm(path: &Path) -> bool {
        path.ends_with("package.json")
    }
    fn is_python(path: &Path) -> bool {
        matches!(
            path.file_name().and_then(|n| n.to_str()),
            Some("requirements.txt" | "pyproject.toml" | "Pipfile")
        )
    }
    fn is_go(path: &Path) -> bool {
        path.ends_with("go.mod")
    }
    fn is_cmake(path: &Path) -> bool {
        let s = path.to_string_lossy();
        s.contains("CMakeLists.txt") || s.ends_with(".cmake")
    }
    fn is_conan(path: &Path) -> bool {
        matches!(
            path.file_name().and_then(|n| n.to_str()),
            Some("conanfile.txt" | "conanfile.py")
        )
    }
}

impl Default for Detector {
    fn default() -> Self {
        Self::new()
    }
}
