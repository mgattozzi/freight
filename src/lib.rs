pub mod config;
mod logger;
pub mod rustc;
pub mod rustdoc;

use crate::rustc::CrateType;
use crate::rustc::Edition;
use crate::rustc::Rustc;
use crate::rustdoc::RustDoc;
use config::Manifest;
use logger::Logger;
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

pub type Result<T> = std::result::Result<T, BoxError>;
pub type BoxError = Box<dyn Error>;

fn lib_compile(
    logger: &mut Logger,
    manifest: &Manifest,
    lib_path: &Path,
    out_dir: &Path,
) -> Result<()> {
    logger.compiling_crate(&manifest.crate_name)?;
    if Rustc::builder()
        .edition(manifest.edition)
        .crate_type(CrateType::Lib)
        .crate_name(&manifest.crate_name)
        .out_dir(out_dir)
        .lib_dir(out_dir)
        .done()
        .run(lib_path.to_str().unwrap())?
        .success()
    {
        Ok(())
    } else {
        Err("Compilation failed".into())
    }
}

fn bin_compile(
    logger: &mut Logger,
    manifest: &Manifest,
    bin_path: &Path,
    out_dir: &Path,
    externs: &[&str],
) -> Result<()> {
    logger.compiling_bin(&manifest.crate_name)?;
    let mut builder = Rustc::builder()
        .edition(manifest.edition)
        .crate_type(CrateType::Bin)
        .crate_name(&manifest.crate_name)
        .out_dir(out_dir)
        .lib_dir(out_dir);

    for ex in externs {
        builder = builder.externs(*ex);
    }

    if builder.done().run(bin_path.to_str().unwrap())?.success() {
        Ok(())
    } else {
        Err("Compilation failed".into())
    }
}

fn test_compile(
    manifest: &Manifest,
    bin_path: &Path,
    out_dir: &Path,
    externs: &[&str],
) -> Result<()> {
    let mut builder = Rustc::builder()
        .edition(manifest.edition)
        .crate_type(CrateType::Bin)
        .crate_name(format!(
            "test_{}_{}",
            &manifest.crate_name,
            bin_path.file_stem().unwrap().to_str().unwrap()
        ))
        .out_dir(out_dir)
        .lib_dir(out_dir)
        .test(true);

    for ex in externs {
        builder = builder.externs(*ex);
    }

    if builder.done().run(bin_path.to_str().unwrap())?.success() {
        Ok(())
    } else {
        Err("Compilation failed".into())
    }
}

pub fn init(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    fs::write(path.join(".gitignore"), b"/target")?;
    let src = path.join("src");
    fs::create_dir_all(&src)?;
    fs::write(
        src.join("main.rs"),
        b"fn main() {\n    println!(\"Hello, World!\");\n}\n",
    )?;
    if !Command::new("git")
        .arg("init")
        .arg(path)
        .output()?
        .status
        .success()
    {
        return Err("git failed to initialize a repository".into());
    }
    let crate_name = path.file_name().unwrap().to_str().unwrap();
    let toml = format!("name = \"{crate_name}\"\nedition = \"2021\"\n");
    fs::write(path.join("Freight.toml"), toml.as_bytes())?;

    Ok(())
}

pub fn run(run_args: Vec<String>) -> Result<()> {
    let root_dir = root_dir()?;
    let main_rs = root_dir.join("src").join("main.rs");
    if main_rs.exists() {
        build()?;
        let manifest = Manifest::parse_from_file(root_dir.join("Freight.toml"))?;
        let target = root_dir.join("target");
        let target_debug = target.join("debug");
        let path = target_debug.join(manifest.crate_name);
        Command::new(path).args(run_args).spawn()?.wait()?;

        Ok(())
    } else {
        Err("Cannot call `freight run` if there is no binary to run".into())
    }
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

pub fn doc(_open: bool) -> Result<()> {
    let root = root_dir()?;
    let manifest = Manifest::parse_from_file(root.join("Freight.toml"))?;
    let target = root.join("target");
    let lib_path = target.join("debug");
    let doc_path = target.join("doc");
    if RustDoc::new(
        manifest.edition,
        manifest.crate_name,
        lib_path,
        Some(doc_path),
    )
    // TODO Fix no main.rs
    .doc(root.join("src").join("lib.rs"))?
    .success()
    {
        Ok(())
    } else {
        Err("Failed to document items".into())
    }
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
            test_compile(&manifest, &lib_rs, &target_tests, &[])?;
            lib_compile(&mut logger, &manifest, &lib_rs, &target_tests)?;
            test_compile(&manifest, &main_rs, &target_tests, &[&manifest.crate_name])?;
        }
        (true, false) => {
            test_compile(&manifest, &lib_rs, &target_tests, &[])?;
        }
        (false, true) => {
            test_compile(&manifest, &main_rs, &target_tests, &[])?;
        }
        (false, false) => return Err("There is nothing to compile".into()),
    }

    if let Ok(items) = root_dir.join("tests").read_dir() {
        for item in items {
            let item = item?;
            let is_file = item.file_type()?.is_file();
            let path = item.path();
            if is_file && path.extension().map(|ext| ext == "rs").unwrap_or(false) {
                test_compile(&manifest, &path, &target_tests, &[&manifest.crate_name])?;
            }
        }
    }

    logger.done_compiling()?;
    Ok(())
}

pub fn run_tests(test_args: Vec<String>) -> Result<()> {
    let mut logger = Logger::new();
    let root = root_dir()?;
    let manifest = Manifest::parse_from_file(root.join("Freight.toml"))?;
    let tests_dir = root.join("target").join("debug").join("tests");

    // Just run the unit tests first
    for item in tests_dir.read_dir()? {
        let item = item?;
        let path = item.path();
        let is_test = path.extension().is_none();
        if is_test {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if file_name == "test_freight_main" {
                logger.main_unit_test()?;
            } else if file_name == "test_freight_lib" {
                logger.lib_unit_test()?;
            } else {
                continue;
            }
            Command::new(path).args(&test_args).spawn()?.wait()?;
        }
    }

    // Then run the tests folder
    for item in tests_dir.read_dir()? {
        let item = item?;
        let path = item.path();
        let is_test = path.extension().is_none();
        if is_test {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if file_name == "test_freight_main" {
                continue;
            } else if file_name == "test_freight_lib" {
                continue;
            } else {
                logger.tests(&file_name.split('_').last().unwrap())?;
            }
            Command::new(path).args(&test_args).spawn()?.wait()?;
        }
    }

    let lib = root.join("src").join("lib.rs");
    // TODO Fix no main.rs doc tests
    if lib.exists() {
        logger.doc_test(&manifest.crate_name)?;
        if !RustDoc::new(
            manifest.edition,
            manifest.crate_name,
            root.join("target").join("debug"),
            None::<&str>,
        )
        .test(lib)?
        .success()
        {
            return Err("Failed to run doc tests".into());
        }
    }
    Ok(())
}

pub fn root_dir() -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    for ancestor in current_dir.ancestors() {
        if ancestor.join("Freight.toml").exists() {
            return Ok(ancestor.into());
        }
    }
    Err("No root dir".into())
}
