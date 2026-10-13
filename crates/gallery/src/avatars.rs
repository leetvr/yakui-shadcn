//! Pinned avatar compositions using public components and gallery-owned textures.
use ui::{Avatar, AvatarSize as S, Icon as I, MenuItem as M};
use yakui_shadcn as ui;
pub const VARIANTS: &[&str] = &[
    "default",
    "basic",
    "badge",
    "badge-icon",
    "group",
    "group-count",
    "group-count-icon",
    "size",
    "dropdown",
];
fn user(index: usize, grayscale: bool, images: bool) -> Avatar {
    Avatar::new(["CN", "LR", "ER", "PP"][index]).image(if images {
        crate::assets::avatar(index, grayscale)
    } else {
        None
    })
}
fn status() -> ui::AvatarBadge {
    ui::AvatarBadge::new().color(ui::Color::hex(if ui::theme().dark {
        0x166534
    } else {
        0x16a34a
    }))
}
fn group(images: bool, count: Option<bool>) {
    ui::avatar_group(|| {
        for index in 0..3 {
            user(index, true, images).show();
        }
        if let Some(icon) = count {
            let count = ui::AvatarGroupCount::new("+3");
            if icon {
                count.icon(I::Plus).show();
            } else {
                count.show();
            }
        }
    });
}
pub fn render(variant: &str, state: &mut crate::examples::ExampleState, disabled: bool) {
    let images = state.avatar_images;
    match variant {
        "default" => ui::row(48., || {
            user(0, true, images).show();
            user(2, false, images).badge(status()).show();
            group(images, Some(false));
        }),
        "basic" => user(0, true, images).show(),
        "badge" => user(0, false, images).badge(status()).show(),
        "badge-icon" => user(3, true, images)
            .badge(ui::AvatarBadge::new().icon(I::Plus))
            .show(),
        "group" => group(images, None),
        "group-count" => group(images, Some(false)),
        "group-count-icon" => group(images, Some(true)),
        "size" => ui::row(8., || {
            for size in [S::Sm, S::Default, S::Lg] {
                user(0, true, images).size(size).show();
            }
        }),
        "dropdown" => {
            state.trigger = ui::dropdown_menu_trigger(&mut state.menu, || {
                ui::Button::new("")
                    .variant(ui::Variant::Ghost)
                    .size(ui::ButtonSize::Icon)
                    .rounded(true)
                    .disabled(disabled)
                    .show_children(|| user(0, false, images).show())
            });
        }
        _ => unreachable!("unconfigured avatar preview"),
    }
}
pub fn content(state: &mut crate::examples::ExampleState) {
    if let Some(id) = ui::dropdown_menu(&mut state.menu, &mut state.avatar_items) {
        state.count = id as u32;
    }
}
pub fn items() -> Vec<M> {
    vec![
        M::action(1, "Profile"),
        M::action(2, "Billing"),
        M::action(3, "Settings"),
        M::separator(),
        M::action(4, "Log out").destructive(true),
    ]
}
