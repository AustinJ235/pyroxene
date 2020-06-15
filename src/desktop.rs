use std::{fs::File, io, io::Read, path::Path};

#[derive(Clone, Debug)]
pub struct DesktopEntry {
    pub name: String,
    pub icon: Option<String>,
    pub generic_name: Option<String>,
    pub comment: Option<String>,
    pub exec: String,
    pub path: Option<String>,
    pub terminal: bool,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug)]
pub enum DesktopEntryErr {
    InvalidGroupHeader,
    NotApplication,
    Hidden,
    OnlyShowIn,
    NotShowIn,
    MissingName,
    MissingExec,
    IO(io::Error),
}

impl DesktopEntry {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, DesktopEntryErr> {
        let mut handle = File::open(path).map_err(|e| DesktopEntryErr::IO(e))?;
        let mut buffer = String::new();
        handle.read_to_string(&mut buffer).map_err(|e| DesktopEntryErr::IO(e))?;

        let mut name: Option<String> = None; // R
        let mut icon: Option<String> = None;
        let mut generic_name: Option<String> = None;
        let mut comment: Option<String> = None;
        let mut exec: Option<String> = None; // R
        let mut path: Option<String> = None;
        let mut terminal: Option<bool> = None;
        let mut categories: Vec<String> = Vec::new();
        let mut keywords: Vec<String> = Vec::new();
        let mut current_group: Option<String> = None;

        for line in buffer.lines() {
            if line.starts_with("[") {
                if line.len() < 3 {
                    return Err(DesktopEntryErr::InvalidGroupHeader);
                }

                current_group = Some(line[1..(line.len() - 1)].to_owned());
                continue;
            }

            if current_group.is_some()
                && current_group.as_ref().unwrap().as_str() == "Desktop Entry"
            {
                if line.starts_with("Type=") {
                    if !line.ends_with("Application") {
                        return Err(DesktopEntryErr::NotApplication);
                    }
                } else if line.starts_with("Name=") {
                    name = Some(line[5..].to_owned());
                } else if line.starts_with("Icon=") {
                    icon = Some(line[5..].to_owned());
                } else if line.starts_with("GenericName=") {
                    generic_name = Some(line[12..].to_owned());
                } else if line.starts_with("Comment=") {
                    comment = Some(line[8..].to_owned());
                } else if line.starts_with("Exec=") {
                    exec = Some(line[5..].to_owned());
                } else if line.starts_with("Path=") {
                    path = Some(line[5..].to_owned());
                } else if line.starts_with("Terminal") {
                    if line.ends_with("true") {
                        terminal = Some(true);
                    } else {
                        terminal = Some(false);
                    }
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
                } else if line.starts_with("Hidden=") {
                    if line.ends_with("true") {
                        return Err(DesktopEntryErr::Hidden);
                    }
                } else if line.starts_with("OnlyShowIn=") {
                    if !line[11..].split(";").map(|v| v.to_owned()).any(|i| i == "Sway") {
                        return Err(DesktopEntryErr::OnlyShowIn);
                    }
                } else if line.starts_with("NotShowIn=") {
                    if line[10..].split(";").map(|v| v.to_owned()).any(|i| i == "Sway") {
                        return Err(DesktopEntryErr::NotShowIn);
                    }
                }
            }
        }

        Ok(DesktopEntry {
            name: name.ok_or(DesktopEntryErr::MissingName)?,
            icon,
            generic_name,
            comment,
            exec: exec.ok_or(DesktopEntryErr::MissingExec)?,
            path,
            terminal: terminal.unwrap_or(false),
            categories,
            keywords,
        })
    }
}
