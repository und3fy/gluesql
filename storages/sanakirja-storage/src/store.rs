// Copyright (c) 2023 und3fy.dev. All rights reserved.
// Created by und3fined <me@und3fy.dev> on 2023 Aug 21.

use sanakirja::btree;

use {
    super::SanakirjaStorage,
    async_trait::async_trait,
    gluesql_core::{
        data::{Key, Schema},
        error::{Error, Result},
        store::{DataRow, RowIter, Store},
    },
};

#[async_trait(?Send)]
impl Store for SanakirjaStorage {
    async fn fetch_all_schemas(&self) -> Result<Vec<Schema>> {
        let schemas = Vec::new();
        let txn = self.pristine.txn_begin()?;

        btree::get(&txn.txn, txn.schemas, k, v)

        Ok(schemas)
    }

    async fn fetch_schema(&self, table_name: &str) -> Result<Option<Schema>> {
        Ok(None)
    }

    async fn fetch_data(&self, table_name: &str, key: &Key) -> Result<Option<DataRow>> {
        Ok(None)
    }

    async fn scan_data(&self, table_name: &str) -> Result<RowIter> {
        Ok(Box::new(vec![].into_iter()))
    }
}
