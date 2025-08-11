// SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
// SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
// SPDX-License-Identifier: BSD-3-Clause

use std::{
    env,
    ffi::OsString,
    path::{Component, Path, PathBuf},
};

use crate::{ShellCore, utils::file_check};

pub fn oss_to_name(oss: &OsString) -> String {
    oss.to_string_lossy().to_string()
}

pub fn buf_to_name(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}

pub fn search_command(command: &str) -> Option<String> {
    let paths = env::var_os("PATH");
    if paths.is_none() {
        return None;
    }

    for path in env::split_paths(&paths.unwrap()) {
        let compath = buf_to_name(&path) + "/" + command;
        if file_check::exists(&compath) {
            return Some(compath);
        }
    }

    None
}

pub fn make_absolute_path(core: &mut ShellCore, path_str: &str) -> PathBuf {
    let path = Path::new(&path_str);
    let mut absolute = PathBuf::new();
    if !path.is_relative() {
        absolute.push(path);
        return absolute;
    }

    if path.starts_with("~") {
        // tilde -> $HOME
        let home_dir = core.db.get_param("HOME").unwrap_or(String::new());
        if home_dir != "" {
            absolute.push(PathBuf::from(home_dir));
            let num = match path_str.len() > 1 && path_str.starts_with("~/") {
                true => 2,
                false => 1,
            };
            absolute.push(PathBuf::from(&path_str[num..]));
        }
    } else {
        // current
        if let Some(tcwd) = core.get_current_directory() {
            absolute.push(tcwd);
            absolute.push(path);
        };
    }

    absolute
}

pub fn make_canonical_path(core: &mut ShellCore, path_str: &str) -> PathBuf {
    let path = make_absolute_path(core, path_str);
    let mut canonical = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir => canonical.push(Component::RootDir),
            Component::ParentDir => {
                canonical.pop();
            },
            Component::Normal(c) => canonical.push(c),
            _ => (),
        }
    }
    canonical
}
