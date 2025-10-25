// TODO: Hand rolled png decoder: https://github.com/madler/zlib/blob/master/contrib/puff/puff.c

use crate::image::ImageMemory;
use tint::Srgb;

mod align;
mod image;
mod process;
mod render;

pub const WIDTH: usize = 900;
pub const HEIGHT: usize = 900;

#[derive(bincode::Encode, bincode::Decode)]
pub struct Memory {
    #[bincode(with_serde)]
    images: ImageMemory,
    #[bincode(with_serde)]
    view: View,
    #[allow(unused)]
    alpha: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
enum View {
    Raw,
    LoG,
    Dilate,
    LocalMax,
    AlignTriangles,
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            images: ImageMemory::default(),
            view: View::Raw,
            alpha: 1.0,
        }
    }
}

impl Memory {
    fn next_image_index(&self) -> usize {
        assert!(!self.images.raw.is_empty());
        if self.images.selected_image == 0 {
            self.images.raw.len() - 1
        } else {
            self.images.selected_image - 1
        }
    }
}

#[unsafe(no_mangle)]
pub fn handle_input(glazer::PlatformInput { memory, input }: glazer::PlatformInput<Memory>) {
    if let glazer::Input::Key {
        code,
        pressed: true,
        ..
    } = input
    {
        match code {
            glazer::KeyCode::LeftArrow => {
                memory.images.selected_image =
                    (memory.images.selected_image + 1) % memory.images.raw.len();
            }
            glazer::KeyCode::RightArrow => {
                memory.images.selected_image = memory.next_image_index();
            }
            glazer::KeyCode::Num1 => {
                memory.view = View::Raw;
            }
            glazer::KeyCode::Num2 => {
                memory.view = View::LoG;
            }
            glazer::KeyCode::Num3 => {
                memory.view = View::Dilate;
            }
            glazer::KeyCode::Num4 => {
                memory.view = View::LocalMax;
            }
            glazer::KeyCode::Num5 => {
                memory.view = View::AlignTriangles;
            }
            _ => {}
        }
    }
}

#[unsafe(no_mangle)]
pub fn update_and_render(
    glazer::PlatformUpdate {
        memory,
        frame_buffer,
        width,
        height,
        ..
    }: glazer::PlatformUpdate<Memory, Srgb>,
) {
    render::render(frame_buffer, width, height, memory);
}
