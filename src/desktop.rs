use std::{fs::File, io, io::Read, path::Path};

#[derive(Clone, Debug)]
pub struct DesktopEntry {
    pub name: String,
    pub icon: String,
    pub ty: String,
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
        let mut ty = None;
        let mut categories = Vec::new();
        let mut keywords = Vec::new();

        for line in buffer.lines() {
            if line.starts_with("Name=") {
                if name.is_none() {
                    name = Some(line[5..].to_string());
                }
            } else if line.starts_with("Comment=") {
                if comment.is_none() {
                    comment = Some(line[8..].to_string());
                }
            } else if line.starts_with("Exec=") {
                if exec.is_none() {
                    exec = Some(line[5..].to_string());
                }
            } else if line.starts_with("Categories=") {
                if categories.is_empty() {
                    for category in line[11..].split(";") {
                        if !category.is_empty() {
                            categories.push(category.to_owned());
                        }
                    }
                }
            } else if line.starts_with("Keywords=") {
                if keywords.is_empty() {
                    for keyword in line[9..].split(";") {
                        if !keyword.is_empty() {
                            keywords.push(keyword.to_owned());
                        }
                    }
                }
            } else if line.starts_with("Icon=") {
                if icon.is_none() {
                    icon = Some(line[5..].to_string());
                }
            } else if line.starts_with("Type=") {
                if ty.is_none() {
                    ty = Some(line[5..].to_string());
                }
            }
        }

        Ok(DesktopEntry {
            name: name.ok_or(io::Error::new(io::ErrorKind::Other, "Missing Name"))?,
            icon: icon.unwrap_or(String::new()),
            ty: ty.unwrap_or(String::from("Application")),
            comment: comment.unwrap_or(String::new()),
            exec: exec.ok_or(io::Error::new(io::ErrorKind::Other, "Missing Exec"))?,
            categories,
            keywords,
        })
    }
}
