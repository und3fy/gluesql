// Copyright (c) 2023 und3fy.dev. All rights reserved.
// Created by und3fined <me@und3fy.dev> on 2023 Aug 21.

mod error;
mod pristine;
mod small_string;
mod store;
mod types;

use {
    error::err_into,
    gluesql_core::error::{Error, Result},
    std::{
        fs::{self, File},
        path::PathBuf,
    },
};

use gluesql_core::data::Schema;
use pristine::Pristine;

pub struct SanakirjaStorage {
    pub pristine: Pristine,
}

impl SanakirjaStorage {
    pub fn new(path: PathBuf) -> Result<Self> {
        fs::create_dir_all(&path).unwrap();
        let path = PathBuf::from(path);
        let db_path = path.join("pristine");

        let pristine = Pristine::new(db_path).map_err(err_into)?;

        Ok(Self { pristine })
    }
}
