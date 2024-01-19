use eframe::egui;
use egui::{Color32, pos2};
use dandy::dfa::Dfa;
use dandy::nfa::Nfa;
use crate::lib::EguiDrawer;

mod lib;

fn test_ascii_draw() {
    let str = include_str!("../../dandy-cli/src/example2.dfa");
    let dfa: Dfa = dandy::parser::dfa(str).unwrap().try_into().unwrap();
    let ascii_art = dandy_draw::dfa_ascii_art(&dfa);
    println!("{ascii_art}");

    let str = include_str!("../../dandy-cli/src/example2.nfa");
    let nfa: Nfa = dandy::parser::nfa(str).unwrap().try_into().unwrap();
    let ascii_art = dandy_draw::nfa_ascii_art(&nfa);
    println!("{ascii_art}");
}

fn main() -> Result<(), eframe::Error> {
    test_ascii_draw();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    // Our application state:
    let mut name = "Arthur".to_owned();
    let mut age = 42;

    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            painter.circle_filled(pos2(50.0, 50.0), 20.0, Color32::from_rgb(255, 0, 0));

            let mut drawer = EguiDrawer { painter };
            dandy_draw::draw_demo(&mut drawer);

            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                age += 1;
            }
            ui.label(format!("Hello '{name}', age {age}"));
        });
    })
}
