use freight::config::Manifest;
use freight::init;
use freight::root_dir;
use freight::rustc::Edition;
use freight::Result;
use std::fs;

#[test]
fn initialize_project() -> Result<()> {
    let root = root_dir()?.canonicalize()?;
    let target = root.join("target");
    let test_init_dir = target.join("test_init_dir");
    fs::create_dir_all(&test_init_dir)?;
    let result = || -> Result<()> {
        init(&test_init_dir)?;
        let git = test_init_dir.join(".git");
        let gitignore = test_init_dir.join(".gitignore");
        let freight_toml = test_init_dir.join("Freight.toml");
        let src = test_init_dir.join("src");
        let main = src.join("main.rs");

        if !git.exists() {
            return Err(".git folder does not exist".into());
        }

        if !gitignore.exists() {
            return Err(".gitignore file does not exist".into());
        }

        if !freight_toml.exists() {
            return Err("Freight.toml file does not exist".into());
        }

        if !src.exists() {
            return Err("src folder does not exist".into());
        }

        if !main.exists() {
            return Err("src/main.rs file does not exist".into());
        }

        let ignore = fs::read_to_string(gitignore)?;
        if ignore != "/target" {
            return Err(format!(".gitignore file was incorrect, contents were '{ignore}'").into());
        }

        let freight = Manifest::parse_from_file(freight_toml)?;
        if freight.crate_name != "test_init_dir" {
            return Err(format!(
                "Freight.toml crate name was incorrect. Value was '{}'",
                freight.crate_name
            )
            .into());
        }
        if freight.edition != Edition::E2021 {
            return Err(format!(
                "Freight.toml edition was incorrect. Value was '{:?}'",
                freight.edition
            )
            .into());
        }
        let contents = fs::read_to_string(main)?;
        if contents != "fn main() {\n    println!(\"Hello, World!\");\n}\n" {
            return Err(
                format!("src/main.rs file was incorrect, contents were '{contents}'").into(),
            );
        }
        Ok(())
    }();
    fs::remove_dir_all(&test_init_dir)?;
    result
}
