extern crate basalt;
extern crate shellexpand;

pub mod category;
pub mod desktop;
pub mod menu;

use basalt::{
    input::{InputHookRes, Qwery},
    Basalt,
};
use category::Category;
use desktop::{DesktopEntry, DesktopEntryErr};
use menu::Menu;
use std::{path::PathBuf, sync::Arc, time::Instant};

fn main() {
    Basalt::initialize(
        basalt::Options::default()
            .ignore_dpi(true)
            .window_size(413, 439)
            .title("Pyroxene")
            .composite_alpha(basalt::vulkano::swapchain::CompositeAlpha::PreMultiplied)
            .app_loop(),
        Box::new(move |basalt_res| {
            let basalt = basalt_res.expect("Failed to initilize basalt!");
            let basalt_cp = basalt.clone();
            let start = Instant::now();

            basalt.input_ref().on_key_press(
                Qwery::Esc,
                Arc::new(move |_| {
                    basalt_cp.exit();
                    InputHookRes::Success
                }),
            );

            let search_dirs = vec![
                PathBuf::from("/usr/share/applications/"),
                shellexpand::tilde("~/.local/share/applications/").into_owned().into(),
            ];

            let mut categories = vec![
                Category::new("utility", "Accessories", "applications-utilities"),
                Category::new("development", "Development", "applications-development"),
                Category::new("education", "Education", "applications-science"),
                Category::new("game", "Games", "applications-games"),
                Category::new("graphics", "Graphics", "applications-graphics"),
                Category::new("audiovideo", "Multimedia", "applications-multimedia"),
                Category::new("network", "Network", "applications-internet"),
                Category::new("office", "Office", "applications-office"),
                Category::new("other", "Other", "applications-other"),
                Category::new("settings", "Settings", "applications-accessories"),
                Category::new("system", "System", "applications-system"),
            ];

            let mut files = Vec::new();

            for search_dir in &search_dirs {
                if search_dir.exists() {
                    match search_dir.read_dir() {
                        Err(e) =>
                            println!(
                                "Warning: Failed to read directory: {:?} ({})",
                                search_dir, e
                            ),
                        Ok(entries) =>
                            for entry_result in entries {
                                match entry_result {
                                    Err(e) =>
                                        println!(
                                            "Warning: Failed to read directory entry: {}",
                                            e
                                        ),
                                    Ok(entry) => {
                                        let entry_path = entry.path();

                                        if entry_path.is_file() {
                                            files.push(entry_path);
                                        }
                                    },
                                }
                            },
                    }
                }
            }

            let entries: Vec<_> = files
                .into_iter()
                .filter_map(|file| {
                    match DesktopEntry::new(&file) {
                        Ok(ok) => Some(Arc::new(ok)),
                        Err(e) =>
                            match e {
                                DesktopEntryErr::NotApplication
                                | DesktopEntryErr::Hidden
                                | DesktopEntryErr::OnlyShowIn
                                | DesktopEntryErr::NotShowIn => None,
                                e => {
                                    println!(
                                        "Failed to parse desktop file: {:?}: {:?}",
                                        file, e
                                    );
                                    None
                                },
                            },
                    }
                })
                .collect();

            for category in &mut categories {
                category.add_entries(&entries);
            }

            categories.retain(|c| !c.entries.is_empty());
            let _menu = Menu::new(basalt.clone(), categories, entries);

            println!("Launched in {} ms!", start.elapsed().as_micros() as f32 / 1000.0);
            basalt.wait_for_exit().unwrap();
        }),
    );
}
