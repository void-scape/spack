use crate::{HEIGHT, Memory, View, WIDTH, align, image::Image};
use tint::{Color, Srgb};

pub fn render(frame_buffer: &mut [Srgb], width: usize, height: usize, memory: &Memory) {
    frame_buffer.fill(Srgb::from_rgb(80, 80, 80));

    let key = memory.images.selected_image;
    let selected_image = if let Some(processed) = memory.images.processed.get(&key) {
        match memory.view {
            View::Raw => &processed.raw,
            View::LoG => &processed.log,
            View::Dilate => &processed.dilate,
            View::LocalMax => &processed.local_max,
            View::AlignTriangles => &processed.raw,
        }
    } else {
        &memory.images.raw[key]
    };

    if matches!(memory.view, View::AlignTriangles) {
        let next_index = memory.next_image_index();
        let processed = &memory.images.processed[&memory.images.selected_image];
        let processed2 = &memory.images.processed[&next_index];
        assert_eq!(processed.log.pixels.len(), processed2.log.pixels.len());
        let triangles = crate::align::align(
            processed.log.width,
            processed.log.height,
            &processed.local_max_points,
            &processed2.local_max_points,
            0.001,
        );

        render_image(frame_buffer, width, height, &processed.raw);
        render_image_with_alpha(frame_buffer, width, height, &processed2.raw, 0.5);

        for (t1, t2) in triangles.iter() {
            render_triangle(
                frame_buffer,
                width,
                height,
                &processed.log,
                t1,
                &processed.local_max_points,
                Srgb::from_rgb(255, 0, 0),
            );
            render_triangle(
                frame_buffer,
                width,
                height,
                &processed2.log,
                t2,
                &processed2.local_max_points,
                Srgb::from_rgb(0, 255, 0),
            );
        }
    } else {
        render_image(frame_buffer, width, height, selected_image);
    }

    // if (1.0 - memory.alpha).abs() > f32::EPSILON {
    //     render_image(frame_buffer, width, height, selected_image);
    //     let next_index = memory.next_image_index();
    //     let key = (next_index, memory.view);
    //     let selected_image = &memory
    //         .images
    //         .get(&key)
    //         .unwrap_or_else(|| &memory.images[&raw_key]);
    //     render_image_with_alpha(frame_buffer, width, height, selected_image, memory.alpha);
    // } else {
    //     render_image(frame_buffer, width, height, selected_image);
    // }
}

// (xmin, ymin, xmax, ymax)
fn image_bounding_box(width: usize, height: usize, image: &Image<Srgb>) -> (f32, f32, f32, f32) {
    let image_width = image.width as f32;
    let image_height = image.height as f32;
    let width = width as f32;
    let height = height as f32;

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

    (xmin, ymin, xmax, ymax)
}

fn render_image(frame_buffer: &mut [Srgb], width: usize, height: usize, image: &Image<Srgb>) {
    let (xmin, ymin, xmax, ymax) = image_bounding_box(width, height, image);
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
            texture: &image.pixels,
            width: image.width,
            height: image.height,
            sampler: rast::Sampler::Bilinear,
            blend_mode: rast::BlendMode::None,
        },
    );
}

#[allow(unused)]
fn render_image_with_alpha(
    frame_buffer: &mut [Srgb],
    width: usize,
    height: usize,
    image: &Image<Srgb>,
    alpha: f32,
) {
    let (xmin, ymin, xmax, ymax) = image_bounding_box(width, height, image);
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
        AlphaTextureShader {
            shader: rast::TextureShader {
                texture: &image.pixels,
                width: image.width,
                height: image.height,
                sampler: rast::Sampler::Bilinear,
                blend_mode: rast::BlendMode::None,
            },
            alpha,
        },
    );
}

#[allow(unused)]
#[derive(Clone, Copy)]
struct AlphaTextureShader<'a> {
    shader: rast::TextureShader<'a, Srgb>,
    alpha: f32,
}

impl<'a> rast::Shader for AlphaTextureShader<'a> {
    type VertexData = <rast::TextureShader<'a, Srgb> as rast::Shader>::VertexData;

    fn interpolate(
        &self,
        bcx: f32,
        bcy: f32,
        bcz: f32,
        d1: Self::VertexData,
        d2: Self::VertexData,
        d3: Self::VertexData,
    ) -> Self::VertexData {
        self.shader.interpolate(bcx, bcy, bcz, d1, d2, d3)
    }

    fn blend_mode(&self) -> rast::BlendMode {
        rast::BlendMode::Alpha
    }

    fn fragment(&mut self, data: Self::VertexData) -> tint::LinearRgb {
        let mut color = self.shader.fragment(data);
        let alpha = color.alpha();
        color.set_alpha(alpha * self.alpha);
        color
    }
}

fn render_triangle(
    frame_buffer: &mut [Srgb],
    width: usize,
    height: usize,
    image: &Image<Srgb>,
    triangle: &align::Triangle,
    local_max_points: &[(f32, f32, f32)],
    color: Srgb,
) {
    let (p1x, p1y, _) = local_max_points[triangle.point_indices[0]];
    let (p2x, p2y, _) = local_max_points[triangle.point_indices[1]];
    let (p3x, p3y, _) = local_max_points[triangle.point_indices[2]];

    let (xmin, ymin, xmax, ymax) = image_bounding_box(width, height, image);
    let xrange = xmax - xmin;
    let yrange = ymax - ymin;

    let w = image.width as f32;
    let h = image.height as f32;

    let p1x = p1x / w * xrange + xmin;
    let p1y = p1y / h * yrange + ymin;
    let p2x = p2x / w * xrange + xmin;
    let p2y = p2y / h * yrange + ymin;
    let p3x = p3x / w * xrange + xmin;
    let p3y = p3y / h * yrange + ymin;

    rast::rast_triangle_wireframe(
        frame_buffer,
        width,
        height,
        p1x as i32,
        p1y as i32,
        p2x as i32,
        p2y as i32,
        p3x as i32,
        p3y as i32,
        color,
    );
}
