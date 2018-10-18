//
// Copyright 2018 Tamas Blummer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
//!
//! # a page in the blockchain store
//!
//! The page is the unit of read and write.
//!
//!

use offset::Offset;
use byteorder::{ByteOrder, BigEndian};

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_PAYLOAD_SIZE: usize = 4090;

/// A page of the persistent files
#[derive(Clone)]
pub struct Page {
    content: [u8; PAGE_SIZE],
}

impl Page {
    /// create an empty page
    pub fn new () -> Page {
        Page{ content: [0u8; PAGE_SIZE] }
    }

    /// create a Page from read buffer
    pub fn from_buf (content: [u8; PAGE_SIZE]) -> Page {
        Page{ content }
    }

    /// interpret the last 6 bytes as an offset
    pub fn offset (&self) -> Offset {
        self.read_offset(PAGE_PAYLOAD_SIZE)
    }

    /// write slice at a position
    pub fn write(&mut self, pos: usize, slice: &[u8]) {
        self.content[pos .. pos + slice.len()].copy_from_slice(slice)
    }

    /// read at position
    pub fn read (&self, pos: usize, buf: &mut [u8]) {
        let len = buf.len();
        buf.copy_from_slice(&self.content[pos .. pos+len])
    }

    /// write an offset into the page
    pub fn write_offset (&mut self, pos: usize, offset: Offset) {
        let mut buf = [0u8; 6];
        BigEndian::write_u48(&mut buf, offset.as_u64());
        self.content[pos..pos+6].copy_from_slice(&buf[..]);
    }

    /// read an offset at a page position
    pub fn read_offset(&self, pos: usize) -> Offset {
        Offset::from(BigEndian::read_u64(&self.content[pos..pos+6]))
    }

    /// write an offset into the page
    pub fn write_u64 (&mut self, pos: usize, n: u64) {
        let mut buf = [0u8; 8];
        BigEndian::write_u64(&mut buf, n);
        self.content[pos..pos+8].copy_from_slice(&buf[..]);
    }

    /// read an offset at a page position
    pub fn read_u64(&self, pos: usize) -> u64 {
        BigEndian::read_u64(&self.content[pos..pos+8])
    }

    /// into write buffer
    pub fn into_buf (self) -> [u8; PAGE_SIZE] {
        self.content
    }
}
