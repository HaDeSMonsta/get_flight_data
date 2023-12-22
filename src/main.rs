#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

mod logic;
mod get_json;

pub fn main() {

    let mut contend = MyApp{
        foo: String::from("Foo"),
        bar: String::from("Bar"),
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Get Flight Data",
        options,
        Box::new(|_| {
            Box::<MyApp>::new(contend)
        }),
    ).expect("Run should be running");

    println!("Shutting down")
}

struct MyApp {
    foo: String,
    bar: String,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Heading");

            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.foo)
                  .labelled_by(name_label.id);
            });

            ui.label(format!("Hello '{}', age {}", self.foo, self.bar));
        });
    }
}
