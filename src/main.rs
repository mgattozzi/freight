use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(not(stage1))]
    {
        use freight::CrateType;
        use freight::Edition;
        use freight::Rustc;
        use std::fs;
        use std::path::PathBuf;
        const BOOTSTRAP_STAGE1: &str = "bootstrap_stage1";

        let target_dir = PathBuf::from("target");
        let bootstrap_dir = target_dir.join(BOOTSTRAP_STAGE1);
        fs::create_dir_all(&bootstrap_dir)?;
        Rustc::builder()
            .edition(Edition::E2021)
            .crate_type(CrateType::Lib)
            .crate_name("freight")
            .out_dir(bootstrap_dir.clone())
            .lib_dir(bootstrap_dir.clone())
            .cfg("stage1")
            .done()
            .run("src/lib.rs")?;
        Rustc::builder()
            .edition(Edition::E2021)
            .crate_type(CrateType::Bin)
            .crate_name("freight_stage1")
            .out_dir(bootstrap_dir.clone())
            .lib_dir(bootstrap_dir)
            .cfg("stage1")
            .externs("freight")
            .done()
            .run("src/main.rs")?;

        println!("Completed Stage1 Build");
    }
    #[cfg(stage1)]
    {
        use std::env;
        use std::process;

        const HELP: &str = "\
          Alternative for Cargo\n\n\
          Usage: freight [COMMAND] [OPTIONS]\n\n\
          Commands:\n    \
              help    Print out this message
        ";

        let mut args = env::args().skip(1);
        match args.next().as_ref().map(String::as_str) {
            Some("help") => println!("{HELP}"),
            _ => {
                println!("Unsupported command");
                println!("{HELP}");

                process::exit(1);
            }
        }
    }

    Ok(())
}
