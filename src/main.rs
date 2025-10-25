use tint::Srgb;

fn main() {
    let frame_buffer =
        glazer::static_frame_buffer!(spack::WIDTH, spack::HEIGHT, Srgb, Srgb::from_rgb(0, 0, 0));
    glazer::run(
        fetch_memory(),
        frame_buffer,
        spack::WIDTH,
        spack::HEIGHT,
        spack::handle_input,
        spack::update_and_render,
        #[cfg(debug_assertions)]
        Some("target/debug/libspack.dylib"),
        #[cfg(not(debug_assertions))]
        None,
    );
}

fn create_memory() -> spack::Memory {
    let memory = spack::Memory::default();
    #[cfg(debug_assertions)]
    {
        let config = bincode::config::standard()
            .with_little_endian()
            .with_fixed_int_encoding();
        let data_path = "data/.cache";
        let memory_bytes = bincode::encode_to_vec(&memory, config).unwrap();
        std::fs::write(data_path, memory_bytes).unwrap();
    }
    memory
}

#[cfg(not(debug_assertions))]
fn fetch_memory() -> spack::Memory {
    create_memory()
}

#[cfg(debug_assertions)]
fn fetch_memory() -> spack::Memory {
    let config = bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding();
    let data_path = "data/.cache";

    let last_image_write = std::fs::metadata("src/image.rs")
        .unwrap()
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if std::fs::exists(data_path).unwrap()
        && std::fs::metadata(data_path)
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            > last_image_write
    {
        let memory_data = std::fs::read(data_path).unwrap();
        let (memory, _) = bincode::decode_from_slice(&memory_data, config).unwrap();
        memory
    } else {
        create_memory()
    }
}
