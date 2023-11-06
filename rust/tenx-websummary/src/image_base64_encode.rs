#![cfg(feature = "image_base64_encode")]

use std::path::Path;

use anyhow::{bail, Error};

use crate::components::RawImage;

#[derive(Clone, Copy)]
pub enum ImageFormat {
    Jpeg,
    Png,
}

impl ImageFormat {
    pub fn guess(img_path: &Path) -> Result<Self, Error> {
        let ext = match img_path.extension() {
            Some(ext) => ext,
            None => bail!("No extension available in {:?}", img_path),
        };
        Ok(match ext.to_str().unwrap() {
            "png" => ImageFormat::Png,
            "jpg" | "jpeg" => ImageFormat::Jpeg,
            ext => bail!("Unknown image extension {ext} in the path {:?}", img_path),
        })
    }
    pub fn encode(self, img_path: &Path) -> Result<String, Error> {
        Ok(self.encode_bytes(&std::fs::read(img_path)?))
    }
    pub fn encode_bytes(self, file_bytes: &[u8]) -> String {
        let base64_encoded = base64::encode(file_bytes);
        let mime = match self {
            ImageFormat::Jpeg => "jpeg",
            ImageFormat::Png => "png",
        };
        format!("data:image/{mime};base64,{base64_encoded}")
    }
}

impl RawImage {
    pub fn encode_with_format(img_path: &Path, format: ImageFormat) -> Result<Self, Error> {
        Ok(RawImage::new(format.encode(img_path)?))
    }
    pub fn encode(img_path: &Path) -> Result<Self, Error> {
        RawImage::encode_with_format(img_path, ImageFormat::guess(img_path)?)
    }
}
