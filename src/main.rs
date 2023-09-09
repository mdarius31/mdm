use eframe::egui;
use freedesktop_entry_parser::parse_entry;
use home;
use std::fs;

fn main() {
    let latin_letters = (b'a'..=b'z').map(char::from);
    let arabic_numerals = (b'0'..=b'9').map(char::from);

    let _all_chars: Vec<char> = latin_letters
        .chain(arabic_numerals)
        .chain(" ;[]\'\\,./".chars())
        .collect();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 480.0)),
        always_on_top: true,
        decorated: false,
        resizable: false,
        centered: true,
        transparent: true,
        ..Default::default()
    };

    eframe::run_native("MDM", options, Box::new(|_cc| Box::<App>::default())).unwrap();
}

#[derive(Debug, Clone)]
struct DesktopFile {
    name: String,
    exec: String,
    path: String,
}

struct App {
    entries: Vec<DesktopFile>,
    filtered_entries: Vec<DesktopFile>,
    search: String,
}

impl Default for App {
    fn default() -> Self {
        let entries = get_entries();
        Self {
            filtered_entries: entries.clone(),
            entries: entries,
            search: String::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(egui::TextEdit::singleline(&mut self.search).desired_width(f32::INFINITY))
                .request_focus();
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_width(f32::INFINITY)
                .show(ui, |ui| {
                    for entry in &self.filtered_entries {
                        if ui.button(&entry.name).clicked() {
                            println!("{}", &entry.path);
                        }
                    }
                });
        });
    }
}

fn get_entries() -> Vec<DesktopFile> {
    let mut entries: Vec<DesktopFile> = Vec::new();
    let mut paths: Vec<String> = vec![String::from("/usr/share/applications/")];
    let local_applications = match home::home_dir() {
        Some(s) => match s.into_os_string().into_string() {
            Ok(s) => s + "/.local/share/applications/",
            Err(s) => {
                println!("Could not get the string from {s:?}!");
                String::from("")
            }
        },
        None => {
            println!("Could not get your home directory!");
            String::from("")
        }
    };
    if !local_applications.is_empty() {
        paths.push(local_applications);
    }
    for path in paths {
        let list = match fs::read_dir(&path) {
            Ok(list) => list,
            //if the directory is invalid we just ignore it
            Err(err) => {
                println!("Error: {err:?} \n Directory is invalid, continuing");
                continue;
            }
        };
        for item in list {
            let entry: fs::DirEntry = match item {
                Ok(entry) => entry,
                Err(err) => {
                    println!("Error: {err:?} \n DirEntry is invalid, continuing");
                    continue;
                }
            };
            let filename = match entry.file_name().into_string() {
                Ok(filename) => {
                    if filename.ends_with("desktop") {
                        filename
                    } else {
                        println!(" {filename:?} \n entry is not a desktop file, continuing");
                        continue;
                    }
                }
                Err(err) => {
                    println!("Error: {err:?} \n DirEntry is invalid, continuing");
                    continue;
                }
            };
            let final_path = path.clone() + &filename;
            match parse_entry(&final_path) {
                Ok(parsed) => {
                    let sec = parsed.section("Desktop Entry");
                    match (sec.attr("Name"), sec.attr("Exec")) {
                        (Some(name), Some(exec)) => {
                            entries.push(DesktopFile {
                                name: String::from(name),
                                exec: String::from(exec),
                                path: final_path.clone(),
                            });
                        }
                        _ => {
                            continue;
                        }
                    }
                }
                Err(err) => {
                    println!("Error: {err:?} \n Path couldnt be parsed, continuing");
                    continue;
                }
            };
        }
    }
    entries
}
