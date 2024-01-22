use dandy_draw::egui::EguiDrawer;
use eframe::egui;
use egui::{FontSelection, TextStyle};
use dandy::dfa::Dfa;
use dandy::nfa::Nfa;

fn example_dfa() -> Dfa {
    dandy::parser::dfa(
        include_str!("../../dandy-cli/src/example.dfa")
    ).unwrap().try_into().unwrap()
}

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

    let mut dfa = example_dfa().to_table();
    let mut dfa_to_render = dfa.clone();

    eframe::run_simple_native("Display DFAs", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut dfa).font(FontSelection::Style(TextStyle::Monospace))
            );

            if ui.button("Render").clicked() {
                dfa_to_render = dfa.clone();
            }

            egui::Area::new("DFA").show(ui.ctx(), |ui| {
                let painter = ui.painter();
                let mut drawer = EguiDrawer::new(painter);
                if let Some(Ok(dfa)) = dandy::parser::dfa(&dfa_to_render).ok().map(TryInto::try_into) {
                    dandy_draw::draw_dfa(&dfa, &mut drawer);
                }
            });
        });
    })
}
