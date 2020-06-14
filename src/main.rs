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

fn main() {
    Basalt::initialize(
		basalt::Options::default()
			.ignore_dpi(true)
			.window_size(275, 262)
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
            let total_bins = categories.iter().map(|c| c.entries.len() + 1).sum();
            let mut bins = basalt.interface_ref().new_bins(total_bins);
            let mut category_bins = Vec::new();

            for (ci, category) in categories.iter().enumerate() {
                let category_bin = bins.pop().unwrap();

                category_bin.style_update(BinStyle {
                    pos_from_t: Some(ci as f32 * 24.0),
                    pos_from_l: Some(0.0),
                    width: Some(125.0),
                    height: Some(24.0),
                    back_color: Some(Color::srgb_hex("303030ff")),
                    pad_t: Some(6.0),
                    pad_l: Some(6.0),
                    pad_r: Some(6.0),
                    text: category.name.clone(),
                    text_height: Some(12.0),
                    text_color: Some(Color::srgb_hex("f8f8f8ff")),
                    overflow_y: Some(true),
                    .. BinStyle::default()
                });

                for (ei, entry) in category.entries.iter().enumerate() {
                    let entry_bin = bins.pop().unwrap();
                    category_bin.add_child(entry_bin.clone());

                    entry_bin.style_update(BinStyle {
                        hidden: Some(true),
                        position: Some(BinPosition::Parent),
                        pos_from_t: Some(ei as f32 * 24.0),
                        pos_from_l: Some(125.0),
                        width: Some(150.0),
                        height: Some(24.0),
                        back_color: Some(Color::srgb_hex("303030ff")),
                        pad_t: Some(6.0),
                        pad_l: Some(6.0),
                        pad_r: Some(6.0),
                        text: entry.name.clone(),
                        text_height: Some(12.0),
                        text_color: Some(Color::srgb_hex("f8f8f8ff")),
                        text_wrap: Some(ImtTextWrap::None),
                        .. BinStyle::default()
                    });

                    if ei == category.entries.len() - 1 {
                        let entry_bin_cp = entry_bin.clone();
                        let basalt_cp = basalt.clone();

                        entry_bin.on_update(Arc::new(move || {
                            let post = entry_bin_cp.post_update();

                            if basalt_cp.window().inner_dimensions()[1] < post.bro[1].ceil() as u32 {
                                basalt_cp.window().request_resize(275, post.bro[1].ceil() as u32 + 1);
                                basalt_cp.force_recreate_swapchain();
                            }
                        }));
                    }

                    let exec = entry.exec.clone();
                    let basalt_cp = basalt.clone();

                    entry_bin.on_mouse_press(MouseButton::Left, Arc::new(move |_, _| {
                        Command::new("sh")
                            .arg("-c")
                            .arg(&exec)
                            .spawn()
                            .unwrap();
                        
                        basalt_cp.exit();
                    }));

                    category_bin.keep_alive(entry_bin);
                }

                category_bins.push(category_bin);
            }

            let category_bins_cp = category_bins.clone();
            let mouse_enter_func: BinHookFn = Arc::new(move |bin: Arc<Bin>, _| {
                for cbin in &category_bins_cp {
                    if cbin.id() == bin.id() {
                        cbin.children().into_iter().for_each(|c| c.hidden(Some(false)));
                    } else {
                        cbin.children().into_iter().for_each(|c| c.hidden(Some(true)));
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