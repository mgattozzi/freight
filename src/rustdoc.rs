use super::BoxError;
use super::Result;
use crate::rustc::Edition;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

pub struct RustDoc {
    edition: Edition,
    crate_name: String,
    lib_path: PathBuf,
    out_path: Option<PathBuf>,
}

impl RustDoc {
    pub fn new(
        edition: Edition,
        crate_name: impl Into<String>,
        lib_path: impl Into<PathBuf>,
        out_path: Option<impl Into<PathBuf>>,
    ) -> Self {
        Self {
            edition,
            crate_name: crate_name.into(),
            lib_path: lib_path.into(),
            out_path: out_path.map(|path| path.into()),
        }
    }
    pub fn doc(&self, path: impl AsRef<Path>) -> Result<ExitStatus> {
        let output = self.out_path.as_ref().ok_or_else(|| {
            BoxError::from("The output path should be specified. This is a freight bug.")
        })?;
        let path = path.as_ref();
        let exit_status = Command::new("rustdoc")
            .arg(path)
            .arg("--crate-name")
            .arg(&self.crate_name)
            .arg("--edition")
            .arg(self.edition.to_string())
            .arg("-L")
            .arg(&self.lib_path)
            .arg("--out-dir")
            .arg(output)
            .spawn()?
            .wait()?;
        Ok(exit_status)
    }
    pub fn test(&self, path: impl AsRef<Path>) -> Result<ExitStatus> {
        let path = path.as_ref();
        let exit_status = Command::new("rustdoc")
            .arg("--test")
            .arg(path)
            .arg("--crate-name")
            .arg(&self.crate_name)
            .arg("--edition")
            .arg(self.edition.to_string())
            .arg("-L")
            .arg(&self.lib_path)
            .spawn()?
            .wait()?;
        Ok(exit_status)
    }
}
