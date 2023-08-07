// Copyright (c) 2023 und3fy.dev. All rights reserved.
// Created by und3fined <me@und3fy.dev> on 2023 Aug 03.
#![cfg(target_arch = "wasm32")]
#![deny(clippy::str_to_string)]

use worker;

use {
    async_trait::async_trait,
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

/// gluesql-schema-names -> {Vec<String>}
const TABLE_NAMES_PATH: &str = "gluesql-schema-names";

/// gluesql-schema/{schema_name} -> {Schema}
const SCHEMA_PATH: &str = "gluesql-schema";

/// gluesql-data/{table_name} -> {Vec<DataRow>}
const DATA_PATH: &str = "gluesql-data";

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct WorkersKVStorage {
    env: &worker::Env,
}

impl WorkersKVStorage {
    pub fn new(env: worker::Env) -> Self {
        Self { env }
    }
}
