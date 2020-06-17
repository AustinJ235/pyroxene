#![allow(dead_code)]

use crate::{category::Category, desktop::DesktopEntry};
use basalt::{
    ilmenite::ImtTextWrap,
    input::{Character, InputHook, InputHookData, InputHookRes, MouseButton},
    interface::{
        bin::{Bin, BinPosition, BinStyle, Color},
        hook::{BinHook, BinHookFn},
    },
    Basalt,
};
use std::{
    process::Command,
    sync::{Arc, Mutex},
};

pub struct Menu {
    basalt: Arc<Basalt>,
    categories: Vec<Arc<MenuCategory>>,
    entries: Vec<Arc<DesktopEntry>>,
    container: Arc<Bin>,
    right: Arc<Bin>,
    search: Arc<Bin>,
    search_entries: Mutex<Vec<Arc<MenuEntry>>>,
}

pub struct MenuCategory {
    category: Category,
    nav_bin: Arc<Bin>,
    entries: Vec<Arc<MenuEntry>>,
}

pub struct MenuEntry {
    entry_bin: Arc<Bin>,
    entry: Arc<DesktopEntry>,
}

impl MenuEntry {
    fn add_hooks(&self, basalt: Arc<Basalt>) {
        self.entry_bin.add_hook_raw(
            BinHook::MouseEnter,
            Arc::new(move |bin, _| {
                bin.style_update(BinStyle {
                    back_color: Some(Color::srgb_hex("ffffff16")),
                    ..bin.style_copy()
                });
            }),
        );

        self.entry_bin.add_hook_raw(
            BinHook::MouseLeave,
            Arc::new(move |bin, _| {
                bin.style_update(BinStyle {
                    back_color: Some(Color::srgb_hex("ffffff1a")),
                    ..bin.style_copy()
                });
            }),
        );

        let entry = self.entry.clone();

        self.entry_bin.on_mouse_press(
            MouseButton::Left,
            Arc::new(move |_, _| {
                let exec = entry
                    .exec
                    .replace("%f", "")
                    .replace("%F", "")
                    .replace("%u", "")
                    .replace("%U", "")
                    .replace(
                        "%i",
                        entry.icon.clone().as_ref().map(|v| v.as_str()).unwrap_or(""),
                    )
                    .replace("%c", entry.name.as_str())
                    .replace("%k", "");
                Command::new("sh").arg("-c").arg(exec).spawn().unwrap();
                basalt.exit();
            }),
        );
    }
}

