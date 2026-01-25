use eframe::egui;


fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "egui app",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

#[derive(Default)]
struct MyApp {
    label: String,
    value: f32,
}

impl eframe::App for MyApp {
    //called every frame
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("egui thing");
            ui.horizontal(|ui| {
                ui.label("input: ");
                ui.text_edit_multiline(&mut self.label);
            });
            ui.add(egui::Slider::new(&mut self.value, 0.0..=100.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }
            ui.label(format!("meowww {}, {}", self.label, self.value));
        });
    }
}

//meowww https://hackmd.io/@Hamze/Sys9nvF6Jl