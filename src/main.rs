#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::time::Instant;

use eframe::egui;

mod logic;
mod get_json;

pub fn main() {
    let contend = MyApp {
        last_update: Instant::now(),
        initial_load: true,
        data: Arc::new(Mutex::new(None)),
        loading: Arc::new(AtomicBool::new(false)),
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),//.with_inner_size([320.0, 240.0]),
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
    last_update: Instant,
    initial_load: bool,
    // Is this the first time we load
    data: Arc<Mutex<Option<(String, String)>>>,
    // Shared data
    loading: Arc<AtomicBool>, // Flag for current loading status
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            let five_mins = Duration::from_secs(5 * 60);

            // Give the user a way to manually reload
            if ui.button("Reload data").clicked() {
                self.last_update = Instant::now() - five_mins;
            }

            // Was the last update > 5 mins ago?
            if self.last_update.elapsed() >= five_mins || self.initial_load {
                self.last_update = Instant::now();
                self.initial_load = false;

                // Clone for ue in new Thread
                let data_to_update = self.data.clone();
                let loading_status = self.loading.clone();

                loading_status.store(true, Ordering::Relaxed); // Set loading status

                thread::spawn(move || {
                    let new_data = logic::update_data();

                    // Update shared data
                    let mut data = data_to_update.lock().unwrap();
                    *data = Some(new_data);
                    drop(data); //Explicitly drop lock

                    // Now, when loading is done, set flag to false
                    loading_status.store(false, Ordering::Relaxed);
                });
            }

            // Check loading status
            {
                if self.loading.load(Ordering::Relaxed) {

                    ui.add_space(25f32);

                    ui.horizontal(|ui| {

                        ui.label("Loading data...");
                        ui.spinner();

                    });
                }
            }

            // Access shared data
            {
                let data = self.data.lock().unwrap();

                // If data is available, display it
                if let Some((departure_val, arrival_val)) = data.as_ref() {
                    ui.add_space(25f32);

                    ui.heading("Departure");
                    ui.label(format!("{}", departure_val));

                    ui.add_space(25f32);

                    ui.heading("Arrival");
                    ui.label(format!("{}", arrival_val));
                }
            }
        });
    }
}
