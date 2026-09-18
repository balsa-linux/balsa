pub mod disko;
pub mod generate;
pub mod plan;

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;

pub use generate::generate;
pub use plan::BalsaInstallPlan;

/// The files that make up a generated system, keyed by name relative to the output directory
#[derive(Debug, Clone)]
pub struct GeneratedConfig {
    pub files: BTreeMap<String, String>,
}

impl GeneratedConfig {
    /// Creates dir if needed and writes every file, overwriting.
    pub fn write_to(&self, dir: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dir)?;
        for (name, body) in &self.files {
            fs::write(dir.join(name), body)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GenerationResult {
    pub config: GeneratedConfig,
    /// Non-fatal notes for a UI, e.g. a CachyOS kernel that will miss the cache
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub enum Error {
    Validation(Vec<String>),
    Template(tera::Error),
    Plan(toml::de::Error),
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Validation(errs) => {
                write!(f, "invalid install plan:")?;
                for e in errs {
                    write!(f, "\n  - {e}")?;
                }
                Ok(())
            }
            Error::Template(e) => write!(f, "template error: {e}"),
            Error::Plan(e) => write!(f, "could not parse plan: {e}"),
            Error::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

pub fn load_plan(path: &Path) -> Result<BalsaInstallPlan, Error> {
    let text = fs::read_to_string(path)?;
    toml::from_str(&text).map_err(Error::Plan)
}
