// TODO Replace current exit with way to free all resources
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{process, thread};
use std::fs::File;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, Utc};
use eframe::egui;

use json_operations::JsonKey;
use logic::log;

mod logic;
mod json_operations;

struct DataCarrier {
    // Time since last request
    last_update: Instant,
    // Shared data
    data: Arc<Mutex<Option<(String, String)>>>,
    // Flag for current loading status
    loading: Arc<AtomicBool>,
    // Credentials to store on button press
    username: Arc<Mutex<String>>,
    api_key: Arc<Mutex<String>>,
    // Timer to check if we saved credential in last 5 sec
    save_credential_time: Instant,
    // The time of last request
    local_time: DateTime<Local>,
    utc_time: DateTime<Utc>,
    // Departure and arrival
    departure: String,
    arrival: String,
    // Flag if we are loading a flight plan through button click
    loading_flight_plan: bool,
    flight_plan_update: Option<mpsc::Receiver<(String, String)>>,
    // Flag if user changed SimBrief username
    username_changed: bool,
    // Flag to check if user wants to pause calls
    stop_updating: bool,
    // Flag if the user is manually updating and thus overriding the checkbox for exactly one time
    manual_update: bool,
}

pub fn main() {

    // Create empty log file
    File::create(logic::LOGFILE_NAME).expect("Unable to create Logfile");

    // Initially call Simbrief to get the flight plan

    let contend = DataCarrier {
        last_update: Instant::now(), // Initially data will be loaded because we simulate click of reload fp button
        data: Arc::new(Mutex::new(None)),
        loading: Arc::new(AtomicBool::new(false)),
        username: Arc::new(Mutex::new(String::new())),
        api_key: Arc::new(Mutex::new(String::new())),
        save_credential_time: Instant::now() - Duration::from_secs(6), // Subtract 6 seconds
        // so later check >= 5 is false at the beginning
        local_time: Local::now(),
        utc_time: Utc::now(),
        departure: String::new(),
        arrival: String::new(),
        loading_flight_plan: false,
        flight_plan_update: None,
        username_changed: true,
        stop_updating: false,
        manual_update: false,
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
            Box::<DataCarrier>::new(contend)
        }),
    ).unwrap_or_else(|err| {
        let msg = format!("Failed to run Egui frame: {err}");
        let msg = msg.as_str();
        log(msg);
        process::exit(1);
    });

    println!("Shutting down")
}

impl eframe::App for DataCarrier {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let five_mins = Duration::from_secs(5 * 60);

            ui.horizontal(|ui| {
                // Give the user a way to manually reload
                if ui.button("Reload data").clicked() {
                    self.last_update = Instant::now() - five_mins;
                    self.manual_update = true;
                }

                if self.loading_flight_plan {
                    // Only show while updating
                    ui.label("Loading Flight Plan...");
                    ui.spinner();

                    if let Some(ref flight_plan_update) = self.flight_plan_update {
                        match flight_plan_update.try_recv() {
                            Ok((departure, arrival)) => {
                                // Update received, apply it
                                self.departure = departure;
                                self.arrival = arrival;
                                // Force an update, regardless if paused
                                self.last_update = Instant::now() - five_mins;
                                self.manual_update = true;

                                // Stop loading and clear the Receiver
                                self.loading_flight_plan = false;
                                self.flight_plan_update = None;
                            }
                            // If no update received yet, nothing to do
                            Err(_) => (),
                        }
                    }
                } else {
                    // Reload flight plan if button clicked or SimBrief username is changed
                    if ui.button("Reload Flight Plan").clicked() || self.username_changed {
                        // Begin loading
                        self.loading_flight_plan = true;
                        self.username_changed = false;

                        // Reset the Receiver
                        let (tx, rx) = mpsc::channel();
                        self.flight_plan_update = Some(rx);

                        // Spawn a new thread to perform the update
                        thread::spawn(move || {
                            let (departure, arrival) = logic::update_fp();

                            // Send the update back to the main thread
                            tx.send((departure, arrival)).unwrap();
                        });
                    }
                }

                // Checkbox for users to stop automatic updates
                // In cruise you usually don't need those constant calls
                let text = "Suppress automatic updates";
                ui.checkbox(&mut self.stop_updating, text);
            });