impl Menu {
    pub fn new(
        basalt: Arc<Basalt>,
        categories: Vec<Category>,
        entries: Vec<Arc<DesktopEntry>>,
    ) -> Arc<Self> {
        let total_bins: usize = categories.iter().map(|c| c.entries.len() + 1).sum();
        let mut bins = basalt.interface_ref().new_bins(total_bins + 3);

        let mut menu = Menu {
            basalt,
            categories: Vec::with_capacity(categories.len()),
            entries,
            container: bins.pop().unwrap(),
            right: bins.pop().unwrap(),
            search: bins.pop().unwrap(),
            search_entries: Mutex::new(Vec::new()),
        };

        menu.container.add_child(menu.right.clone());
        menu.container.add_child(menu.search.clone());

        menu.container.style_update(BinStyle {
            position: Some(BinPosition::Window),
            pos_from_t: Some(0.0),
            pos_from_l: Some(0.0),
            pos_from_r: Some(0.0),
            pos_from_b: Some(0.0),
            back_color: Some(Color::srgb_hex("2a2a2cfc")),
            border_radius_br: Some(3.0),
            ..BinStyle::default()
        });

        menu.right.style_update(BinStyle {
            position: Some(BinPosition::Parent),
            pos_from_t: Some(3.0),
            pos_from_l: Some(103.0),
            pos_from_r: Some(3.0),
            pos_from_b: Some(36.0),
            back_color: Some(Color::srgb_hex("ffffff10")),
            border_radius_tl: Some(3.0),
            border_radius_tr: Some(3.0),
            border_radius_bl: Some(3.0),
            border_radius_br: Some(3.0),
            ..BinStyle::default()
        });

        menu.search.style_update(BinStyle {
            position: Some(BinPosition::Parent),
            pos_from_b: Some(3.0),
            pos_from_l: Some(3.0),
            pos_from_r: Some(3.0),
            height: Some(30.0),
            back_color: Some(Color::srgb_hex("ffffff35")),
            border_radius_tl: Some(3.0),
            border_radius_tr: Some(3.0),
            border_radius_bl: Some(3.0),
            border_radius_br: Some(3.0),
            pad_t: Some(9.0),
            pad_l: Some(9.0),
            pad_r: Some(9.0),
            text_color: Some(Color::srgb_hex("ffffffff")),
            text_height: Some(14.0),
            ..BinStyle::default()
        });

        for (ci, category) in categories.into_iter().enumerate() {
            let entries_len = category.entries.len();
            let mut menu_category = MenuCategory {
                category,
                nav_bin: bins.pop().unwrap(),
                entries: Vec::with_capacity(entries_len),
            };
            menu.container.add_child(menu_category.nav_bin.clone());

            menu_category.nav_bin.style_update(BinStyle {
                position: Some(BinPosition::Parent),
                pos_from_t: Some((ci as f32 * 25.0) + 6.0),
                pos_from_l: Some(3.0),
                pad_l: Some(5.0),
                width: Some(97.0),
                height: Some(24.0),
                pad_t: Some(5.0),
                text: menu_category.category.name.clone(),
                text_height: Some(12.5),
                text_color: Some(Color::srgb_hex("f8f8f8ff")),
                overflow_y: Some(true),
                ..BinStyle::default()
            });

            let mut x = 3.0;
            let mut y = 3.0;

            for entry in menu_category.category.entries.iter() {
                let menu_entry = MenuEntry {
                    entry_bin: bins.pop().unwrap(),
                    entry: entry.clone(),
                };
                menu.right.add_child(menu_entry.entry_bin.clone());

                menu_entry.entry_bin.style_update(BinStyle {
                    hidden: Some(true),
                    position: Some(BinPosition::Parent),
                    pos_from_t: Some(y),
                    pos_from_l: Some(x),
                    width: Some(150.0),
                    height: Some(24.0),
                    back_color: Some(Color::srgb_hex("ffffff1a")),
                    pad_t: Some(6.0),
                    pad_l: Some(6.0),
                    pad_r: Some(8.0),
                    border_radius_tl: Some(2.0),
                    border_radius_tr: Some(2.0),
                    border_radius_bl: Some(2.0),
                    border_radius_br: Some(2.0),
                    text: entry.name.clone(),
                    text_height: Some(12.5),
                    text_color: Some(Color::srgb_hex("f8f8f8ff")),
                    text_wrap: Some(ImtTextWrap::None),
                    ..BinStyle::default()
                });

                y += 25.0;

                if y >= 420.0 {
                    x += 151.0;
                    y = 3.0;
                }

                menu_category.entries.push(Arc::new(menu_entry));
            }

            menu.categories.push(Arc::new(menu_category));
        }

        let menu = Arc::new(menu);
        menu.add_hooks();
        menu
    }

