use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{anyhow, Result};
use image::imageops::{resize, FilterType};
use image::{DynamicImage, GenericImageView, GrayImage, Luma};

use crate::db::models::CustomImageConfig;

const MAX_IMAGE_WIDTH: u32 = 160;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsciiCharset {
    Dense,
    Simple,
}

impl AsciiCharset {
    pub fn name(self) -> &'static str {
        match self {
            Self::Dense => "dense",
            Self::Simple => "simple",
        }
    }

    fn symbols(self) -> &'static str {
        match self {
            Self::Dense => " .:-=+*#%@",
            Self::Simple => " .#",
        }
    }
}

impl Display for AsciiCharset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for AsciiCharset {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "dense" => Ok(Self::Dense),
            "simple" => Ok(Self::Simple),
            _ => Err(anyhow!("unknown ASCII charset '{value}'")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomImageOptions {
    pub width: u32,
    pub height_scale: f32,
    pub charset: AsciiCharset,
    pub invert: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedAscii {
    pub lines: Vec<String>,
    pub width: u32,
    pub height: u32,
}

impl CustomImageConfig {
    pub fn render_options(&self) -> Result<CustomImageOptions> {
        Ok(CustomImageOptions {
            width: self.width,
            height_scale: self.height_scale,
            charset: self.charset.parse()?,
            invert: self.invert,
        })
    }

    pub fn render_key(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            self.path.display(),
            self.width,
            self.height_scale,
            self.charset,
            self.invert
        )
    }
}

pub fn render_config(config: &CustomImageConfig) -> Result<RenderedAscii> {
    render_path(&config.path, &config.render_options()?)
}

pub fn render_path(path: &Path, options: &CustomImageOptions) -> Result<RenderedAscii> {
    validate_options(options)?;
    let image = load_image(path)?;
    render_ascii(&image, options)
}

fn load_image(path: &Path) -> Result<DynamicImage> {
    if !has_supported_extension(path) {
        return Err(anyhow!(
            "unsupported image format for '{}'; expected png, jpg, or jpeg",
            path.display()
        ));
    }

    image::open(path)
        .map_err(|source| anyhow!("failed to open image '{}': {source}", path.display()))
}

fn has_supported_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg"
            )
        })
        .unwrap_or(false)
}

fn render_ascii(image: &DynamicImage, options: &CustomImageOptions) -> Result<RenderedAscii> {
    validate_options(options)?;

    let (source_width, source_height) = image.dimensions();
    let output_height = output_height(
        source_width,
        source_height,
        options.width,
        options.height_scale,
    );
    let grayscale = to_luma_over_white(image);
    let resized = resize(
        &grayscale,
        options.width,
        output_height,
        FilterType::Triangle,
    );
    let symbols: Vec<char> = options.charset.symbols().chars().collect();
    let mut lines = Vec::with_capacity(output_height as usize);

    for y in 0..output_height {
        let mut line = String::with_capacity(options.width as usize);

        for x in 0..options.width {
            let luminance = resized.get_pixel(x, y)[0];
            line.push(map_luminance(luminance, &symbols, options.invert));
        }

        lines.push(line);
    }

    Ok(RenderedAscii {
        lines,
        width: options.width,
        height: output_height,
    })
}

fn validate_options(options: &CustomImageOptions) -> Result<()> {
    if options.width == 0 {
        return Err(anyhow!("image width must be greater than 0"));
    }

    if options.width > MAX_IMAGE_WIDTH {
        return Err(anyhow!(
            "image width must be less than or equal to {MAX_IMAGE_WIDTH}"
        ));
    }

    if !options.height_scale.is_finite() || options.height_scale <= 0.0 {
        return Err(anyhow!("height-scale must be a positive finite number"));
    }

    Ok(())
}

fn output_height(
    source_width: u32,
    source_height: u32,
    output_width: u32,
    height_scale: f32,
) -> u32 {
    let aspect = source_height as f32 / source_width as f32;
    (aspect * output_width as f32 * height_scale)
        .round()
        .max(1.0) as u32
}

fn map_luminance(luminance: u8, symbols: &[char], invert: bool) -> char {
    let density = if invert { luminance } else { 255 - luminance };
    let max_index = symbols.len() - 1;
    let index = density as usize * max_index / 255;
    symbols[index]
}

fn to_luma_over_white(image: &DynamicImage) -> GrayImage {
    let rgba = image.to_rgba8();
    GrayImage::from_fn(rgba.width(), rgba.height(), |x, y| {
        let [red, green, blue, alpha] = rgba.get_pixel(x, y).0;
        let alpha = u16::from(alpha);
        let red = composite_over_white(red, alpha);
        let green = composite_over_white(green, alpha);
        let blue = composite_over_white(blue, alpha);
        let luma =
            (299 * u32::from(red) + 587 * u32::from(green) + 114 * u32::from(blue) + 500) / 1000;

        Luma([luma as u8])
    })
}

fn composite_over_white(channel: u8, alpha: u16) -> u8 {
    let channel = u16::from(channel);
    ((channel * alpha + 255 * (255 - alpha) + 127) / 255) as u8
}

pub fn normalized_image_path(path: PathBuf) -> PathBuf {
    std::fs::canonicalize(&path).unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, GrayImage, Luma};

    use super::*;

    #[test]
    fn maps_dark_pixels_to_dense_characters_by_default() {
        let symbols: Vec<char> = AsciiCharset::Dense.symbols().chars().collect();

        assert_eq!(map_luminance(0, &symbols, false), '@');
        assert_eq!(map_luminance(255, &symbols, false), ' ');
    }

    #[test]
    fn render_ascii_produces_lines_without_trailing_newline() {
        let image = DynamicImage::ImageLuma8(GrayImage::from_fn(2, 1, |x, _| {
            if x == 0 {
                Luma([0])
            } else {
                Luma([255])
            }
        }));
        let options = CustomImageOptions {
            width: 2,
            height_scale: 1.0,
            charset: AsciiCharset::Dense,
            invert: false,
        };

        let rendered = render_ascii(&image, &options).unwrap();

        assert_eq!(rendered.lines, vec!["@ ".to_string()]);
        assert_eq!(rendered.width, 2);
        assert_eq!(rendered.height, 1);
    }

    #[test]
    fn rejects_zero_width() {
        let options = CustomImageOptions {
            width: 0,
            height_scale: 1.0,
            charset: AsciiCharset::Dense,
            invert: false,
        };

        assert!(validate_options(&options).is_err());
    }
}
