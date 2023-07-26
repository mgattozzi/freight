use std::io;
use std::io::Write;

pub struct Logger {
    out: io::StdoutLock<'static>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            out: io::stdout().lock(),
        }
    }
    pub fn compiling_crate(&mut self, crate_name: &str) {
        let _ = self
            .out
            .write_all(format!("Compiling crate {crate_name}...").as_bytes());
    }
    pub fn compiling_bin(&mut self, crate_name: &str) {
        let _ = self
            .out
            .write_all(format!("Compiling bin {crate_name}...").as_bytes());
    }
    pub fn done_compiling(&mut self) {
        let _ = self.out.write_all(b"Done\n");
    }
}
