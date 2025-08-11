// SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
// SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, utils::arg};

fn print_all(core: &mut ShellCore) -> i32 {
    println!("hits	command");

    for com in core.db.get_indexes_all("BASH_CMDS") {
        if let Ok(path) = core.db.get_elem("BASH_CMDS", &com) {
            if let Some(n) = core.db.hash_counter.get(&com) {
                println!("{:4}\t{}", &n, &path);
            }
        }
    }

    0
}

pub fn hash(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut args = arg::dissolve_options(args);

    if args.len() == 1 {
        return print_all(core);
    }

    if arg::consume_option("-p", &mut args) {
        if args.len() == 1 {
            return super::error_exit(1, "hash", "-p: option requires an argument", core);
        }
        if args.len() == 2 {
            return super::error_exit(1, "hash", "still not implemented", core);
        }

        if let Err(e) = core.db.set_assoc_elem("BASH_CMDS", &args[2], &args[1], None) {
            let msg = String::from(&e);
            return super::error_exit(1, "hash", &msg, core);
        }
    }
    0
}
