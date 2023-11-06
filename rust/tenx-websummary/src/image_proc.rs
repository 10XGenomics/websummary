#![cfg(feature = "image_proc")]

use std::path::Path;

use anyhow::Error;
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::DynamicImage;

use crate::components::RawImage;

pub enum ImageResize {
    /// Resize the image such that the new height is less than the number specified
    /// preserving the aspect ratio. If the image height is already less than the
    /// specified height, do nothing.
    ClampHeight(u32),
    /// Resize the image such that the new width is less than the number specified
    /// preserving the aspect ratio. If the image width is already less than the
    /// specified width, do nothing.
    ClampWidth(u32),
    /// Resize the image such that the new height is exactly equal to the number specified
    /// preserving the aspect ratio
    ExactHeight(u32),
    /// Resize the image such that the new width is exactly equal to the number specified
    /// preserving the aspect ratio
    ExactWidth(u32),
}

impl ImageResize {
    pub fn resize_image(
        self,
        img_path: &Path,
        filter_type: FilterType,
    ) -> Result<DynamicImage, Error> {
        Ok(self.resize_dynamic_image(ImageReader::open(img_path)?.decode()?, filter_type))
    }

    pub fn resize_dynamic_image(self, img: DynamicImage, filter_type: FilterType) -> DynamicImage {
        fn scaled_dim(dim_a: u32, new_dim_b: u32, dim_b: u32) -> u32 {
            ((dim_a as f64) * (new_dim_b as f64) / (dim_b as f64)).round() as u32
        }

        let height = img.height();
        let width = img.width();
        let (new_width, new_height) = match self {
            ImageResize::ClampHeight(new_height) => {
                if height <= new_height {
                    return img;
                } else {
                    (scaled_dim(width, new_height, height), new_height)
                }
            }
            ImageResize::ClampWidth(new_width) => {
                if width <= new_width {
                    return img;
                } else {
                    (new_width, scaled_dim(height, new_width, width))
                }
            }
            ImageResize::ExactHeight(new_height) => {
                (scaled_dim(width, new_height, height), new_height)
            }
            ImageResize::ExactWidth(new_width) => (new_width, scaled_dim(height, new_width, width)),
        };
        img.resize(new_width, new_height, filter_type)
    }

    #[cfg(feature = "image_base64_encode")]
    pub fn resize_and_encode_image(
        self,
        img_path: &Path,
        filter_type: FilterType,
    ) -> Result<String, Error> {
        use crate::image_base64_encode::ImageFormat;
        use image::ImageOutputFormat;
        use std::io::Cursor;

        let img = self.resize_image(img_path, filter_type)?;
        let mut buf = Cursor::new(Vec::with_capacity(img.as_bytes().len()));
        img.write_to(&mut buf, ImageOutputFormat::Png)?;
        Ok(ImageFormat::Png.encode_bytes(buf.get_ref()))
    }
}

impl RawImage {
    #[cfg(feature = "image_base64_encode")]
    pub fn resize_and_encode(
        img_path: &Path,
        filter_type: FilterType,
        resize: ImageResize,
    ) -> Result<Self, Error> {
        Ok(RawImage::new(
            resize.resize_and_encode_image(img_path, filter_type)?,
        ))
    }
}
