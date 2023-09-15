use std::env;
use std::error::Error;
use std::process;

fn main() -> Result<(), Box<dyn Error>> {
    const HELP: &str = include_str!("help.txt");

    let mut args = env::args().skip(1);
    match args.next().as_ref().map(String::as_str) {
        Some("init") => freight::init(&env::current_dir()?)?,
        Some("run") => freight::run(args.collect::<Vec<String>>())?,
        Some("build") => freight::build()?,
        Some("doc") => {
            let open = args
                .next()
                .as_ref()
                .map(String::as_str)
                .map(|open| open == "--open")
                .unwrap_or(false);
            freight::doc(open)?
        }
        Some("test") => {
            freight::build_tests()?;
            loop {
                match args.next().as_ref().map(String::as_str) {
                    Some("--") | None => break,
                    _ => continue,
                }
            }
            freight::run_tests(args.collect::<Vec<String>>())?
        }
        Some("help") => println!("{HELP}"),
        _ => {
            println!("Unsupported command");
            println!("{HELP}");

            process::exit(1);
        }
    }

    Ok(())
}
