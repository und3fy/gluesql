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

    pub fn get<T>(&self, key: impl AsRef<str>) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let value = match self.storage_type {
            WorkersKVStorage::Local => LocalStorage::get(key),
            WorkersKVStorage::Session => SessionStorage::get(key),
        };

        match value {
            Ok(value) => Ok(Some(value)),
            Err(StorageError::KeyNotFound(_)) => Ok(None),
            Err(e) => Err(Error::StorageMsg(e.to_string())),
        }
    }

    pub fn set<T>(&self, key: impl AsRef<str>, value: T) -> Result<()>
    where
        T: Serialize,
    {
        match self.storage_type {
            WorkersKVStorage::Local => LocalStorage::set(key, value),
            WorkersKVStorage::Session => SessionStorage::set(key, value),
        }
        .map_err(|e| Error::StorageMsg(e.to_string()))
    }

    pub fn delete(&self, key: impl AsRef<str>) {
        match self.storage_type {
            WorkersKVStorage::Local => LocalStorage::delete(key),
            WorkersKVStorage::Session => SessionStorage::delete(key),
        }
    }
}

#[async_trait(?Send)]
impl Store for WorkersKVStorage {
    async fn fetch_all_schemas(&self) -> Result<Vec<Schema>> {
        let mut table_names: Vec<String> = self.get(TABLE_NAMES_PATH)?.unwrap_or_default();
        table_names.sort();

        table_names
            .iter()
            .filter_map(|table_name| {
                self.get(format!("{}/{}", SCHEMA_PATH, table_name))
                    .transpose()
            })
            .collect::<Result<Vec<_>>>()
    }

    async fn fetch_schema(&self, table_name: &str) -> Result<Option<Schema>> {
        self.get(format!("{}/{}", SCHEMA_PATH, table_name))
    }

    async fn fetch_data(&self, table_name: &str, target: &Key) -> Result<Option<DataRow>> {
        let path = format!("{}/{}", DATA_PATH, table_name);
        let row = self
            .get::<Vec<(Key, DataRow)>>(path)?
            .unwrap_or_default()
            .into_iter()
            .find_map(|(key, row)| (&key == target).then_some(row));

        Ok(row)
    }

    async fn scan_data(&self, table_name: &str) -> Result<RowIter> {
        let path = format!("{}/{}", DATA_PATH, table_name);
        let mut rows = self.get::<Vec<(Key, DataRow)>>(path)?.unwrap_or_default();

        match self.get(format!("{}/{}", SCHEMA_PATH, table_name))? {
            Some(Schema {
                column_defs: Some(column_defs),
                ..
            }) if column_defs.iter().any(|column_def| {
                matches!(
                    column_def.unique,
                    Some(ColumnUniqueOption { is_primary: true })
                )
            }) =>
            {
                rows.sort_by(|(key_a, _), (key_b, _)| key_a.cmp(key_b));
            }
            _ => {}
        }

        Ok(Box::new(rows.into_iter().map(Ok)))
    }
}

#[async_trait(?Send)]
impl StoreMut for WorkersKVStorage {
    async fn insert_schema(&mut self, schema: &Schema) -> Result<()> {
        let mut table_names: Vec<String> = self.get(TABLE_NAMES_PATH)?.unwrap_or_default();
        table_names.push(schema.table_name.clone());

        self.set(TABLE_NAMES_PATH, table_names)?;
        self.set(format!("{}/{}", SCHEMA_PATH, schema.table_name), schema)
    }

    async fn delete_schema(&mut self, table_name: &str) -> Result<()> {
        let mut table_names: Vec<String> = self.get(TABLE_NAMES_PATH)?.unwrap_or_default();
        table_names
            .iter()
            .position(|name| name == table_name)
            .map(|i| table_names.remove(i));

        self.set(TABLE_NAMES_PATH, table_names)?;
        self.delete(format!("{}/{}", SCHEMA_PATH, table_name));
        self.delete(format!("{}/{}", DATA_PATH, table_name));
        Ok(())
    }

    async fn append_data(&mut self, table_name: &str, new_rows: Vec<DataRow>) -> Result<()> {
        let path = format!("{}/{}", DATA_PATH, table_name);
        let rows = self.get::<Vec<(Key, DataRow)>>(&path)?.unwrap_or_default();
        let new_rows = new_rows.into_iter().map(|row| {
            let key = Key::Uuid(Uuid::new_v4().as_u128());

            (key, row)
        });

        let rows = rows.into_iter().chain(new_rows).collect::<Vec<_>>();

        self.set(path, rows)
    }

    async fn insert_data(&mut self, table_name: &str, new_rows: Vec<(Key, DataRow)>) -> Result<()> {
        let path = format!("{}/{}", DATA_PATH, table_name);
        let mut rows = self.get::<Vec<(Key, DataRow)>>(&path)?.unwrap_or_default();

        for (key, row) in new_rows.into_iter() {
            if let Some(i) = rows.iter().position(|(k, _)| k == &key) {
                rows[i] = (key, row);
            } else {
                rows.push((key, row));
            }
        }

        self.set(path, rows)
    }

    async fn delete_data(&mut self, table_name: &str, keys: Vec<Key>) -> Result<()> {
        let path = format!("{}/{}", DATA_PATH, table_name);
        let mut rows = self.get::<Vec<(Key, DataRow)>>(&path)?.unwrap_or_default();

        for key in keys.iter() {
            if let Some(i) = rows.iter().position(|(k, _)| k == key) {
                rows.remove(i);
            }
        }

        self.set(path, rows)
    }
}

impl gluesql_core::store::AlterTable for WorkersKVStorage {}
impl gluesql_core::store::Index for WorkersKVStorage {}
impl gluesql_core::store::IndexMut for WorkersKVStorage {}
impl gluesql_core::store::Transaction for WorkersKVStorage {}
impl Metadata for WorkersKVStorage {}
impl gluesql_core::store::CustomFunction for WorkersKVStorage {}
impl gluesql_core::store::CustomFunctionMut for WorkersKVStorage {}
