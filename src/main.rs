use structopt::StructOpt;
extern crate dirs;
use serde_json::json;
use std::fs;
extern crate colored;
use colored::*;
mod cli;
mod platform;


#[derive(StructOpt)]
struct Cli {
    pattern: Option<String>,

    project: Option<String>,

    #[structopt(short = "i", long = "ui", default_value = "template")]
    ui: String,

    #[structopt(short = "t", long = "target", default_value = "node")]
    target: String,
}

fn main() {
    let settings_dir: String = format!(
        "{}/.pyra/settings.json",
        dirs::home_dir().unwrap().display()
    );

    let default = json!({
        "commandToOpen": "code",
        "projects": []
    });

    // Check whether setting file exists
    if !cli::path_exists(settings_dir.clone()) {
        println!(
            "Generating new settings file at {}...",
            settings_dir.clone()
        );
        match fs::create_dir_all(format!(
            "{}/.pyra",
            dirs::home_dir().unwrap().display()
        )) {
            Ok(_) => (),
            Err(why) => panic!("Failed to create dir: {}", why),
        }
        cli::save_settings(default.clone());
    }

    let file_string = match fs::read_to_string(settings_dir.clone()) {
        Err(why) => {
            panic!("Setting file error at {}: {}", settings_dir.red(), why);
        }
        Ok(file) => file,
    };

    let settings_data = serde_json::from_str(&file_string).unwrap();
    let args = Cli::from_args();
    match args.pattern {
        None => cli::open_project(settings_data, args.project),
        Some(ref x) if x == "open" => cli::open_project(settings_data, args.project),
        Some(ref x) if x == "init" => cli::init_project(settings_data),
        Some(ref x) if x == "add" || x == "save" => cli::add_project(settings_data, None, None),
        Some(ref x) if x == "remove" => cli::remove_project(settings_data),
        Some(ref x) if x == "seteditor" => cli::set_editor(settings_data),
        Some(ref x) if x == "run" => cli::run_project(settings_data),
        Some(ref x) if x == "build" => cli::build_project(settings_data, args.target),
        Some(ref x) if x == "test" => cli::test_project(settings_data, args.target),
        Some(ref x) if x == "interact" => cli::run_substrate_ui(settings_data, Some(args.ui)),
        Some(ref x) if x == "purge" => cli::purge_chain(settings_data),
        Some(ref _x) => {
            println!("{}", format!("Command '{}' not found", _x).red());
            cli::help()
        }
    }
}

