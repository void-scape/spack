use crate::process;
use std::collections::HashMap;
use tint::{Color, LinearRgb, Srgb};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ImageMemory {
    pub raw: Vec<Image<Srgb>>,
    pub processed: HashMap<usize, ProcessedImage>,
    pub selected_image: usize,
}

impl Default for ImageMemory {
    fn default() -> Self {
        let raw: Vec<_> = std::fs::read_dir("data")
            .unwrap()
            .take(5)
            .map(|entry| {
                let path = entry.unwrap().path();
                let path_str = path.to_str().unwrap();
                Image::from_path(path_str)
            })
            .collect();
        if raw.is_empty() {
            panic!("no images in data directory");
        }
        let processed = raw.iter().map(process_image).enumerate().collect();

        Self {
            raw,
            processed,
            selected_image: 0,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProcessedImage {
    pub raw: Image<Srgb>,
    pub log: Image<Srgb>,
    pub dilate: Image<Srgb>,
    pub local_max: Image<Srgb>,
    pub local_max_points: Vec<(f32, f32, f32)>,
}

pub fn process_image(image: &Image<Srgb>) -> ProcessedImage {
    fn f32_to_srgb(image: &Image<f32>) -> Image<Srgb> {
        assert_eq!(image.pixels.len(), image.width * image.height);
        Image {
            width: image.width,
            height: image.height,
            pixels: image
                .pixels
                .iter()
                .map(|l| LinearRgb::from_rgb(*l, *l, *l).to_srgb())
                .collect(),
        }
    }

    let sigma = 2f32;
    let dilate_size = (3.0 * sigma).ceil() as usize;
    let luminance_percentile = 0.9999;

    let raw = image.clone();
    let log_f32: Image<f32> = process::laplacian_of_gaussian(&raw, sigma);
    let dilate_f32: Image<f32> = process::dilate(&log_f32, dilate_size);
    let local_max_points = process::peak_local_max(&log_f32, &dilate_f32, luminance_percentile);

    let log = f32_to_srgb(&log_f32);
    let dilate = f32_to_srgb(&dilate_f32);
    let mut local_max = log.clone();
    local_max.pixels.fill(Srgb::from_rgb(0, 0, 0));
    for (x, y, l) in local_max_points.iter() {
        for dy in 0..5 {
            for dx in 0..5 {
                let y = (*y as usize + dy).min(local_max.height - 1);
                let x = (*x as usize + dx).min(local_max.width - 1);
                let i = y * local_max.width + x;
                local_max.pixels[i] = LinearRgb::from_rgb(*l, *l, *l).to_srgb();
            }
        }
    }

    ProcessedImage {
        raw,
        log,
        dilate,
        local_max,
        local_max_points,
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Image<T> {
    pub pixels: Vec<T>,
    pub width: usize,
    pub height: usize,
}

impl Image<Srgb> {
    pub fn from_path(path: &str) -> Self {
        use image::GenericImageView;
        use image::Pixel;

        let bytes = std::fs::read(path).unwrap();
        let mut image = image::load_from_memory(&bytes).unwrap();
        image.set_color_space(image::metadata::Cicp::SRGB).unwrap();

        let width = image.width() as usize;
        let height = 3200;
        assert!(image.height() as usize > 3200);

        Self {
            pixels: image
                .pixels()
                .take(width * height)
                .map(|p| {
                    let c = p.2.channels();
                    Srgb::new(c[0], c[1], c[2], c[3])
                })
                .collect::<Vec<_>>(),
            width,
            height,
        }
    }
}

pub trait Luminance {
    fn luminance(self) -> f32;
}

impl Luminance for f32 {
    fn luminance(self) -> f32 {
        self
    }
}

impl Luminance for Srgb {
    fn luminance(self) -> f32 {
        let c = self.to_linear();
        (c.r() + c.g() + c.b()) / 3.0
    }
}

pub trait FromLuminance {
    fn from_luminance(luminance: f32) -> Self;
}

impl FromLuminance for f32 {
    fn from_luminance(luminance: f32) -> Self {
        luminance
    }
}

impl FromLuminance for Srgb {
    fn from_luminance(luminance: f32) -> Self {
        LinearRgb::from_rgb(luminance, luminance, luminance).to_srgb()
    }
}
