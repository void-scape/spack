use tint::Srgb;

fn main() {
    let frame_buffer =
        glazer::static_frame_buffer!(spack::WIDTH, spack::HEIGHT, Srgb, Srgb::from_rgb(0, 0, 0));
    glazer::run(
        spack::Memory::default(),
        frame_buffer,
        spack::WIDTH,
        spack::HEIGHT,
        spack::handle_input,
        spack::update_and_render,
        None,
    );
}
