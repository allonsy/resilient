
const DELETING_PATTERN: &str = "*deleting";

pub enum Change {
    Deleting(String),
    Modification(Modification)
}

impl Change {
    pub fn get_mod_string(&self) -> String {
        match self {
            Change::Deleting(fname) => {
                format!("Deleting: {}", fname)
            },
            Change::Modification(modification) => {
                let fname = modification.file_name.clone();
                match &modification.modifiers {
                    Modifiers::Creation => {
                        match &modification.file_type {
                            FileType::Symlink(dest) => {
                                format!("Creating (link): {} -> {}", fname, dest)
                            },
                            _ => {
                                format!("Creating: {}", fname)
                            }
                        }
                    },
                    Modifiers::Update(modlist) => {
                        if modlist.checksum && modification.file_type.is_symlink() {
                            format!("Updating (link): {} -> {}", fname, modification.file_type.get_symlink_dest())
                        } else if modlist.mod_time && !modlist.size {
                            format!("Updating timestamps: {}", fname)
                        } else if modlist.owner && !modlist.size {
                            format!("Updating owner: {}", fname)
                        } else if modlist.group && !modlist.size {
                            format!("Updating group: {}", fname)
                        } else if modlist.perms && !modlist.size {
                            format!("Updating perms: {}", fname)
                        } else if modlist.acl && !modlist.size {
                            format!("Updating acls: {}", fname)
                        } else if modlist.xattrs && !modlist.size {
                            format!("Updating xattrs: {}", fname)
                        } else {
                            format!("Updating: {}", fname)
                        }
                    }
                }
            }
        }
    }
}

struct Modification {
    file_name: String,
    update_type: UpdateType,
    file_type: FileType,
    modifiers: Modifiers,
}

enum UpdateType {
    Transfer,
    Creation,
}

enum FileType {
    File,
    Directory,
    Symlink(String),
    Device,
    Special,
}

impl FileType {
    fn is_symlink(&self) -> bool {
        match self {
            FileType::Symlink(_) => true,
            _ =>  false
        }
    }

    fn get_symlink_dest(&self) -> &str {
        match self {
            FileType::Symlink(dest) => &dest,
            _ => panic!("File isn't a symlink")
        }
    }
}

enum Modifiers {
    Creation,
    Update(ModList)
}

struct ModList {
    checksum: bool,
    size: bool,
    mod_time: bool,
    perms: bool,
    owner: bool,
    group: bool,
    acl: bool,
    xattrs: bool,
}

pub fn parse_change(line: &str) -> Option<Change> {
    let trimmed_line = line.trim();

    if trimmed_line.starts_with(DELETING_PATTERN) {
        let tokens: Vec<&str> = trimmed_line.split_whitespace().collect();
        if tokens.len() < 2 {
            return None;
        }
        return Some(Change::Deleting(tokens[1].to_string()));
    }

    let chars: Vec<char> = trimmed_line.chars().collect();
    if chars.is_empty() {
        return None;
    }
    let update_type_char = chars[0];
    
    let update_type = match update_type_char {
        '>' => {
            UpdateType::Transfer
        },
        '.' => {
            UpdateType::Transfer
        },
        'c' => {
            UpdateType::Creation
        },

        _ => {
            return None;
        }
    };

    if chars.len() < 2 {
        return None;
    }
    let file_type = match chars[1] {
        'f' => FileType::File,
        'd' => FileType::Directory,
        'D' => FileType::Device,
        'S' => FileType::Special,
        'L' => {
            let arrow_split = trimmed_line.split("->").collect::<Vec<&str>>();
            if arrow_split.len() < 2 {
                return None;
            }
            FileType::Symlink(arrow_split[1].trim().to_string())
        },
        _ => {
            return None;
        }
    };

    if chars.len() < 3 {
        return None;
    }
    let modifiers = match chars[2] {
        '+' => {
            Modifiers::Creation
        },
        _ => {
            let modlist = parse_modifiers(&chars[2..]);
            if modlist.is_none() {
                return None;
            }
            Modifiers::Update(modlist.unwrap())
        }
    };

    let tokens: Vec<&str> = trimmed_line.split_whitespace().collect();
    if tokens.len() < 2 {
        return None;
    }

    let file_name = tokens[1].to_string();

    Some(Change::Modification(Modification {
        file_name,
        update_type,
        file_type,
        modifiers
    }))
}

fn parse_modifiers(chars: &[char]) -> Option<ModList> {
    if chars.len() < 9 {
        return None;
    }

    let checksum = chars[0] != '.';
    let size = chars[1] != '.';
    let mod_time = chars[2] != '.';
    let perms = chars[3] != '.';
    let owner = chars[4] != '.';
    let group = chars[5] != '.';
    let acl = chars[7] != '.';
    let xattrs = chars[8] != '.';

    Some(ModList {
        checksum,
        size,
        mod_time,
        perms,
        owner,
        group,
        acl,
        xattrs
    })
}

