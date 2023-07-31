mod config;
mod logger;

use config::Manifest;
use logger::Logger;
use std::env;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

pub type Result<T> = std::result::Result<T, BoxError>;
pub type BoxError = Box<dyn Error>;

fn lib_compile(
    logger: &mut Logger,
    manifest: &Manifest,
    lib_path: &Path,
    out_dir: &Path,
) -> Result<()> {
    logger.compiling_crate(&manifest.crate_name);
    Rustc::builder()
        .edition(manifest.edition)
        .crate_type(CrateType::Lib)
        .crate_name(&manifest.crate_name)
        .out_dir(out_dir.clone())
        .lib_dir(out_dir.clone())
        .done()
        .run(lib_path.to_str().unwrap())?;
    logger.done_compiling();
    Ok(())
}

fn bin_compile(
    logger: &mut Logger,
    manifest: &Manifest,
    bin_path: &Path,
    out_dir: &Path,
    externs: &[&str],
) -> Result<()> {
    logger.compiling_bin(&manifest.crate_name);
    let mut builder = Rustc::builder()
        .edition(manifest.edition)
        .crate_type(CrateType::Bin)
        .crate_name(&manifest.crate_name)
        .out_dir(out_dir.clone())
        .lib_dir(out_dir.clone());

    for ex in externs {
        builder = builder.externs(*ex);
    }

    builder.done().run(bin_path.to_str().unwrap())?;
    logger.done_compiling();
    Ok(())
}

fn test_compile(
    logger: &mut Logger,
    manifest: &Manifest,
    bin_path: &Path,
    out_dir: &Path,
    externs: &[&str],
) -> Result<()> {
    logger.compiling_bin(&manifest.crate_name);
    let mut builder = Rustc::builder()
        .edition(manifest.edition)
        .crate_type(CrateType::Bin)
        .crate_name(format!(
            "test_{}_{}",
            &manifest.crate_name,
            bin_path.file_stem().unwrap().to_str().unwrap()
        ))
        .out_dir(out_dir.clone())
        .lib_dir(out_dir.clone())
        .test(true);

    for ex in externs {
        builder = builder.externs(*ex);
    }

    builder.done().run(bin_path.to_str().unwrap())?;
    logger.done_compiling();
    Ok(())
}

pub fn build() -> Result<()> {
    let mut logger = Logger::new();
    let root_dir = root_dir()?;
    let manifest = Manifest::parse_from_file(root_dir.join("Freight.toml"))?;

    let lib_rs = root_dir.join("src").join("lib.rs");
    let main_rs = root_dir.join("src").join("main.rs");
    let target = root_dir.join("target");
    let target_debug = target.join("debug");
    fs::create_dir_all(&target_debug)?;

    match (lib_rs.exists(), main_rs.exists()) {
        (true, true) => {
            lib_compile(&mut logger, &manifest, &lib_rs, &target_debug)?;
            bin_compile(
                &mut logger,
                &manifest,
                &main_rs,
                &target_debug,
                &[&manifest.crate_name],
            )?;
        }
        (true, false) => {
            lib_compile(&mut logger, &manifest, &lib_rs, &target_debug)?;
        }
        (false, true) => {
            bin_compile(&mut logger, &manifest, &main_rs, &target_debug, &[])?;
        }
        (false, false) => return Err("There is nothing to compile".into()),
    }

    Ok(())
}

pub fn build_tests() -> Result<()> {
    let mut logger = Logger::new();
    let root_dir = root_dir()?;
    let manifest = Manifest::parse_from_file(root_dir.join("Freight.toml"))?;

    let lib_rs = root_dir.join("src").join("lib.rs");
    let main_rs = root_dir.join("src").join("main.rs");
    let target = root_dir.join("target");
    let target_tests = target.join("debug").join("tests");
    fs::create_dir_all(&target_tests)?;

    match (lib_rs.exists(), main_rs.exists()) {
        (true, true) => {
            test_compile(&mut logger, &manifest, &lib_rs, &target_tests, &[])?;
            lib_compile(&mut logger, &manifest, &lib_rs, &target_tests)?;
            test_compile(
                &mut logger,
                &manifest,
                &main_rs,
                &target_tests,
                &[&manifest.crate_name],
            )?;
        }
        (true, false) => {
            test_compile(&mut logger, &manifest, &lib_rs, &target_tests, &[])?;
        }
        (false, true) => {
            test_compile(&mut logger, &manifest, &main_rs, &target_tests, &[])?;
        }
        (false, false) => return Err("There is nothing to compile".into()),
    }

    Ok(())
}

pub fn run_tests(test_args: Vec<String>) -> Result<()> {
    for item in root_dir()?
        .join("target")
        .join("debug")
        .join("tests")
        .read_dir()?
    {
        let item = item?;
        let path = item.path();
        let is_test = path.extension().is_none();
        if is_test {
            Command::new(path).args(&test_args).spawn()?.wait()?;
        }
    }
    Ok(())
}

fn root_dir() -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    for ancestor in current_dir.ancestors() {
        if ancestor.join("Freight.toml").exists() {
            return Ok(ancestor.into());
        }
    }
    Err("No root dir".into())
}

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
