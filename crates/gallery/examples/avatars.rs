//! A separate app owns image decoding/textures and uses the public avatar API.
//! Run: cargo run -p adequate-gallery --example avatars
// The shared helper is compiled separately for each example; this one uses setup.
#[allow(dead_code)]
mod support;
use yakui_adequate_ui as ui;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    support::run_initialized(|yak| {
        let photo = image::load_from_memory(include_bytes!("../assets/avatars/shadcn.jpg"))
            .unwrap()
            .into_rgba8();
        let (width, height) = photo.dimensions();
        let mut texture = yakui::paint::Texture::new(
            yakui::paint::TextureFormat::Rgba8Srgb,
            [width, height].into(),
            photo.into_raw(),
        );
        texture.min_filter = yakui::paint::TextureFilter::Linear;
        let image = ui::AvatarImage::new(
            yak.add_texture(texture).into(),
            ui::Vec2::new(width as f32, height as f32),
        );
        let mut show_image = true;
        move || {
            ui::width(384., || {
                ui::card(|| {
                    ui::card_header(|| {
                        ui::card_title("Team");
                        ui::card_description("The app owns images; avatars own presentation.");
                    });
                    ui::card_content(|| {
                        ui::column(20., || {
                            ui::row(12., || {
                                ui::Avatar::new("CN")
                                    .image(show_image.then_some(image))
                                    .size(ui::AvatarSize::Lg)
                                    .badge(ui::AvatarBadge::new())
                                    .show();
                                ui::label("shadcn");
                            });
                            ui::avatar_group(|| {
                                ui::Avatar::new("CN")
                                    .image(show_image.then_some(image))
                                    .show();
                                ui::Avatar::new("ER").show();
                                ui::AvatarGroupCount::new("+3").show();
                            });
                            ui::checkbox("Show profile image", &mut show_image);
                        })
                    });
                    ui::card_footer(|| {
                        if ui::button("Reset to initials").clicked {
                            show_image = false;
                        }
                    });
                })
            })
        }
    })
}
