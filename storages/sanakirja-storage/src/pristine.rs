// Copyright (c) 2023 und3fy.dev. All rights reserved.
// Created by und3fined <me@und3fy.dev> on 2023 Aug 21.

use std::{path::Path, sync::Arc};

use sanakirja::*;

use crate::error::SanakirjaError;

pub struct Pristine {
    pub env: Arc<Env>,
}

pub type P<K, V> = btree::page::Page<K, V>;
pub type Db<K, V> = btree::Db<K, V>;
pub type UP<K, V> = btree::page_unsized::Page<K, V>;
pub type UDb<K, V> = btree::Db_<K, V, UP<K, V>>;

impl Pristine {
    pub fn new<P: AsRef<Path>>(name: P) -> Result<Self, SanakirjaError> {
        Self::new_with_size(name, 1 << 20)
    }

    fn new_with_size<P: AsRef<Path>>(name: P, size: u64) -> Result<Self, SanakirjaError> {
        let env = Env::new(name, size, 2);
        match env {
            Ok(env) => Ok(Pristine { env: Arc::new(env) }),
            Err(Error::IO(e)) => {
                if let std::io::ErrorKind::WouldBlock = e.kind() {
                    Err(SanakirjaError::PristineLocked)
                } else {
                    Err(SanakirjaError::Sanakirja(Error::IO(e)))
                }
            }
            Err(e) => Err(SanakirjaError::Sanakirja(e)),
        }
    }
}

pub type Txt = GenericTxn<Txn<Arc<Env>>>;
pub type MutTxn<T> = GenericTxn<::sanakirja::MutTxn<Arc<Env>, T>>;

pub struct GenericTxn<T: LoadPage<Error = ::sanakirja::Error> + RootPage> {
    pub txn: T,
}
