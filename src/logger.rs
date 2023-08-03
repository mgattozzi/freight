use crate::Result;
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
    pub fn compiling_crate(&mut self, crate_name: &str) -> Result<()> {
        self.out
            .write_all(format!("   Compiling lib {crate_name}\n").as_bytes())?;
        self.out.flush()?;
        Ok(())
    }
    pub fn compiling_bin(&mut self, crate_name: &str) -> Result<()> {
        self.out
            .write_all(format!("   Compiling bin {crate_name}\n").as_bytes())?;
        self.out.flush()?;
        Ok(())
    }
    pub fn done_compiling(&mut self) -> Result<()> {
        self.out.write_all(b"    Finished dev\n")?;
        self.out.flush()?;
        Ok(())
    }
    pub fn main_unit_test(&mut self) -> Result<()> {
        self.unit_test("src/main.rs")?;
        Ok(())
    }
    pub fn lib_unit_test(&mut self) -> Result<()> {
        self.unit_test("src/lib.rs")?;
        Ok(())
    }
    fn unit_test(&mut self, file: &str) -> Result<()> {
        self.out.write_all(b"     Running unittests ")?;
        self.out.write_all(file.as_bytes())?;
        self.out.write_all(b"\n")?;
        self.out.flush()?;
        Ok(())
    }
    pub fn doc_test(&mut self, crate_name: &str) -> Result<()> {
        self.out.write_all(b"   Doc-tests ")?;
        self.out.write_all(crate_name.as_bytes())?;
        self.out.write_all(b"\n")?;
        self.out.flush()?;
        Ok(())
    }
}
