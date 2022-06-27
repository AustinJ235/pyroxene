use crate::desktop::DesktopEntry;
use std::sync::Arc;

#[derive(Debug)]
pub struct Category {
	pub iden: String,
	pub name: String,
	pub icon: String,
	pub entries: Vec<Arc<DesktopEntry>>,
}

impl Category {
	pub fn new<Id: Into<String>, N: Into<String>, Ic: Into<String>>(iden: Id, name: N, icon: Ic) -> Self {
		Category {
			iden: iden.into(),
			name: name.into(),
			icon: icon.into(),
			entries: Vec::new(),
		}
	}

	pub fn add_entries(&mut self, entries: &Vec<Arc<DesktopEntry>>) {
		for entry in entries {
			if entry.categories.iter().any(|c| c.to_lowercase() == self.iden.to_lowercase()) {
				self.entries.push(entry.clone());
			}
		}

		self.entries.sort_by_key(|e| e.name.to_lowercase());
	}
}
