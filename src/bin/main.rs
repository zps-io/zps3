/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2020 Zachary Schneider
 */

use clap::{App, Arg, AppSettings};
use zps::app::ZPS;
use zps::{Emitter, console};
use zps::console::UI;

fn main() {
    let matches = App::new("ZPS")
        .version("1.0")
        .author("Zachary Schneider <sigil.66@gmail.com>")
        .about("The last word in package management")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::new("tree")
            .short('t')
            .long("tree")
            .value_name("TREE")
            .about("Override path to ZPS tree")
            .takes_value(true))
        .subcommand(App::new("env")
            .about("dumps ZPS environment"))
        .get_matches();

    let mut zps = ZPS::new(matches.value_of("tree"));

    UI::bind(&mut zps, true);

    match matches.subcommand_name() {
        Some("env") => {
            for (k, v) in zps.env() {
                println!("{}: {}", k, v)
            }
        },
        None => (),
        _ => println!("Command not found"),
    }
}
