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
//! # a file that is read and wrote by pages
//!

use page::{Page, PAGE_SIZE, PAGE_PAYLOAD_SIZE};
use error::BCDBError;
use offset::Offset;

use std::io;
use std::io::Read;
use std::cmp::min;

pub trait FileOps {
    /// flush buffered writes
    fn flush(&mut self) -> Result<(), BCDBError>;
    /// length of the storage
    fn len (&self) -> Result<u64, BCDBError>;
    /// truncate storage
    fn truncate(&mut self, new_len: u64) -> Result<(), BCDBError>;
    /// tell OS to flush buffers to disk
    fn sync (&self) -> Result<(), BCDBError>;
    /// shutdown async write
    fn shutdown (&mut self);
}

/// by page accessed storage
pub trait PagedFile: FileOps + Send + Sync {
    /// read a page at offset
    fn read_page (&self, offset: Offset) -> Result<Option<Page>, BCDBError>;
    /// append a page
    fn append_page (&mut self, page: Page) -> Result<(), BCDBError>;
}

pub trait RandomWritePagedFile : PagedFile {
    /// write a page at its position
    fn write_page (&mut self, page: Page) -> Result<u64, BCDBError>;
}

/// iterate through pages of a paged file
pub struct PagedFileIterator<'file> {
    // the current page of the iterator
    pagenumber: u64,
    // the current page
    page: Option<Page>,
    // position on current page
    pos: usize,
    // the iterated file
    file: &'file PagedFile
}

/// page iterator
impl<'file> PagedFileIterator<'file> {
    /// create a new iterator starting at given page
    pub fn new (file: &'file PagedFile, offset: Offset) -> PagedFileIterator {
        PagedFileIterator {pagenumber: offset.page_number(), page: None, pos: offset.in_page_pos(), file}
    }

    /// return position next read would be reading from
    pub fn position (&self) -> Offset {
        Offset::from(self.pagenumber * PAGE_SIZE as u64 + self.pos as u64)
    }
}

impl<'file> Iterator for PagedFileIterator<'file> {
    type Item = Page;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pagenumber <= (1 << 35) / PAGE_SIZE as u64 {
            let offset = Offset::from((self.pagenumber)* PAGE_SIZE as u64);
            if let Ok(Some(page)) = self.file.read_page(offset) {
                self.pagenumber += 1;
                return Some(page);
            }
        }
        None
    }
}

impl<'file> Read for PagedFileIterator<'file> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let mut read = 0;
        loop {
            if self.page.is_none() {
                self.page = self.file.read_page(Offset::from(self.pagenumber))?;
            }

            if let Some(ref page) = self.page {
                let have = min(PAGE_PAYLOAD_SIZE - self.pos, buf.len() - read);
                page.read(self.pos, &mut buf[read .. read + have]);
                self.pos += have;
                read += have;
            } else {
                return Ok(read)
            }

            if read == buf.len() {
                break;
            } else {
                self.page = None;
                self.pagenumber += 1;
                self.pos = 0;
            }
        }
        Ok(buf.len())
    }
}
