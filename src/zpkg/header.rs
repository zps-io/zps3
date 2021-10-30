use std::fs::File;
use anyhow::Error;
use bytes::BufMut;

pub(crate) const MAGIC: &[u8; 5] = b"zpkg!";

#[derive(Copy, Clone)]
pub enum CompType {
    ZSTD = 0
}

#[derive(Copy, Clone)]
pub enum HashMethod {
    SHA3_256 = 0
}

impl From<u8> for CompType {
    fn from(val: u8) -> CompType {
        match val {
            0 => CompType::ZSTD,
            _ => CompType::ZSTD
        }
    }
}

impl From<u8> for HashMethod {
    fn from(val: u8) -> HashMethod {
        match val {
            0 => HashMethod::SHA3_256,
            _ => HashMethod::SHA3_256
        }
    }
}

#[derive(Copy, Clone)]
pub enum Version {
    V1 = 1
}

pub trait Header {
    fn comp_type(&self) -> u8;
    fn hash_method(&self) -> u8;
    fn manifest_len(&self) -> u32;
    fn to_vec(&self) -> Vec<u8>;
    fn version(&self) -> u8;
}

#[derive(Copy, Clone)]
pub struct HeaderV1 {
    version: u8,
    compression: u8,
    hash_method: u8,
    manifest_length: u32
}

impl HeaderV1 {
    pub fn new(compression: CompType, hash_method: HashMethod, manifest_len: u32) -> HeaderV1 {
        let mut header = Self::default();

        header.compression = compression as u8;
        header.hash_method = hash_method as u8;
        header.manifest_length = manifest_len;
        header
    }

    pub fn default() -> HeaderV1 {
        HeaderV1 {
            version: Version::V1 as u8,
            compression: CompType::ZSTD as u8,
            hash_method: HashMethod::SHA3_256 as u8,
            manifest_length: 0
        }
    }
}

impl Header for HeaderV1 {
    fn comp_type(&self) -> u8 {
        self.compression
    }

    fn hash_method(&self) -> u8 {
        self.hash_method
    }

    fn manifest_len(&self) -> u32 {
        self.manifest_length
    }

    fn to_vec(&self) -> Vec<u8> {
        let mut buf = vec![];

        buf.put_u8(self.version);
        buf.put_u8(self.compression);
        buf.put_u8(self.hash_method);
        buf.put_u32_le(self.manifest_length);

        buf
    }

    fn version(&self) -> u8 {
        self.version
    }
}