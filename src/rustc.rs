use super::BoxError;
use super::Result;
use std::fmt;
use std::fmt::Display;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

pub struct Rustc {
    edition: Edition,
    crate_type: CrateType,
    crate_name: String,
    out_dir: PathBuf,
    lib_dir: PathBuf,
    cfg: Vec<String>,
    externs: Vec<String>,
    test: bool,
}

impl Rustc {
    /// Create a builder type to build up commands to then invoke rustc with.
    /// ```
    /// # use std::error::Error;
    /// # use freight::rustc::Rustc;
    /// # use freight::rustc::Edition;
    /// # use freight::rustc::CrateType;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    ///     let builder = Rustc::builder()
    ///       .edition(Edition::E2021)
    ///       .crate_type(CrateType::Bin)
    ///       .crate_name("freight")
    ///       .out_dir(".")
    ///       .lib_dir(".");
    /// #   Ok(())
    /// # }
    /// ```
    pub fn builder() -> RustcBuilder {
        RustcBuilder {
            ..Default::default()
        }
    }

    pub fn run(self, path: &str) -> Result<()> {
        Command::new("rustc")
            .arg(path)
            .arg("--edition")
            .arg(self.edition.to_string())
            .arg("--crate-type")
            .arg(self.crate_type.to_string())
            .arg("--crate-name")
            .arg(self.crate_name)
            .arg("--out-dir")
            .arg(self.out_dir)
            .arg("-L")
            .arg(self.lib_dir)
            .args(if self.test { vec!["--test"] } else { vec![] })
            .args(
                self.externs
                    .into_iter()
                    .map(|r#extern| ["--extern".into(), r#extern])
                    .flatten(),
            )
            .args(
                self.cfg
                    .into_iter()
                    .map(|cfg| ["--cfg".into(), cfg])
                    .flatten(),
            )
            .spawn()?
            .wait()?;

        Ok(())
    }
}

#[derive(Default)]
pub struct RustcBuilder {
    edition: Option<Edition>,
    crate_type: Option<CrateType>,
    crate_name: Option<String>,
    out_dir: Option<PathBuf>,
    lib_dir: Option<PathBuf>,
    cfg: Vec<String>,
    externs: Vec<String>,
    test: bool,
}

impl RustcBuilder {
    pub fn edition(mut self, edition: Edition) -> Self {
        self.edition = Some(edition);
        self
    }
    pub fn out_dir(mut self, out_dir: impl Into<PathBuf>) -> Self {
        self.out_dir = Some(out_dir.into());
        self
    }
    pub fn lib_dir(mut self, lib_dir: impl Into<PathBuf>) -> Self {
        self.lib_dir = Some(lib_dir.into());
        self
    }
    pub fn crate_name(mut self, crate_name: impl Into<String>) -> Self {
        self.crate_name = Some(crate_name.into());
        self
    }
    pub fn crate_type(mut self, crate_type: CrateType) -> Self {
        self.crate_type = Some(crate_type);
        self
    }
    pub fn cfg(mut self, cfg: impl Into<String>) -> Self {
        self.cfg.push(cfg.into());
        self
    }
    pub fn externs(mut self, r#extern: impl Into<String>) -> Self {
        self.externs.push(r#extern.into());
        self
    }

    pub fn test(mut self, test: bool) -> Self {
        self.test = test;
        self
    }

    pub fn done(self) -> Rustc {
        Rustc {
            edition: self.edition.unwrap_or(Edition::E2015),
            crate_type: self.crate_type.expect("Crate type given"),
            crate_name: self.crate_name.expect("Crate name given"),
            out_dir: self.out_dir.expect("Out dir given"),
            lib_dir: self.lib_dir.expect("Lib dir given"),
            cfg: self.cfg,
            externs: self.externs,
            test: self.test,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edition {
    E2015,
    E2018,
    E2021,
}

impl Display for Edition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let edition = match self {
            Self::E2015 => "2015",
            Self::E2018 => "2018",
            Self::E2021 => "2021",
        };
        write!(f, "{edition}")
    }
}

impl FromStr for Edition {
    type Err = BoxError;
    fn from_str(input: &str) -> Result<Self> {
        match input {
            "2015" => Ok(Self::E2015),
            "2018" => Ok(Self::E2018),
            "2021" => Ok(Self::E2021),
            edition => Err(format!("Edition {edition} is not supported").into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrateType {
    Bin,
    Lib,
    RLib,
    DyLib,
    CDyLib,
    StaticLib,
    ProcMacro,
}

impl Display for CrateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let crate_type = match self {
            Self::Bin => "bin",
            Self::Lib => "lib",
            Self::RLib => "rlib",
            Self::DyLib => "dylib",
            Self::CDyLib => "cdylib",
            Self::StaticLib => "staticlib",
            Self::ProcMacro => "proc-macro",
        };
        write!(f, "{crate_type}")
    }
}

impl FromStr for CrateType {
    type Err = BoxError;
    fn from_str(input: &str) -> Result<Self> {
        match input {
            "bin" => Ok(Self::Bin),
            "lib" => Ok(Self::Lib),
            "rlib" => Ok(Self::RLib),
            "dylib" => Ok(Self::DyLib),
            "cdylib" => Ok(Self::CDyLib),
            "staticlib" => Ok(Self::StaticLib),
            "proc-macro" => Ok(Self::ProcMacro),
            crate_type => Err(format!("Crate Type {crate_type} is not supported").into()),
        }
    }
}

#[test]
fn edition_from_str() -> Result<()> {
    let e2015 = Edition::from_str("2015")?;
    assert_eq!(e2015, Edition::E2015);
    let e2018 = Edition::from_str("2018")?;
    assert_eq!(e2018, Edition::E2018);
    let e2021 = Edition::from_str("2021")?;
    assert_eq!(e2021, Edition::E2021);
    if !Edition::from_str("\"2015\"").is_err() {
        panic!("bad string parsed correctly");
    }

    Ok(())
}

#[test]
fn crate_type_from_str() -> Result<()> {
    let bin = CrateType::from_str("bin")?;
    assert_eq!(bin, CrateType::Bin);
    let lib = CrateType::from_str("lib")?;
    assert_eq!(lib, CrateType::Lib);
    let rlib = CrateType::from_str("rlib")?;
    assert_eq!(rlib, CrateType::RLib);
    let dylib = CrateType::from_str("dylib")?;
    assert_eq!(dylib, CrateType::DyLib);
    let cdylib = CrateType::from_str("cdylib")?;
    assert_eq!(cdylib, CrateType::CDyLib);
    let staticlib = CrateType::from_str("staticlib")?;
    assert_eq!(staticlib, CrateType::StaticLib);
    let proc_macro = CrateType::from_str("proc-macro")?;
    assert_eq!(proc_macro, CrateType::ProcMacro);
    if !CrateType::from_str("proc-marco").is_err() {
        panic!("bad string parsed correctly");
    }

    Ok(())
}
