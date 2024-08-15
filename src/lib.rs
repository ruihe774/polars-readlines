use std::fs::File;
use std::io;
use std::path::Path;
use std::ptr::NonNull;

use memchr::memchr;
use memmap2::Mmap;
use polars::prelude::*;
use polars_arrow::array::{BinaryViewArrayGeneric, View};
use polars_arrow::buffer::Buffer;
use polars_arrow::types::NativeType;
use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;
use pyo3_polars::PolarsAllocator;

#[global_allocator]
static ALLOC: PolarsAllocator = PolarsAllocator::new();

fn read_lines(path: impl AsRef<Path>) -> io::Result<DataFrame> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file) }?;
    let _ = file;
    let mut views = Vec::new();
    let mut buffers = Vec::new();
    let mut start_offset = 0;
    let mut last = 0;
    let mut total_len = 0;
    loop {
        let Some(slice) = mmap.get(last..).filter(|slice| !slice.is_empty()) else {
            let last_slice = &mmap[start_offset..];
            if !last_slice.is_empty() {
                buffers.push((last_slice.as_ptr(), last_slice.len()));
            }
            break;
        };
        let orig_len = memchr(b'\n', slice).unwrap_or(slice.len());
        let line = &slice[..orig_len];
        let len = orig_len
            .checked_sub(1)
            .filter(|&pos| line[pos] == b'\r')
            .unwrap_or(orig_len);
        let line = &line[..len];
        total_len += len;
        assert!(len < u32::MAX as usize, "line too long");
        let start = last - start_offset;
        if start + len > u32::MAX as usize {
            let last_slice = &mmap[start_offset..last];
            buffers.push((last_slice.as_ptr(), last_slice.len()));
            start_offset = last;
        }
        let start = last - start_offset;
        let mut payload = [0; 16];
        payload[0..4].copy_from_slice(&(len as u32).to_le_bytes());
        if len <= 12 {
            payload[4..4 + len].copy_from_slice(line);
        } else {
            payload[4..8].copy_from_slice(&line[0..4]);
            payload[8..12].copy_from_slice(&(buffers.len() as u32).to_le_bytes());
            payload[12..16].copy_from_slice(&(start as u32).to_le_bytes());
        }
        views.push(View::from_le_bytes(payload));
        last = last + orig_len + 1;
    }
    let mmap = Arc::new(mmap);
    let buffers: Vec<_> = buffers
        .into_iter()
        .map(|(ptr, len)| unsafe {
            arrow_buffer::Buffer::from_custom_allocation(
                NonNull::new_unchecked(ptr as *mut u8),
                len,
                mmap.clone(),
            )
        })
        .map(Buffer::<u8>::from)
        .collect();
    let array = unsafe {
        BinaryViewArrayGeneric::<str>::new_unchecked(
            ArrowDataType::Utf8View,
            views.into(),
            buffers.into(),
            None,
            total_len,
            mmap.len(),
        )
    };
    let array = StringChunked::with_chunk("lines", array);
    let frame = DataFrame::new(vec![array]).unwrap();
    Ok(frame)
}

#[pymodule]
#[pyo3(name = "_polars_readlines")]
fn polars_readlines(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    #[pyfn(m, name = "read_lines")]
    fn py_read_lines(py: Python, path: String) -> PyResult<PyDataFrame> {
        let frame = py.allow_threads(move || read_lines(path))?;
        Ok(PyDataFrame(frame))
    }
    Ok(())
}
