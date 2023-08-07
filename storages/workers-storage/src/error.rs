// Copyright (c) 2023 und3fy.dev. All rights reserved.
// Created by und3fined <me@und3fy.dev> on 2023 Aug 07.

use {gluesql_core::error::Error, thiserror::Error};

pub trait ResultExt<T, E: ToString> {
    fn map_storage_err(self) -> Result<T, Error>;
}

impl<T, E: ToString> ResultExt<T, E> for std::result::Result<T, E> {
    fn map_storage_err(self) -> Result<T, Error> {
        self.map_err(|e| e.to_string()).map_err(Error::StorageMsg)
    }
}

pub trait OptionExt<T, E: ToString> {
    fn map_storage_err(self, error: E) -> Result<T, Error>;
}

impl<T, E: ToString> OptionExt<T, E> for std::option::Option<T> {
    fn map_storage_err(self, error: E) -> Result<T, Error> {
        self.ok_or_else(|| error.to_string())
            .map_err(Error::StorageMsg)
    }
}

#[derive(Error, Debug)]
pub enum WorkersStorageError {
    #[error("bucket not found")]
    BucketNotFound,

    #[error("key not found")]
    KeyNotFound,

    #[error("table does not exist")]
    TableDoesNotExist,

    #[error("column does not exist: {0}")]
    ColumnDoesNotExist(String),

    #[error("table name does not match with file")]
    TableNameDoesNotMatchWithFile,

    #[error("invalid log file content: {0}")]
    InvalidJsonContent(String),
}
