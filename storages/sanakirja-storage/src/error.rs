// Copyright (c) 2023 und3fy.dev. All rights reserved.
// Created by und3fined <me@und3fy.dev> on 2023 Aug 21.
use {
    gluesql_core::error::{AlterTableError, Error, IndexError},
    std::{str, time},
    thiserror::Error as ThisError,
};

#[derive(Debug, ThisError)]
pub enum SanakirjaError {
    #[error(transparent)]
    Sanakirja(#[from] sanakirja::Error),
    #[error(transparent)]
    Borrow(#[from] std::cell::BorrowError),

    #[error("Pristine locked")]
    PristineLocked,
    #[error("Pristine corrputed")]
    PristineCorrupted,

    #[error("Pristine version mismatch. Cloning over the network can fix this.")]
    Version,
}

impl From<SanakirjaError> for Error {
    fn from(e: SanakirjaError) -> Error {
        use SanakirjaError::*;

        match e {
            Sanakirja(e) => Error::StorageMsg(e.to_string()),
            Borrow(e) => Error::StorageMsg(e.to_string()),
            PristineLocked => Error::StorageMsg("Pristine locked".to_string()),
            PristineCorrupted => Error::StorageMsg("Pristine corrupted".to_string()),
            Version => Error::StorageMsg(
                "Pristine version mismatch. Cloning over the network can fix this.".to_string(),
            ),
        }
    }
}

pub fn err_into<E>(e: E) -> Error
where
    E: Into<SanakirjaError>,
{
    let e: SanakirjaError = e.into();
    let e: Error = e.into();

    e
}
