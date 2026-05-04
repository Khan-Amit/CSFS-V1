//! Cyclic Seeking File System – Core Library
//!
//! Provides a circular buffer with desire-based retention, domain separation,
//! and quantum slice storage.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;

const SLICE_HEADER_SIZE: usize = 32; // desire(f32) + timestamp(u64) + domain(u32) + len(u32)

/// CSFS instance
pub struct Csfs {
    file: File,
    ring_size: u64,
    head: u64,
    tail: u64,
}

impl Csfs {
    /// Format a file (or block device) as a circular CSFS arena.
    pub fn format(path: &Path, size_mb: u64) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        file.set_len(size_mb * 1024 * 1024)?;
        // Write superblock (head = 0, tail = 0, version = 1)
        file.write_all(&[0u8; 20])?;
        Ok(())
    }

    /// Open an existing CSFS device.
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let mut file = OpenOptions::new().read(true).write(true).open(path)?;
        // Read superblock to get head/tail (simplified: start at 0)
        Ok(Csfs {
            file,
            ring_size: file.metadata()?.len(),
            head: 4096,      // first slice after superblock
            tail: 4096,
        })
    }

    /// Append a new slice to the head.
    pub fn append(&mut self, domain: u32, desire: f32, data: &[u8]) -> std::io::Result<()> {
        let slice_len = SLICE_HEADER_SIZE + data.len();
        let pos = self.head % self.ring_size;
        self.file.seek(SeekFrom::Start(pos))?;
        // Write header: desire (f32), timestamp (u64), domain (u32), data len (u32)
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.file.write_all(&desire.to_le_bytes())?;
        self.file.write_all(&timestamp.to_le_bytes())?;
        self.file.write_all(&domain.to_le_bytes())?;
        self.file.write_all(&(data.len() as u32).to_le_bytes())?;
        self.file.write_all(data)?;
        self.head += slice_len as u64;
        Ok(())
    }

    /// Read the slice at a given logical offset.
    pub fn read(&mut self, offset: u64) -> std::io::Result<Vec<u8>> {
        let pos = (self.tail + offset) % self.ring_size;
        self.file.seek(SeekFrom::Start(pos))?;
        let mut desire_buf = [0u8; 4];
        self.file.read_exact(&mut desire_buf)?;
        let mut ts_buf = [0u8; 8];
        self.file.read_exact(&mut ts_buf)?;
        let mut domain_buf = [0u8; 4];
        self.file.read_exact(&mut domain_buf)?;
        let mut len_buf = [0u8; 4];
        self.file.read_exact(&mut len_buf)?;
        let data_len = u32::from_le_bytes(len_buf) as usize;
        let mut data = vec![0u8; data_len];
        self.file.read_exact(&mut data)?;
        Ok(data)
    }

    /// Advance the tail (delete old data) based on minimum desire threshold.
    pub fn advance_tail(&mut self, min_desire: f32) -> std::io::Result<usize> {
        let mut removed = 0;
        while self.tail < self.head {
            // Peek desire at current tail
            let pos = self.tail % self.ring_size;
            self.file.seek(SeekFrom::Start(pos))?;
            let mut desire_buf = [0u8; 4];
            self.file.read_exact(&mut desire_buf)?;
            let desire = f32::from_le_bytes(desire_buf);
            if desire >= min_desire {
                break;
            }
            // Read slice length
            self.file.seek(SeekFrom::Start(pos + 4 + 8 + 4))?; // skip desire+timestamp+domain
            let mut len_buf = [0u8; 4];
            self.file.read_exact(&mut len_buf)?;
            let slice_len = u32::from_le_bytes(len_buf) as u64 + SLICE_HEADER_SIZE as u64;
            self.tail += slice_len;
            removed += 1;
        }
        Ok(removed)
    }

    /// Get current head and tail positions.
    pub fn position(&self) -> (u64, u64) { (self.head, self.tail) }
}
