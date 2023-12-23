#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::time::Instant;

use chrono::{DateTime, Local, Utc};
use eframe::egui;
use json_operations::JsonKey;
use logic::log;

mod logic;
mod json_operations;

struct MyApp {
    last_update: Instant,
    // Is this the first time we load
    initial_load: bool,
    // Shared data
    data: Arc<Mutex<Option<(String, String)>>>,
    loading: Arc<AtomicBool>,
    username: Arc<Mutex<String>>,
    // Flag for current loading status
    api_key: Arc<Mutex<String>>,
    // Timer to check if we saved credential in last 5 sec
    save_credential_time: Instant,
    local_time: DateTime<Local>,
    utc_time: DateTime<Utc>,
}

/// Entry point for the program.
///
/// This function initializes the `MyApp` struct, sets up the `options` for the eframe window,
/// and starts running the application using `eframe::run_native`.
pub fn main() {
    let contend = MyApp {
        last_update: Instant::now(),
        initial_load: true,
        data: Arc::new(Mutex::new(None)),
        loading: Arc::new(AtomicBool::new(false)),
        username: Arc::new(Mutex::new(String::new())),
        api_key: Arc::new(Mutex::new(String::new())),
        save_credential_time: Instant::now() - Duration::from_secs(6), // Subtract 6 seconds
        // so later check >= 5 is false at the beginning
        local_time: Local::now(),
        utc_time: Utc::now(),
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(
            [750.0, 725.0]), // [x, y]
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
                self.local_time = Local::now();
                self.utc_time = Utc::now();

                // Clone for use in new Thread
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
                    ui.add_space(25.0);

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
                    ui.add_space(25.0);

                    ui.label(format!("Data will be refreshed every five minutes, \
                    last request time was at: {}lcl ({}z)",
                                     self.local_time.format("%H:%M"),
                                     self.utc_time.format("%H:%M")));

                    ui.add_space(25f32);

                    ui.heading("Departure");
                    ui.label(format!("{}", departure_val));

                    ui.add_space(25.0);

                    ui.heading("Arrival");
                    ui.label(format!("{}", arrival_val));
                }
            }

            ui.add_space(25.0);

            // Add a way to store credentials
            egui::CollapsingHeader::new("Set Credentials")
                .show(ui, |ui| {
                    let mut username = self.username.lock().unwrap();
                    let mut api_key = self.api_key.lock().unwrap();

                    ui.horizontal(|ui| {
                        ui.label("Username:");
                        ui.text_edit_singleline(&mut *username);
                    });

                    ui.horizontal(|ui| {
                        ui.label("API Key:     ");
                        ui.text_edit_singleline(&mut *api_key);
                    });

                    if ui.button("Save").clicked() {
                        // Set data if not empty
                        if !username.trim().is_empty() || !api_key.trim().is_empty() {
                            // Set username if not empty
                            if !username.trim().is_empty() {
                                json_operations::set_json_data(JsonKey::Name, username.trim().to_string());
                                log("Replacing username")
                            }
                            // Set API-Key if not empty
                            if !api_key.trim().is_empty() {
                                json_operations::set_json_data(JsonKey::Key, api_key.trim().to_string());
                                log("Replacing API-Key")
                            }
                            // Display changed data message
                            self.save_credential_time = Instant::now();
                            // Reload on change of data
                            self.last_update = Instant::now() - five_mins;
                        }
                        // Clear both fields, even if no contend
                        username.clear();
                        api_key.clear();
                    }

                    // If a credential was saved in the last five seconds
                    if self.save_credential_time.elapsed() <= Duration::from_secs(5) {
                        // Display success message
                        // Note: the program would panic if not successful, so we can assume it worked
                        ui.colored_label(egui::Color32::GREEN, "Success! Data has been saved.");
                    }
                });
        });
    }
}
