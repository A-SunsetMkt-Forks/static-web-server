// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module that provides file stream functionality.
//!

use bytes::{BufMut, Bytes, BytesMut};
use futures_util::Stream;
use std::fs::Metadata;
use std::io::Read;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::mem_cache::CACHE_STORE;
use crate::Result;

#[cfg(unix)]
const DEFAULT_READ_BUF_SIZE: usize = 4_096;

#[cfg(not(unix))]
const DEFAULT_READ_BUF_SIZE: usize = 8_192;

#[derive(Debug)]
pub(crate) struct FileStream<T> {
    pub(crate) reader: T,
    pub(crate) buf_size: usize,
    pub(crate) file_path: Option<String>,
}

impl<T: Read + Unpin> Stream for FileStream<T> {
    type Item = Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buf = BytesMut::zeroed(self.buf_size);
        let file_path = self.file_path.to_owned();

        match Pin::into_inner(self).reader.read(&mut buf[..]) {
            Ok(n) => {
                if n == 0 {
                    Poll::Ready(None)
                } else {
                    buf.truncate(n);
                    let buf = buf.freeze();
                    if let Some(s) = file_path {
                        if let Ok(mut cache) = CACHE_STORE.get().unwrap().lock() {
                            if let Some(mem_file) = cache.get_mut(s.as_str()) {
                                mem_file.data.put(buf.clone());
                            }
                        }
                    }
                    Poll::Ready(Some(Ok(buf)))
                }
            }
            Err(err) => Poll::Ready(Some(Err(anyhow::Error::from(err)))),
        }
    }
}

pub(crate) fn optimal_buf_size(metadata: &Metadata) -> usize {
    let block_size = get_block_size(metadata);
    // If file length is smaller than block size,
    // don't waste space reserving a bigger-than-needed buffer.
    std::cmp::min(block_size as u64, metadata.len()) as usize
}

#[cfg(unix)]
fn get_block_size(metadata: &Metadata) -> usize {
    use std::os::unix::fs::MetadataExt;
    // TODO: blksize() returns u64, should handle bad cast...
    // (really, a block size bigger than 4gb?)

    // Use device blocksize unless it's really small.
    std::cmp::max(metadata.blksize() as usize, DEFAULT_READ_BUF_SIZE)
}

#[cfg(not(unix))]
fn get_block_size(_metadata: &Metadata) -> usize {
    DEFAULT_READ_BUF_SIZE
}
