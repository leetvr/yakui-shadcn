//! Run: cargo run -p adequate-gallery --example checkout
mod support;
use yakui_adequate_ui as ui;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut email = String::new();
    let mut notes = String::new();
    let mut agreed = false;
    let mut submitted = false;
    support::run(move || {
        ui::width(420., || {
            ui::form(|| {
                ui::heading(2, "Checkout");
                ui::field(
                    "Email",
                    "Your receipt will be sent here.",
                    if submitted && !email.contains('@') {
                        Some("Enter a valid email address.")
                    } else {
                        None
                    },
                    || {
                        ui::input(&mut email, "you@example.com");
                    },
                );
                ui::field("Notes", "", None, || {
                    ui::textarea(&mut notes, "Delivery instructions");
                });
                ui::checkbox("I agree to the terms", &mut agreed);
                if ui::Button::new("Submit").disabled(!agreed).show().clicked {
                    submitted = true;
                }
                if submitted && email.contains('@') {
                    ui::alert(
                        "Order received",
                        "This example keeps its data in memory.",
                        false,
                    );
                }
            })
        })
    })
}
