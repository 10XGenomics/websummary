#![cfg(feature = "image_base64_encode")]

use crate::components::RawImage;
use anyhow::{bail, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use std::path::Path;

#[derive(Clone, Copy)]
pub enum Base64ImageEncoder {
    Jpeg,
    Png,
}

impl Base64ImageEncoder {
    pub fn guess(img_path: &Path) -> Result<Self> {
        let ext = match img_path.extension() {
            Some(ext) => ext,
            None => bail!("No extension available in {:?}", img_path),
        };
        Ok(match ext.to_str().unwrap() {
            "png" => Base64ImageEncoder::Png,
            "jpg" | "jpeg" => Base64ImageEncoder::Jpeg,
            ext => bail!("Unknown image extension {ext} in the path {:?}", img_path),
        })
    }

    pub fn encode(self, img_path: &Path) -> Result<String> {
        Ok(self.encode_bytes(&std::fs::read(img_path)?))
    }

    pub fn encode_bytes(self, file_bytes: &[u8]) -> String {
        let base64_encoded = BASE64_STANDARD.encode(file_bytes);
        let mime = match self {
            Base64ImageEncoder::Jpeg => "jpeg",
            Base64ImageEncoder::Png => "png",
        };
        format!("data:image/{mime};base64,{base64_encoded}")
    }
}

impl RawImage {
    pub fn encode_with_format(img_path: &Path, format: Base64ImageEncoder) -> Result<Self> {
        Ok(RawImage::new(format.encode(img_path)?))
    }

    pub fn encode(img_path: &Path) -> Result<Self> {
        RawImage::encode_with_format(img_path, Base64ImageEncoder::guess(img_path)?)
    }
}