    fn add_hooks(self: &Arc<Self>) {
        let menu = self.clone();

        let nav_enter_func: BinHookFn = Arc::new(move |bin: Arc<Bin>, _| {
            menu.search_entries.lock().unwrap().clear();

            for menu_cat in menu.categories.iter() {
                if bin.id() == menu_cat.nav_bin.id() {
                    menu_cat.entries.iter().for_each(|e| {
                        e.entry_bin.hidden(Some(false));
                    });

                    menu_cat.nav_bin.style_update(BinStyle {
                        border_radius_tl: Some(2.0),
                        border_radius_tr: Some(2.0),
                        border_radius_bl: Some(2.0),
                        border_radius_br: Some(2.0),
                        back_color: Some(Color::srgb_hex("ffffff20")),
                        ..menu_cat.nav_bin.style_copy()
                    });
                } else {
                    menu_cat.entries.iter().for_each(|e| {
                        e.entry_bin.hidden(Some(true));
                    });

                    menu_cat.nav_bin.style_update(BinStyle {
                        border_radius_tl: None,
                        border_radius_tr: None,
                        border_radius_bl: None,
                        border_radius_br: None,
                        back_color: None,
                        ..menu_cat.nav_bin.style_copy()
                    });
                }
            }
        });

        for menu_cat in self.categories.iter() {
            menu_cat.nav_bin.add_hook_raw(BinHook::MouseEnter, nav_enter_func.clone());
        }

        for menu_cat in self.categories.iter() {
            for menu_en in menu_cat.entries.iter() {
                menu_en.add_hooks(self.basalt.clone());
            }
        }

        let menu = self.clone();

        self.basalt.input_ref().add_hook(
            InputHook::Character,
            Arc::new(move |data| {
                if let InputHookData::Character {
                    character,
                } = data
                {
                    let mut text = menu.search.style_copy().text;

                    match character {
                        Character::Value(v) => {
                            text.push(*v);
                        },
                        Character::Backspace =>
                            if !text.is_empty() {
                                text.pop();
                            },
                    }

                    menu.display_search(text.clone());

                    menu.search.style_update(BinStyle {
                        text,
                        ..menu.search.style_copy()
                    });
                }

                InputHookRes::Success
            }),
        );
    }

    fn display_search(&self, text: String) {
        let mut entries = self.entries.clone();

        entries.sort_by_key(|e| {
            (strsim::jaro_winkler(e.name.as_str(), text.as_str()) * u64::max_value() as f64)
                .floor() as u64
        });

        for menu_cat in self.categories.iter() {
            menu_cat.entries.iter().for_each(|e| {
                e.entry_bin.hidden(Some(true));
            });

            menu_cat.nav_bin.style_update(BinStyle {
                border_size_b: None,
                border_color_b: None,
                ..menu_cat.nav_bin.style_copy()
            });
        }

        let mut search_entries = self.search_entries.lock().unwrap();
        search_entries.clear();
        let mut x = 3.0;
        let mut y = 3.0;
        let mut bins = self.basalt.interface_ref().new_bins(34);

        for entry in entries.into_iter().rev() {
            if search_entries.len() >= 34 {
                break;
            }

            let menu_entry = MenuEntry {
                entry_bin: bins.pop().unwrap(),
                entry: entry.clone(),
            };

            self.right.add_child(menu_entry.entry_bin.clone());

            menu_entry.entry_bin.style_update(BinStyle {
                position: Some(BinPosition::Parent),
                pos_from_t: Some(y),
                pos_from_l: Some(x),
                width: Some(150.0),
                height: Some(24.0),
                back_color: Some(Color::srgb_hex("ffffff1a")),
                pad_t: Some(6.0),
                pad_l: Some(6.0),
                pad_r: Some(8.0),
                border_radius_tl: Some(2.0),
                border_radius_tr: Some(2.0),
                border_radius_bl: Some(2.0),
                border_radius_br: Some(2.0),
                text: entry.name.clone(),
                text_height: Some(12.5),
                text_color: Some(Color::srgb_hex("f8f8f8ff")),
                text_wrap: Some(ImtTextWrap::None),
                ..BinStyle::default()
            });

            y += 25.0;

            if y >= 420.0 {
                x += 151.0;
                y = 3.0;
            }

            menu_entry.add_hooks(self.basalt.clone());
            search_entries.push(Arc::new(menu_entry));
        }
    }
}
