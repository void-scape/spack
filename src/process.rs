use crate::image::{FromLuminance, Image, Luminance};

pub fn laplacian_of_gaussian<In: Luminance + Copy, Out: FromLuminance + Default + Clone>(
    image: &Image<In>,
    sigma: f32,
) -> Image<Out> {
    // https://homepages.inf.ed.ac.uk/rbf/HIPR2/gsmooth.htm
    let (gaussian_kernel, gaussian_kernel_size) = generate_gaussian_kernel(sigma);
    #[rustfmt::skip]
    let gaussian_kernel = Image { 
        pixels: gaussian_kernel,
        width: gaussian_kernel_size, 
        height: gaussian_kernel_size,
    };
    // https://homepages.inf.ed.ac.uk/rbf/HIPR2/log.htm
    #[rustfmt::skip]
    let laplacian_kernel = Image {
        pixels: vec![
            0.0,  -1.0,  0.0,
            -1.0,  4.0, -1.0,
            0.0,  -1.0,  0.0,
        ],
        width: 3,
        height: 3,
    };
    let gaussian_image: Image<f32> = conv(image, &gaussian_kernel);
    conv(&gaussian_image, &laplacian_kernel)
}

// https://en.wikipedia.org/wiki/Dilation_(morphology)#Flat_structuring_functions
pub fn dilate<In: Luminance + Copy, Out: FromLuminance + Default + Clone>(
    image: &Image<In>,
    size: usize,
) -> Image<Out> {
    assert_eq!(image.pixels.len(), image.width * image.height);

    let mut output = Image {
        pixels: vec![Out::default(); image.pixels.len()],
        width: image.width,
        height: image.height,
    };
    let width = image.width as i32;
    let height = image.height as i32;
    let size2 = size as i32 / 2;
    for oy in 0..image.height {
        for ox in 0..image.width {
            let mut result = f32::MIN;
            for fy in -size2..=size2 {
                for fx in -size2..=size2 {
                    let y = (oy as i32 + fy).max(0).min(height - 1) as usize;
                    let x = (ox as i32 + fx).max(0).min(width - 1) as usize;
                    let sample = image.pixels[y * image.width + x].luminance();
                    result = sample.max(result);
                }
            }
            output.pixels[oy * image.width + ox] = Out::from_luminance(result);
        }
    }
    output
}

// https://scikit-image.org/docs/stable/auto_examples/segmentation/plot_peak_local_max.html
pub fn peak_local_max<In1: Luminance + Copy, In2: Luminance + Copy>(
    image: &Image<In1>,
    max: &Image<In2>,
    percentile: f32,
) -> Vec<(f32, f32, f32)> {
    assert_eq!(image.width, max.width);
    assert_eq!(image.height, max.height);
    assert_eq!(image.pixels.len(), max.pixels.len());

    let min_luminance = compute_adaptive_threshold(image, percentile);
    let mut points = Vec::new();
    let width = image.width;
    for y in 0..image.height {
        for x in 0..image.width {
            let i = y * width + x;
            let in_pixel = image.pixels[i];
            let max_pixel = max.pixels[i];
            if in_pixel.luminance() == max_pixel.luminance()
                && max_pixel.luminance() >= min_luminance
            {
                points.push((x as f32, y as f32, max_pixel.luminance()));
            }
        }
    }
    // sort by descending luminance
    points.sort_by(|a, b| b.2.total_cmp(&a.2));
    points
}

fn compute_adaptive_threshold<In: Luminance + Copy>(image: &Image<In>, percentile: f32) -> f32 {
    let mut values: Vec<f32> = image
        .pixels
        .iter()
        .map(|p| p.luminance())
        .filter(|v| *v > 0.0)
        .collect();
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((values.len() - 1) as f32 * percentile) as usize;
    values[idx]
}

// https://en.wikipedia.org/wiki/Gaussian_blur
fn generate_gaussian_kernel(sigma: f32) -> (Vec<f32>, usize) {
    let kernel_size = (6.0 * sigma).ceil() as usize | 1;
    let kernel_size_2 = kernel_size as i32 / 2;
    let mut kernel = vec![0.0; kernel_size * kernel_size];
    let mult = 1.0 / (std::f32::consts::TAU * sigma * sigma);
    for y in 0..kernel_size as i32 {
        for x in 0..kernel_size as i32 {
            let dy = y - kernel_size_2;
            let dx = x - kernel_size_2;
            let exp = (dx * dx + dy * dy) as f32 / (2.0 * sigma * sigma);
            kernel[y as usize * kernel_size + x as usize] = mult * (-exp).exp();
        }
    }
    let sum: f32 = kernel.iter().sum();
    for v in kernel.iter_mut() {
        *v /= sum;
    }
    (kernel, kernel_size)
}

fn conv<In1: Luminance + Copy, In2: Luminance + Copy, Out: FromLuminance + Default + Clone>(
    i1: &Image<In1>,
    i2: &Image<In2>,
) -> Image<Out> {
    assert_eq!(i1.pixels.len(), i1.width * i1.height);
    assert_eq!(i2.pixels.len(), i2.width * i2.height);

    let out_height = i1.height - i2.height + 1;
    let out_width = i1.width - i2.width + 1;
    let mut output = Image {
        pixels: vec![Out::default(); out_width * out_height],
        width: out_width,
        height: out_height,
    };

    for oy in 0..out_height {
        for ox in 0..out_width {
            let mut result = 0.0;
            for fy in 0..i2.height {
                for fx in 0..i2.width {
                    result += i1.pixels[(oy + fy) * i1.width + ox + fx].luminance()
                        * i2.pixels[fy * i2.width + fx].luminance();
                }
            }
            output.pixels[oy * out_width + ox] = Out::from_luminance(result);
        }
    }

    output
}
