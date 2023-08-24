// Copyright (c) 2023 und3fy.dev. All rights reserved.
// Created by und3fined <me@und3fy.dev> on 2023 Aug 24.

use sanakirja::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct L64(pub u64);

impl From<usize> for L64 {
    fn from(u: usize) -> Self {
        L64((u as u64).to_le())
    }
}

impl From<u64> for L64 {
    fn from(u: u64) -> Self {
        L64(u.to_le())
    }
}

impl From<L64> for u64 {
    fn from(u: L64) -> Self {
        u64::from_le(u.0)
    }
}

impl From<L64> for usize {
    fn from(u: L64) -> Self {
        u64::from_le(u.0) as usize
    }
}

impl L64 {
    pub fn as_u64(&self) -> u64 {
        u64::from_le(self.0)
    }
    pub fn as_usize(&self) -> usize {
        u64::from_le(self.0) as usize
    }
}

impl std::fmt::Display for L64 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(fmt)
    }
}

impl Ord for L64 {
    fn cmp(&self, x: &Self) -> std::cmp::Ordering {
        u64::from_le(self.0).cmp(&u64::from_le(x.0))
    }
}

impl PartialOrd for L64 {
    fn partial_cmp(&self, x: &Self) -> Option<std::cmp::Ordering> {
        Some(u64::from_le(self.0).cmp(&u64::from_le(x.0)))
    }
}

impl std::ops::Add<L64> for L64 {
    type Output = Self;
    fn add(self, x: L64) -> L64 {
        L64((u64::from_le(self.0) + u64::from_le(x.0)).to_le())
    }
}

impl std::ops::Add<usize> for L64 {
    type Output = Self;
    fn add(self, x: usize) -> L64 {
        L64((u64::from_le(self.0) + x as u64).to_le())
    }
}

impl std::ops::SubAssign<usize> for L64 {
    fn sub_assign(&mut self, x: usize) {
        self.0 = ((u64::from_le(self.0)) - x as u64).to_le()
    }
}

impl L64 {
    pub fn from_slice_le(s: &[u8]) -> Self {
        let mut u = 0u64;
        assert!(s.len() >= 8);
        unsafe { std::ptr::copy_nonoverlapping(s.as_ptr(), &mut u as *mut u64 as *mut u8, 8) }
        L64(u)
    }
    pub fn to_slice_le(&self, s: &mut [u8]) {
        assert!(s.len() >= 8);
        unsafe {
            std::ptr::copy_nonoverlapping(&self.0 as *const u64 as *const u8, s.as_mut_ptr(), 8)
        }
    }
}

direct_repr!(L64);
impl debug::Check for L64 {}