            // Was the last update > 5 mins ago?
            if (!self.stop_updating || self.manual_update) && self.last_update.elapsed() >= five_mins {
                // Set times
                self.local_time = Local::now();
                self.utc_time = Utc::now();
                // Reset activation conditions
                self.last_update = Instant::now();
                self.manual_update = false;

                // Clone for use in new Thread
                let data_to_update = self.data.clone();
                let loading_status = self.loading.clone();

                loading_status.store(true, Ordering::Relaxed); // Set loading status

                // Clone the fields to use in new thread
                let departure = self.departure.clone();
                let arrival = self.arrival.clone();

                thread::spawn(move || {
                    let new_data = logic::update_data(&departure, &arrival);

                    // Update shared data
                    match data_to_update.lock() {
                        Ok(mut data) => {
                            *data = Some(new_data);
                        }
                        Err(err) => {
                            let msg = format!("Mutex was poisoned. \
                            Failed to fetch data from the `data` Mutex guard: {err}");
                            let msg = msg.as_str();
                            log(msg);
                            process::exit(1);
                        }
                    }

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
                match self.data.lock() {
                    Ok(data) => {
                        // If data is available, display it
                        if let Some((departure_val, arrival_val)) = data.as_ref() {
                            if !self.loading.load(Ordering::Relaxed) {
                                ui.add_space(25.0);

                                ui.label(format!("Data will be refreshed every five minutes, \
                                        last request time was at: {}lcl ({}z)",
                                                 self.local_time.format("%H:%M"),
                                                 self.utc_time.format("%H:%M")));
                            }
                            ui.add_space(25.0);

                            ui.heading("Departure");
                            ui.label(format!("{}", departure_val));

                            ui.add_space(25.0);

                            ui.heading("Arrival");
                            ui.label(format!("{}", arrival_val));
                        }
                    }
                    Err(err) => {
                        let msg = format!("Mutex was poisoned. \
                            Failed to fetch data from the `data` Mutex guard: {err}");
                        let msg = msg.as_str();
                        log(msg);
                        process::exit(1);
                    }
                }
            }

            ui.add_space(25.0);

            // Add a way to store credentials
            egui::CollapsingHeader::new("Set Credentials")
                .show(ui, |ui| {
                    let mut username = match self.username.lock() {
                        Ok(name) => name,
                        Err(err) => {
                            let msg = format!("Mutex was poisoned. \
                            Failed to fetch data from the `data` Mutex guard: {err}");
                            let msg = msg.as_str();
                            log(msg);
                            process::exit(1);
                        }
                    };
                    let mut api_key = match self.api_key.lock() {
                        Ok(key) => key,
                        Err(err) => {
                            let msg = format!("Mutex was poisoned. \
                            Failed to fetch data from the `data` Mutex guard: {err}");
                            let msg = msg.as_str();
                            log(msg);
                            process::exit(1);
                        }
                    };

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
                            // Set username if not empty and different
                            if !username.trim().is_empty() &&
                                username.trim() != json_operations::get_json_data(JsonKey::Name) {
                                json_operations::set_json_data(JsonKey::Name, username.trim());
                                log("Replacing username");
                                // Reload Flight Plan from SimBrief with new username
                                self.username_changed = true;
                            }
                            // Set API-Key if not empty but different
                            if !api_key.trim().is_empty() &&
                                api_key.trim() != json_operations::get_json_data(JsonKey::Key) {
                                json_operations::set_json_data(JsonKey::Key, api_key.trim());
                                log("Replacing API-Key");
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