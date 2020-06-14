extern crate basalt;
extern crate shellexpand;

pub mod desktop;
pub mod category;

use desktop::DesktopEntry;
use category::Category;

use std::path::PathBuf;
use std::sync::Arc;
use basalt::{
	input::Qwery,
	interface::bin::{Color, BinPosition, BinStyle},
	Basalt,
};
use basalt::interface::hook::{BinHook,BinHookFn};
use basalt::interface::bin::Bin;
use basalt::ilmenite::ImtTextWrap;
use basalt::input::InputHookRes;
use basalt::input::MouseButton;
use std::process::Command;
use std::collections::BTreeMap;

fn main() {
    Basalt::initialize(
		basalt::Options::default()
			.ignore_dpi(true)
			.window_size(413, 482)
			.title("Pyroxene")
            .composite_alpha(basalt::vulkano::swapchain::CompositeAlpha::PreMultiplied)
			.app_loop(),
		Box::new(move |basalt_res| {
			let basalt = basalt_res.expect("Failed to initilize basalt!");
            let basalt_cp = basalt.clone();

            basalt.input_ref().on_key_press(Qwery::Esc, Arc::new(move |_| {
                basalt_cp.exit();
                InputHookRes::Success
            }));

            let search_dirs = vec![
                PathBuf::from("/usr/share/applications/"),
                shellexpand::tilde("~/.local/share/applications/").into_owned().into()
            ];

            let mut categories = vec![
                Category::new("utility",     "Accessories",  "applications-utilities"),
                Category::new("development", "Development",  "applications-development"),
                Category::new("education",   "Education",    "applications-science"),
                Category::new("game",        "Games",        "applications-games"),
                Category::new("graphics",    "Graphics",     "applications-graphics"),
                Category::new("audiovideo",  "Multimedia",   "applications-multimedia"),
                Category::new("network",     "Network",      "applications-internet"),
                Category::new("office",      "Office",       "applications-office"),
                Category::new("other",       "Other",        "applications-other"),
                Category::new("settings",    "Settings",     "applications-accessories"),
                Category::new("system",      "System",       "applications-system"),
            ];

            let mut files = Vec::new();

            for search_dir in &search_dirs {
                if search_dir.exists() {
                    match search_dir.read_dir() {    
                        Err(e) => println!("Warning: Failed to read directory: {:?} ({})", search_dir, e),
                        Ok(entries) => for entry_result in entries {
                            match entry_result {
                                Err(e) => println!("Warning: Failed to read directory entry: {}", e),
                                Ok(entry) => {
                                    let entry_path = entry.path();

                                    if entry_path.is_file() {
                                        files.push(entry_path);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let entries: Vec<_> = files.into_iter().filter_map(|file| match DesktopEntry::new(&file) {
                Ok(ok) => Some(Arc::new(ok)),
                Err(e) => {
                    println!("Failed to parse desktop file: {:?}: {}", file, e);
                    None
                }
            }).collect();

            for category in &mut categories {
                category.add_entries(&entries);
            }

            categories.retain(|c| !c.entries.is_empty());
            let total_bins: usize = categories.iter().map(|c| c.entries.len() + 2).sum();
            let mut bins = basalt.interface_ref().new_bins(total_bins + 1);
            let container = bins.pop().unwrap();

            container.style_update(BinStyle {
                pos_from_t: Some(0.0),
                pos_from_b: Some(0.0),
                width: Some(413.0),
                height: Some(482.0),
                back_color: Some(Color::srgb_hex("2a2a2cfa")),
                .. BinStyle::default()
            });

            let right = bins.pop().unwrap();
            container.add_child(right.clone());

            right.style_update(BinStyle {
                position: Some(BinPosition::Parent),
                pos_from_t: Some(3.0),
                pos_from_l: Some(103.0),
                pos_from_r: Some(3.0),
                pos_from_b: Some(3.0),
                back_color: Some(Color::srgb_hex("00000080")),
                .. BinStyle::default()
            });

            let mut category_bins = Vec::new();
            let mut category_entry_bins: BTreeMap<u64, Vec<Arc<Bin>>> = BTreeMap::new();

            for (ci, category) in categories.iter().enumerate() {
                let category_bin = bins.pop().unwrap();
                container.add_child(category_bin.clone());

                category_bin.style_update(BinStyle {
                    position: Some(BinPosition::Parent),
                    pos_from_t: Some((ci as f32 * 24.0) + 6.0),
                    pos_from_l: Some(8.0),
                    width: Some(88.0),
                    height: Some(22.0),
                    pad_t: Some(6.0),
                    text: category.name.clone(),
                    text_height: Some(12.5),
                    text_color: Some(Color::srgb_hex("f8f8f8ff")),
                    overflow_y: Some(true),
                    .. BinStyle::default()
                });

                let mut x = 3.0;
                let mut y = 3.0;

                for (ei, entry) in category.entries.iter().enumerate() {
                    let entry_bin = bins.pop().unwrap();
                    right.add_child(entry_bin.clone());

                    category_entry_bins
                        .entry(category_bin.id())
                        .or_insert_with(|| Vec::with_capacity(category.entries.len()))
                        .push(entry_bin.clone());

                    entry_bin.style_update(BinStyle {
                        hidden: Some(true),
                        position: Some(BinPosition::Parent),
                        pos_from_t: Some(y),
                        pos_from_l: Some(x),
                        width: Some(150.0),
                        height: Some(24.0),
                        back_color: Some(Color::srgb_hex("ffffff20")),
                        pad_t: Some(6.0),
                        pad_l: Some(6.0),
                        pad_r: Some(6.0),
                        text: entry.name.clone(),
                        text_height: Some(12.5),
                        text_color: Some(Color::srgb_hex("f8f8f8ff")),
                        text_wrap: Some(ImtTextWrap::None),
                        .. BinStyle::default()
                    });

                    y += 25.0;

                    if y >= 482.0 {
                        x += 151.0;
                        y = 3.0;
                    }

                    let exec = entry.exec.clone();
                    let basalt_cp = basalt.clone();

                    entry_bin.on_mouse_press(MouseButton::Left, Arc::new(move |_, _| {
                        Command::new("sh")
                            .arg("-c")
                            .arg(shellexpand::full(&exec).unwrap().into_owned())
                            .spawn()
                            .unwrap();
                        
                        basalt_cp.exit();
                    }));
                }

                category_bins.push(category_bin);
            }

            let category_bins_cp = category_bins.clone();

            let mouse_enter_func: BinHookFn = Arc::new(move |bin: Arc<Bin>, _| {
                for cbin in &category_bins_cp {
                    if cbin.id() == bin.id() {
                        if let Some(entry_bins) = category_entry_bins.get(&cbin.id()) {
                            entry_bins.iter().for_each(|c| c.hidden(Some(false)));
                        }

                        cbin.style_update(BinStyle {
                            border_size_b: Some(1.0),
                            border_color_b: Some(Color::srgb_hex("4040d0ff")),
                            .. cbin.style_copy()
                        });
                    } else {
                        if let Some(entry_bins) = category_entry_bins.get(&cbin.id()) {
                            entry_bins.iter().for_each(|c| c.hidden(Some(true)));
                        }

                        cbin.style_update(BinStyle {
                            border_size_b: None,
                            border_color_b: None,
                            .. cbin.style_copy()
                        });
                    }
                }
            });

            category_bins.iter().for_each(|c| {
                c.add_hook_raw(BinHook::MouseEnter, mouse_enter_func.clone());
            });

            basalt.wait_for_exit().unwrap();
        })
    );
}