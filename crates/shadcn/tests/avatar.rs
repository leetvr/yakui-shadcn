//! Real layout, texture painting and button composition for common avatar use.
use yakui::{Alignment, Rect, Vec2, Yakui, event::Event, input::MouseButton};
use yakui_shadcn as ui;
fn yak() -> Yakui {
    let mut y = Yakui::new();
    let size = Vec2::new(400., 200.);
    y.set_surface_size(size);
    y.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));
    ui::add_fonts(&mut y);
    y
}
#[test]
fn avatar_group_lays_out_overlapping_members_and_paints_the_application_texture() {
    let mut y = yak();
    let texture = y.add_texture(yakui::paint::Texture::new(
        yakui::paint::TextureFormat::Rgba8Srgb,
        [2, 1].into(),
        vec![255, 0, 0, 255, 0, 0, 255, 255],
    ));
    y.start();
    yakui::align(Alignment::TOP_LEFT, || {
        ui::avatar_group(|| {
            ui::Avatar::new("CN")
                .image(Some(ui::AvatarImage::new(
                    texture.into(),
                    Vec2::new(2., 1.),
                )))
                .show();
            ui::Avatar::new("ER")
                .badge(ui::AvatarBadge::new().icon(ui::Icon::Plus))
                .show();
            ui::AvatarGroupCount::new("+3").show();
        })
    });
    y.finish();
    fn visit(y: &Yakui, id: yakui::WidgetId, out: &mut Vec<Rect>) {
        let node = y.dom().get(id).unwrap();
        if node.widget.type_name().ends_with("::AvatarWidget") {
            out.push(y.layout_dom().get(id).unwrap().rect);
        }
        for &child in &node.children {
            visit(y, child, out);
        }
    }
    let mut avatars = vec![];
    visit(&y, y.dom().root(), &mut avatars);
    assert_eq!(avatars.len(), 3);
    assert_eq!(
        avatars.iter().map(|r| r.pos().x).collect::<Vec<_>>(),
        vec![0., 24., 48.]
    );
    assert!(avatars.iter().all(|r| r.size() == Vec2::splat(32.)));
    let paint = y.paint();
    let call = paint
        .layers()
        .iter()
        .flat_map(|layer| &layer.calls)
        .find(|call| call.texture == Some(texture.into()))
        .unwrap();
    assert!(
        call.vertices
            .iter()
            .all(|v| (0.25 - 1e-5..=0.75 + 1e-5).contains(&v.texcoord.x))
    );
    assert!(
        call.indices
            .iter()
            .all(|&index| (index as usize) < call.vertices.len())
    );
}
#[test]
fn avatar_button_keeps_icon_button_dimensions_and_reports_pointer_activation_once() {
    fn frame(y: &mut Yakui) -> ui::ControlResponse {
        y.start();
        let mut response = ui::ControlResponse::default();
        yakui::align(Alignment::TOP_LEFT, || {
            response = ui::Button::new("")
                .variant(ui::Variant::Ghost)
                .size(ui::ButtonSize::Icon)
                .rounded(true)
                .show_children(|| ui::Avatar::new("CN").show());
        });
        y.finish();
        response
    }
    let mut y = yak();
    let button = frame(&mut y);
    let rect = y.layout_dom().get(button.id.unwrap()).unwrap().rect;
    assert_eq!(rect.size(), Vec2::splat(32.));
    y.handle_event(Event::CursorMoved(Some(rect.center())));
    for down in [true, false] {
        y.handle_event(Event::MouseButtonChanged {
            button: MouseButton::One,
            down,
        });
    }
    assert!(frame(&mut y).clicked);
    assert!(!frame(&mut y).clicked);
}
