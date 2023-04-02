use fltk::{app, button, enums, group, prelude::*, text, window::SingleWindow};
use fltk_theme::{color_themes, ColorTheme};
use freedesktop_entry_parser::parse_entry;
use home;
use std::fs;
use std::process::Command;

#[derive(Debug, Clone)]
struct DesktopFile {
    name: String,
    exec: String,
}

fn main() {
    let entries_og: Vec<DesktopFile> = entries();
    let entries = entries_og.clone();
    let app = app::App::default().with_scheme(app::Scheme::Base);

    let theme = ColorTheme::new(color_themes::DARK_THEME);
    theme.apply();

    let wind_w = 640;
    let wind_h = 480;
    let mut wind = SingleWindow::new(0, 0, wind_w, wind_h, "mdm").center_screen();
    wind.set_border(false);

    let input_h = 30;

    let mut text_buf = text::TextBuffer::default();
    text_buf.set_text("");

    let _text = text::TextDisplay::new(0, 0, wind_w, input_h, "").set_buffer(text_buf.clone());

    let latin_letters = (b'a'..=b'z').map(char::from);
    let arabic_numerals = (b'0'..=b'9').map(char::from);

    let all_chars: Vec<char> = latin_letters
        .chain(arabic_numerals)
        .chain(" ;[]\'\\,./".chars())
        .collect();

    // let mut buts: Vec<button::Button> = Vec::new();

    let mut group_container = group::Group::new(0, input_h, wind_w, wind_h - input_h, "");

    let but_h = 30;
    create_buttons(&entries, wind_w, input_h, but_h, &mut group_container);

    wind.handle(move |_w, e| {
        let mut text = text_buf.text();

        match e {
            enums::Event::KeyDown => {
                match app::event_key() {
                    enums::Key::Escape => std::process::exit(0),
                    enums::Key::Enter => {
                        println!("Enter");

                        match group_container.child(0) {
                            Some(mut c) => {
                                println!("Executing {}", c.label());
                                c.do_callback();
                            }
                            None => (),
                        }
                    }
                    enums::Key::BackSpace => {
                        // println!("delete");
                        if text.len() > 0 {
                            text_buf.set_text(&text[..text.len() - 1]);
                        }
                    }

                    all => match all.to_char() {
                        Some(letter) => {
                            // println!("{letter}");

                            for l in all_chars.clone() {
                                if letter == l {
                                    let capital = app::event_key_down(enums::Key::ShiftL)
                                        || app::event_key_down(enums::Key::ShiftR);
                                    let letter = if capital {
                                        match letter {
                                            ';' => ':',
                                            '[' => '{',
                                            ']' => '}',
                                            '\'' => '"',
                                            '\\' => '|',
                                            ',' => '<',
                                            '.' => '>',
                                            '/' => '?',
                                            letter => letter.to_ascii_uppercase(),
                                        }
                                    } else {
                                        letter
                                    };

                                    text.push(letter);
                                    text_buf.set_text(&text)
                                }
                            }
                        }
                        _ => (),
                    },
                }

                let entries = entries_og.clone();
                let entries = entries
                    .iter()
                    .filter(move |&i| i.name.to_uppercase().contains(&text.to_uppercase()))
                    .cloned()
                    .collect();
                // println!("{:?}\n\n", entries);
                create_buttons(&entries, wind_w, input_h, but_h, &mut group_container);
                app.redraw();
            }
            _ => (),
        }
        // println!("{:?}",text_buf.text());
        true
    });
    wind.end();
    wind.show();

    app.run().unwrap();
}

fn entries() -> Vec<DesktopFile> {
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
            match parse_entry(final_path) {
                Ok(parsed) => {
                    let sec = parsed.section("Desktop Entry");
                    match (sec.attr("Name"), sec.attr("Exec")) {
                        (Some(name), Some(exec)) => {
                            entries.push(DesktopFile {
                                name: String::from(name),
                                exec: String::from(exec),
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

trait StringToStaticStr {
    fn to_static_str(&self) -> Option<&'static str>;
}

impl StringToStaticStr for String {
    fn to_static_str(&self) -> Option<&'static str> {
        Some(Box::leak(self.clone().into_boxed_str()))
    }
}
fn create_buttons(
    entries: &Vec<DesktopFile>,
    wind_w: i32,
    input_h: i32,
    but_h: usize,
    parent: &mut group::Group,
) {
    for w in parent.clone().into_iter() {
        parent.remove(&w);
    }

    for (i, entry) in entries.iter().enumerate() {
        // println!("{entry:?}");
        let s: &str = &entry.name.to_static_str().unwrap();

        let mut but =
            button::LightButton::new(0, input_h + (i * but_h) as i32, wind_w, but_h as i32, s);
        let exec = entry.exec.clone();

        but.set_callback(move |_but| {
            match Command::new("sh").arg("-c").arg(exec.clone()).spawn() {
                _ => {
                    std::process::exit(0);
                }
            };
        });
        parent.add(&but);
    }
}
