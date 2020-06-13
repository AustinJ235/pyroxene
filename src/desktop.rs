use std::io::Read;
use std::fs::File;
use std::io;
use std::path::Path;

#[derive(Clone,Debug)]
pub struct DesktopEntry {
    pub name: String,
    pub icon: String,
    pub comment: String,
    pub exec: String,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
}

impl DesktopEntry {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut handle = File::open(path)?;
        let mut buffer = String::new();
        handle.read_to_string(&mut buffer)?;

        let mut name = None;
        let mut icon = None;
        let mut comment = None;
        let mut exec = None;
        let mut categories = Vec::new();
        let mut keywords = Vec::new();

        for line in buffer.lines() {
            if line.starts_with("Name=") {
                name = Some(line[5..].to_string());
            } else if line.starts_with("Comment=") {
                comment = Some(line[8..].to_string());
            } else if line.starts_with("Exec=") {
                exec = Some(line[5..].to_string());
            } else if line.starts_with("Categories=") {
                for category in line[11..].split(";") {
                    if !category.is_empty() {
                        categories.push(category.to_owned());
                    }
                }
            } else if line.starts_with("Keywords=") {
                for keyword in line[9..].split(";") {
                    if !keyword.is_empty() {
                        keywords.push(keyword.to_owned());
                    }
                }
            } else if line.starts_with("Icon=") {
                icon = Some(line[5..].to_string());
            }
        }

        Ok(DesktopEntry {
            name: name.ok_or(io::Error::new(io::ErrorKind::Other, "Missing Name"))?,
            icon: icon.unwrap_or(String::new()),
            comment: comment.unwrap_or(String::new()),
            exec: exec.ok_or(io::Error::new(io::ErrorKind::Other, "Missing Exec"))?,
            categories,
            keywords,
        })
    }
}
