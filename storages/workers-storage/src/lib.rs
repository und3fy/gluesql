// Copyright (c) 2023 und3fy.dev. All rights reserved.
// Created by und3fined <me@und3fy.dev> on 2023 Aug 03.
#![cfg(target_arch = "wasm32")]
#![deny(clippy::str_to_string)]

pub mod error;
pub mod store;

use serde::de::value;
use worker;

use {
    async_trait::async_trait,
    error::{OptionExt, ResultExt, WorkersStorageError},
    gloo_storage::{errors::StorageError, LocalStorage, SessionStorage, Storage},
    gluesql_core::{
        ast::ColumnUniqueOption,
        data::{Key, Schema},
        error::{Error, Result},
        store::{DataRow, Metadata, RowIter, Store, StoreMut},
    },
    serde::{Deserialize, Serialize},
    uuid::Uuid,
};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct WorkersStorage {
    bucket: &worker::Bucket,
}

impl WorkersStorage {
    pub fn new(env: &worker::Env, bucket_name: &str) -> Result<Self> {
        let bucket = env.bucket(bucket_name);
        match bucket {
            Ok(bucket) => Ok(Self { bucket }),
            Err(_) => Err(Error::StorageMsg(
                WorkersStorageError::BucketNotFound.to_string(),
            )),
        }
    }

    fn fetch_schema(&self, table_name: &str) -> Result<Option<Schema>> {
        let key = format!("gluesql/schema/{}", table_name);
        let result = self.bucket.get(&key);
        match result {
            Ok(value) => {}
            Err(_) => Err(Error::StorageMsg(
                WorkersStorageError::KeyNotFound.to_string(),
            )),
        }
    }
}
