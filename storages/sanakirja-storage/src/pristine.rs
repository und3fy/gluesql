// Copyright (c) 2023 und3fy.dev. All rights reserved.
// Created by und3fined <me@und3fy.dev> on 2023 Aug 21.

use std::{path::Path, sync::Arc};

use sanakirja::{Env as sEnv, MutTxn as sMutTxn, Txn as sTxn, *};

use crate::{error::SanakirjaError, types::*};

pub struct Pristine {
    pub env: Arc<sEnv>,
}

impl Pristine {
    pub fn new<P: AsRef<Path>>(name: P) -> Result<Self, SanakirjaError> {
        Self::new_with_size(name, 1 << 20)
    }

    fn new_with_size<P: AsRef<Path>>(name: P, size: u64) -> Result<Self, SanakirjaError> {
        let env = sEnv::new(name, size, 2);
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

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(usize)]
pub enum Root {
    Version,
    Schemas,
}

const VERSION: L64 = L64(1u64.to_le());

impl Pristine {
    pub fn txn_begin(&self) -> Result<Txn, SanakirjaError> {
        let txn = sEnv::txn_begin(self.env.clone())?;
        if L64(txn.root(Root::Version as usize)) != VERSION {
            return Err(SanakirjaError::Version);
        }

        fn begin(txn: sTxn<Arc<sEnv>>) -> Option<Txn> {
            Some(Txn {
                schemas: txn.root_db(Root::Schemas as usize)?,
                counter: 0,
                txn,
            })
        }

        if let Some(txn) = begin(txn) {
            Ok(txn)
        } else {
            Err(SanakirjaError::PristineCorrupted)
        }
    }

    pub fn mut_txn_begin(&self) -> Result<MutTxn<()>, SanakirjaError> {
        let mut txn = sEnv::mut_txn_begin(self.env.clone()).unwrap();
        if let Some(version) = txn.root(Root::Version as usize) {
            if L64(version) != VERSION {
                return Err(SanakirjaError::Version);
            }
        } else {
            txn.set_root(Root::Version as usize, VERSION.0)
        }

        Ok(MutTxn {
            schemas: if let Some(db) = txn.root_db(Root::Schemas as usize) {
                db
            } else {
                btree::create_db_(&mut txn)?
            },
            txn,
            counter: 0,
        })
    }
}

pub type Txn = GenericTxn<sTxn<Arc<sEnv>>>;
pub type MutTxn<T> = GenericTxn<sMutTxn<Arc<sEnv>, T>>;

pub struct GenericTxn<T: LoadPage<Error = ::sanakirja::Error> + RootPage> {
    pub txn: T,

    pub schemas: btree::UDb<SmallStr, SerializedSchema>,

    counter: usize,
}
