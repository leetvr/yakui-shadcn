//! Gallery-owned texture decoding. Components receive public AvatarImage values.
use yakui_shadcn as ui;
#[derive(Clone, Default)]
pub struct Assets {
    pub color: [Option<ui::AvatarImage>; 4],
    pub grayscale: [Option<ui::AvatarImage>; 4],
}
pub fn load(yak: &mut yakui::Yakui) {
    let mut assets = Assets::default();
    for (index, bytes) in [
        include_bytes!("../assets/avatars/shadcn.jpg").as_slice(),
        include_bytes!("../assets/avatars/01.png").as_slice(),
        include_bytes!("../assets/avatars/02.png").as_slice(),
        include_bytes!("../assets/avatars/03.png").as_slice(),
    ]
    .into_iter()
    .enumerate()
    {
        let decoded = image::load_from_memory(bytes)
            .expect("pinned gallery avatar")
            .into_rgba8();
        let dimensions = decoded.dimensions();
        let pixels = decoded.into_raw();
        for grayscale in [false, true] {
            let mut data = pixels.clone();
            if grayscale {
                for pixel in data.chunks_exact_mut(4) {
                    let value = (pixel[0] as f32 * 0.2126
                        + pixel[1] as f32 * 0.7152
                        + pixel[2] as f32 * 0.0722)
                        .round() as u8;
                    pixel[..3].fill(value);
                }
            }
            let mut texture = yakui::paint::Texture::new(
                yakui::paint::TextureFormat::Rgba8Srgb,
                [dimensions.0, dimensions.1].into(),
                data,
            );
            texture.min_filter = yakui::paint::TextureFilter::Linear;
            let id = yak.add_texture(texture);
            let image = Some(ui::AvatarImage::new(
                id.into(),
                ui::Vec2::new(dimensions.0 as f32, dimensions.1 as f32),
            ));
            if grayscale {
                assets.grayscale[index] = image;
            } else {
                assets.color[index] = image;
            }
        }
    }
    yak.dom().get_global_or_init(|| assets);
}
pub fn avatar(index: usize, grayscale: bool) -> Option<ui::AvatarImage> {
    let assets = yakui::context::dom().get_global_or_init(Assets::default);
    if grayscale {
        assets.grayscale[index]
    } else {
        assets.color[index]
    }
}
