use super::Edition;
use super::Result;
use std::error::Error;
use std::fs;
use std::path::Path;

pub struct Manifest {
    pub crate_name: String,
    pub edition: Edition,
}

impl Manifest {
    pub fn parse_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut crate_name = None;
        let mut edition = None;

        for line in fs::read_to_string(path)?.lines() {
            let mut split = line.split('=');
            let field = split.next().unwrap().trim();
            let value = split.next().unwrap().trim();

            match field {
                "name" => crate_name = Some(value.replace('"', "")),
                "edition" => {
                    edition = Some(match value.replace('"', "").parse()? {
                        2015 => Edition::E2015,
                        2018 => Edition::E2018,
                        2021 => Edition::E2021,
                        edition => return Err(format!("Edition {edition} is unsupported").into()),
                    })
                }
                field => return Err(format!("Field {field} is unsupported").into()),
            }
        }

        Ok(Self {
            crate_name: crate_name.ok_or::<Box<dyn Error>>("name is a required field".into())?,
            edition: edition.ok_or::<Box<dyn Error>>("edition is a required field".into())?,
        })
    }
}
