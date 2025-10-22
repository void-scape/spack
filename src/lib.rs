// TODO: Hand rolled png decoder: https://github.com/madler/zlib/blob/master/contrib/puff/puff.c

use tint::Srgb;

pub const WIDTH: usize = 600;
pub const HEIGHT: usize = 600;

pub struct Memory {
    images: Vec<Image>,
    selected_image: usize,
}

impl Default for Memory {
    fn default() -> Self {
        let images: Vec<_> = std::fs::read_dir("data")
            .unwrap()
            .map(|entry| Image::from_path(entry.unwrap().path().to_str().unwrap()))
            .collect();
        if images.is_empty() {
            panic!("no images in data directory");
        }

        Self {
            images,
            selected_image: 0,
        }
    }
}

struct Image {
    pixels: Vec<Srgb>,
    width: usize,
    height: usize,
}

impl Image {
    pub fn from_path(path: &str) -> Self {
        use image::GenericImageView;
        use image::Pixel;

        let bytes = std::fs::read(path).unwrap();
        let mut image = image::load_from_memory(&bytes).unwrap();
        image.set_color_space(image::metadata::Cicp::SRGB).unwrap();

        Self {
            pixels: image
                .pixels()
                .map(|p| {
                    let c = p.2.channels();
                    Srgb::new(c[0], c[1], c[2], c[3])
                })
                .collect::<Vec<_>>(),
            width: image.width() as usize,
            height: image.height() as usize,
        }
    }
}

pub fn handle_input(glazer::PlatformInput { memory, input }: glazer::PlatformInput<Memory>) {
    if let glazer::Input::Key {
        code,
        pressed: true,
        ..
    } = input
    {
        match code {
            glazer::KeyCode::LeftArrow => {
                memory.selected_image = (memory.selected_image + 1) % memory.images.len();
            }
            glazer::KeyCode::RightArrow => {
                assert!(!memory.images.is_empty());
                if memory.selected_image == 0 {
                    memory.selected_image = memory.images.len() - 1;
                } else {
                    memory.selected_image -= 1;
                }
            }
            _ => {}
        }
    }
}

pub fn update_and_render(
    glazer::PlatformUpdate {
        memory,
        frame_buffer,
        ..
    }: glazer::PlatformUpdate<Memory, Srgb>,
) {
    render(memory, frame_buffer);
}

fn render(memory: &Memory, frame_buffer: &mut [Srgb]) {
    frame_buffer.fill(Srgb::from_rgb(80, 80, 80));

    let selected_image = &memory.images[memory.selected_image];

    let image_width = selected_image.width as f32;
    let image_height = selected_image.height as f32;
    let width = WIDTH as f32;
    let height = HEIGHT as f32;

    let mut xmax = image_width;
    let mut ymax = image_height;

    let image_scale = (width / image_width).min(height / image_height);

    xmax *= image_scale;
    ymax *= image_scale;

    let (xmin, ymin) = if width / image_width > height / image_height {
        let xoffset = (width - xmax) / 2.0;
        xmax += xoffset;
        (xoffset, 0.0)
    } else {
        let yoffset = (height - ymax) / 2.0;
        ymax += yoffset;
        (0.0, yoffset)
    };

    rast::rast_quad(
        frame_buffer,
        WIDTH,
        HEIGHT,
        xmin as i32,
        ymin as i32,
        xmax as i32,
        ymin as i32,
        xmax as i32,
        ymax as i32,
        xmin as i32,
        ymax as i32,
        (0.0, 0.0),
        (1.0, 0.0),
        (1.0, 1.0),
        (0.0, 1.0),
        rast::TextureShader {
            texture: &selected_image.pixels,
            width: selected_image.width,
            height: selected_image.height,
            sampler: rast::Sampler::Bilinear,
        },
    );
}
