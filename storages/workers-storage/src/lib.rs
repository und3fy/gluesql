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
    pub fn new(env: worker::Env, bucket: &str) -> Result<Self> {
        let bucket = env.bucket(bucket).unwrap();
        Ok(Self { bucket })
    }

    fn fetch_schema(&self, table_name: &str) -> Result<Option<Schema>> {
        let key = format!("gluesql/schema/{}", table_name);
        let result = self.bucket.get(key).execute();
        match result {
            Ok(value) => {
                let schema = value.unwrap();

                // let schema = value
                //     .map(|v| bincode::deserialize(&v))
                //     .transpose()
                //     .map_err(err_into)
                //     .map_err(ConflictableTransactionError::Abort)?;
                // Ok(schema)
            }
            Err(_) => Err(Error::StorageMsg(
                WorkersStorageError::KeyNotFound.to_string(),
            )),
        }

        // let key = format!("schema/{}", table_name);
        // let value = tree.get(key.as_bytes())?;
        // let schema_snapshot = value
        //     .map(|v| bincode::deserialize(&v))
        //     .transpose()
        //     .map_err(err_into)
        //     .map_err(ConflictableTransactionError::Abort)?;

        // Ok((key, schema_snapshot))
    }
}
