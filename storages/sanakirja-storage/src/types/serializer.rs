// Copyright (c) 2023 und3fy.dev. All rights reserved.
// Created by und3fined <me@und3fy.dev> on 2023 Aug 24.

use super::*;
use sanakirja::*;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
#[repr(C)]
pub struct SerializedSchema {
    name: L64,
    schema: L64,
}

direct_repr!(SerializedSchema);
impl debug::Check for SerializedSchema {}
