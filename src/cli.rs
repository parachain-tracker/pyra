use std::env;
extern crate dirs;
use dialoguer::{
    theme::ColorfulTheme, theme::CustomPromptCharacterTheme, Confirmation, Input, Select, Checkboxes
};
use serde_json::json;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process;
use std::process::Command;
extern crate colored;
use colored::*;
use serde::{Deserialize, Serialize};
use webbrowser;
use crate::platform::substrate;

extern crate ctrlc;
use log::{debug, warn};

#[derive(Serialize, Deserialize, Debug)]
struct Project {
    name: String,
    path: String,
    platform: String,
    editor: String,
}

static PLATFORMS: [&'static str; 1] = ["substrate"];

pub fn list_projects(settings_data: serde_json::value::Value) -> Vec<String> {
    let mut selections = vec![];
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let selection = settings_data["projects"][i]["name"]
            .as_str()
            .unwrap()
            .to_string();
        selections.push(selection.clone());
    }
    selections
}

pub fn browse(prompt: &str, settings_data: serde_json::value::Value) -> String {
    let selections = list_projects(settings_data.clone());
    if selections.len() == 0 {
        println!("{}", format!("Project does not exist :( Add it using {} or cd till current working project and type {}",
     "`pyra init`".yellow(), 
     "`pyra add`".yellow()).red().bold());
        panic!("No project found");
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(0)
        .items(&selections[..])
        .interact()
        .unwrap();
    let result = &selections[selection.clone()];

    result.to_string()
}

pub fn checklist(prompt: &str, settings_data: serde_json::value::Value) -> Vec<usize> {
    let selections = list_projects(settings_data.clone());
    if selections.len() == 0 {
        println!("{}", format!("Project does not exist :( Add it using {} or cd till the project folder and type {}",
     "`pyra add [projectPath]`".yellow(), 
     "`pyra add`".yellow()).red().bold());
        panic!("No project found");
    }
    let mut selections_ptr: Vec<&str> = [].to_vec();
    for i in 0..selections.len() {
        selections_ptr.push(&selections[i]);
    } 

    let results = Checkboxes::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt.clone())
        .items(&selections_ptr[..])
        .interact()
        .unwrap();

    if results.is_empty() {
        println!("You did not select anything :(");
        checklist(prompt.clone(), settings_data);
    } 
    results
}


pub fn open_project(settings_data: serde_json::value::Value, project: Option<String>) {
    let open_prompt: &str = &prompt("Select project to open");
    match project {
        // if input is none give selections
        None => {
            let result = browse(open_prompt, settings_data.clone());
            open_project(settings_data, Some(result))
        }
        // if input is in the list, open it
        Some(ref x) if project_exists(x.clone(), settings_data.clone()) => {
            let editor = find_project_editor(x.clone(), settings_data.clone());
            if editor != "default" {
                let command = editor;
                let path = find_project_path(project.clone().unwrap(), settings_data.clone());
                open_editor(command, path);
                return;
            } else {
                let command = settings_data["commandToOpen"].as_str().unwrap();
                let path = find_project_path(project.clone().unwrap(), settings_data.clone());
                println!(">>> Opening {}...", x.green());
                open_editor(command.to_string(), path);
            }
        }
        // if the input is not in the list, call support
        Some(ref _x) => {
            let command = settings_data["commandToOpen"].as_str().unwrap();
            println!(
                "{}\n{}\n{}",
                "Could not open project :(".red().bold(),
                format!(
                    "Are you sure your editor uses command `{}` to open directories from terminal?",
                    command.yellow().bold()
                ),
                format!(
                    "If not, use {} to set Editor/IDE of your choice",
                    "`pyra seteditor`".yellow().bold()
                )
            );
        }
    }
}

pub fn init_project(settings_data: serde_json::value::Value) {
    let theme = CustomPromptCharacterTheme::new(':');
    let project_name: String = Input::with_theme(&theme)
        .with_prompt(&prompt(&bold("The Name of the blockchain")))
        .interact()
        .unwrap();
    /* TODO: leave author for making package.json
    let author: String = Input::with_theme(&theme)
        .with_prompt(&prompt(&bold("The Author")))
        .interact()
        .unwrap();
    "*/
    let prompt = prompt("Blockchain platform to develop");

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(&prompt)
        .default(0)
        .items(&PLATFORMS)
        .interact()
        .unwrap();
    let platform_name = PLATFORMS[selection].to_string();
    add_project(
        settings_data.clone(),
        Some(project_name.clone()),
        Some(platform_name.clone()),
    );
    let project_dir = format!("{}/{}", env::current_dir().unwrap().display(), project_name);

    println!("Generating project_directory at {}...", project_dir.clone());

    match &platform_name[..] {
        "substrate" => {
            substrate::init_substrate(project_dir, project_name)
            },
        _ => panic!("Not implemented yet")
        // TODO: Add platform command functions for other parchain projects
    }
}

pub fn run_project(settings_data: serde_json::value::Value) {
    let run_prompt = &prompt("Which Substrate node would you like to operate?");
    let project_name = browse(run_prompt, settings_data.clone());
    let path = find_project_path(project_name.clone(), settings_data.clone());
    let platform_name = find_project_platform(project_name.clone(), settings_data);
    
    match &platform_name[..] {
        "substrate" => substrate::run_substrate(project_name, path),
        _ => panic!("Not implemented yet")
        // TODO: Add platform command functions for other parchain projects
    }
}

pub fn purge_chain(settings_data: serde_json::value::Value) {
    let purge_prompt = &prompt("Which Substrate node would you like to purge and restart?");
    let project_name = browse(purge_prompt, settings_data.clone());
    let path = find_project_path(project_name.clone(), settings_data.clone());
    let platform_name = find_project_platform(project_name.clone(), settings_data);

    match &platform_name[..] {
        "substrate" => substrate::purge_substrate(project_name, path),
        _ => panic!("Not implemented yet")
        // TODO: Add platform command functions for other parchain projects
    }
}

pub fn build_project(settings_data: serde_json::value::Value, target: String) {
    let substrate_prompt = &prompt("Which Substrate node would you like to build?");
    let project_name = browse(substrate_prompt, settings_data.clone());
    let path = find_project_path(project_name.clone(), settings_data.clone());
    let platform_name = find_project_platform(project_name.clone(), settings_data);
    
    match &platform_name[..] {
        "substrate" => substrate::build_substrate(project_name, path, target),
        _ => panic!("Not implemented yet")
        // TODO: Add platform command functions for other parchain projects
    }
}

pub fn test_project(settings_data: serde_json::value::Value, target: String) {
    let substrate_prompt = &prompt("Which Substrate node would you like to test?");
    let project_name = browse(substrate_prompt, settings_data.clone());
    let path = find_project_path(project_name.clone(), settings_data.clone());
    let platform_name = find_project_platform(project_name.clone(), settings_data);
    
    match &platform_name[..] {
        "substrate" => substrate::test_substrate(project_name, path, target),
        _ => panic!("Not implemented yet")
        // TODO: Add platform command functions for other parchain projects
    }
}


pub fn run_substrate_ui(settings_data: serde_json::value::Value, ui: Option<String>) {
    let substrate_prompt = &prompt("Which Substrate node would you like to interact?");
    let project_name = browse(substrate_prompt, settings_data.clone());
    let path = find_project_path(project_name.clone(), settings_data);
    let substrate_ui_path = format!("{}/{}-ui", path, project_name.clone());
    let substrate_app_path = format!("{}/apps", path);

    match ui {
        None => panic!("Should not happen"),
        Some(ref x) if x == "gav" => {
            match env::set_current_dir(&substrate_ui_path) {
                Ok(_) => (),
                Err(why) => panic!("Failed to set current dir: {}", why),
            }
            
            ctrlc::set_handler(move || {
                Command::new("yarn")
                .args(&["run", "dev"])
                .spawn()
                .expect("Failed to run substrate ui");
            }).expect("Error setting Ctrl-C handler");
            
            match webbrowser::open("http://localhost:8000") {
                Ok(_) => (),
                Err(why) => panic!("Failed to open webbrowser: {}", why),
            }
        }
        Some(ref x) if x == "apps" => {
            match env::set_current_dir(&substrate_app_path) {
                Ok(_) => (),
                Err(why) => panic!("Failed to set current dir: {}", why),
            }

            ctrlc::set_handler(move || {
                Command::new("yarn")
                .spawn()
                .expect("Failed to run yarn");
                Command::new("yarn")
                    .args(&["run", "start"])
                    .spawn()
                    .expect("Failed to start yarn app");
            }).expect("Error setting Ctrl-C handler");
            
            match webbrowser::open("http://localhost:3000") {
                Ok(_) => (),
                Err(why) => panic!("Failed to open webbrowser: {}", why),
            }
        },
        Some(ref _x) => panic!("default value not set")
    }
}

// TODO: run Polkascan app
pub fn run_scanner(settings_data: serde_json::value::Value) {}

pub fn add_project(
    settings_data: serde_json::value::Value,
    project: Option<String>,
    platform: Option<String>,
) {
    let suggestion = env::current_dir()
        .unwrap()
        .display()
        .to_string()
        .split("/")
        .last()
        .unwrap()
        .to_string();
    let theme = CustomPromptCharacterTheme::new(':');
    let project_name = match project {
        None => Input::with_theme(&theme)
            .with_prompt("Project Name \u{2692}")
            .allow_empty(true)
            .default(suggestion)
            .interact()
            .unwrap(),
        Some(x) => x.to_string(),
    };
    let platform_name = match platform {
        None => {
            let prompt = prompt("Blockchain platform to develop");
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt(&prompt)
                .default(0)
                .items(&PLATFORMS)
                .interact()
                .unwrap();
            PLATFORMS[selection].to_string()
        }
        Some(x) => x.to_string(),
    };
    let mut next_settings = settings_data.clone();
    let path = env::current_dir();
    // Check whether the project already exists
    if project_exists(project_name.clone(), next_settings.clone()) {
        println!(
            "{}",
            format!("{}", "Project with this name already exists".red().bold())
        );
        panic!();
    }

    let new_project: Project = Project {
        name: project_name.clone(),
        path: format!("{}/{}",path.unwrap().display(),project_name.clone()),
        platform: platform_name,
        editor: settings_data["commandToOpen"].as_str().unwrap().to_string(),
    };
    let p = serde_json::to_value(new_project).unwrap();
    next_settings["projects"].as_array_mut().unwrap().push(p);

    // Save next settings file
    println!(
        "{}",
        format!(
            "Project {} is successfully registered in project registry",
            project_name.cyan().bold()
        )
        .green()
    );
    save_settings(next_settings);
}

pub fn save_settings(settings_data: serde_json::value::Value) {
    let settings_dir: String = format!(
        "{}/.pyra/settings.json",
        dirs::home_dir().unwrap().display()
    );
    let f = serde_json::to_string(&settings_data).unwrap();
    let mut file = File::create(&settings_dir).expect("Unable to write");
    file.write_all(f.as_bytes())
        .expect("Cannot write to a file");
}

pub fn remove_project(settings_data: serde_json::value::Value) {
    let mut next_settings = settings_data.clone();
    let remove_prompt: &str = &prompt("Select projects to remove. Press SPACE to check the projects");
    let results = checklist(remove_prompt.clone(), settings_data.clone());
    let selections = list_projects(settings_data.clone());
    
    for i in results {
        let result = &selections[i];
        let path = find_project_path(result.clone().to_string(), settings_data.clone());
        // Remove the project in json file
        next_settings = delete_project_json(next_settings, result.to_string());
        // Remove it in the disk
        let mut p = Command::new("rm")
            .args(&["-rf", &path])
            .stderr(process::Stdio::piped())
            .spawn()
            .unwrap();
        p.wait().unwrap();
        println!(
            "{}",
            format!("Project {} has been successfully removed", result.cyan().bold()).green()
        );
    }
    // Save the next settings
    save_settings(next_settings);
}

pub fn delete_project_json(
    mut settings_data: serde_json::value::Value,
    project: String,
) -> serde_json::value::Value {
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let selection = settings_data["projects"][i]["name"]
            .as_str()
            .unwrap()
            .to_string();
        if selection == project {
            settings_data["projects"].as_array_mut().unwrap().remove(i);
            return settings_data;
        }
    }
    panic!("The project to remove does not exist in the settings file".red());
}

pub fn set_editor(settings_data: serde_json::value::Value) {
    let mut next_settings = settings_data.clone();

    let seteditor_prompt: &str = &prompt("Select project to set editor");
    let result = browse(seteditor_prompt, settings_data);

    let theme = CustomPromptCharacterTheme::new('>');

    let input: String = Input::with_theme(&theme)
        .with_prompt("The command to open your editor")
        .interact()
        .unwrap();

    // Set editor for the project in json file
    next_settings = seteditor_project_json(next_settings, result.to_string(), input);
    println!("{}", "Editor is successfully updated".green());
    save_settings(next_settings);
}

pub fn seteditor_project_json(
    mut settings_data: serde_json::value::Value,
    project: String,
    editor: String,
) -> serde_json::value::Value {
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let selection = settings_data["projects"][i]["name"]
            .as_str()
            .unwrap()
            .to_string();
        if selection == project {
            *settings_data["projects"][i].get_mut("editor").unwrap() = json!(editor);
            return settings_data;
        }
    }
    return settings_data;
}

pub fn help() {
    print!(
        "\nUsage: pyra <command>

Options:
  -V, --version                output the version number
  -h, --help                   output usage information
  -t, --target                 component to compile
  -i, -ui                      user interface to open 
  --dev                        run in dev mode

Commands:
  init                         Initialize Substrate dev environment 
  open                         Open one of your saved projects
  add | save                   Add/Save current directory as a project
  remove                       Remove the project in the registry and project files
  seteditor                    Set text editor to use for a project
  run                          Run built Substrate node binary
  deploy                       Deploy Substrate nodes in local/cloud environment
  interact                     Open Substrate UI to interact with the node
  publish                      Publish runtime or node in hosting website\n"
    )
}

pub fn find_project_path(name: String, settings_data: serde_json::value::Value) -> String {
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let project = settings_data["projects"][i]["name"].as_str().unwrap();
        let path = settings_data["projects"][i]["path"].as_str().unwrap();
        if project == name {
            return path.to_string();
        }
    }
    panic!("setting file is broken".red());
}

pub fn find_project_platform(name: String, settings_data: serde_json::value::Value) -> String {
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let project = settings_data["projects"][i]["name"].as_str().unwrap();
        let platform = settings_data["projects"][i]["platform"].as_str().unwrap();
        if project == name {
            return platform.to_string();
        }
    }
    panic!("setting file is broken".red());
}

pub fn find_project_editor(name: String, settings_data: serde_json::value::Value) -> String {
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let project = settings_data["projects"][i]["name"].as_str().unwrap();
        let editor = settings_data["projects"][i]["editor"].as_str().unwrap();
        if project == name {
            return editor.to_string();
        }
    }
    return "default".to_string();
}

pub fn project_exists(prop: String, settings_data: serde_json::value::Value) -> bool {
    for i in 0..settings_data["projects"].as_array().unwrap().len() {
        let project = settings_data["projects"][i]["name"].as_str().unwrap();
        if project == prop {
            return true;
        }
    }
    false
}

pub fn open_editor(command: String, path: String) {
    Command::new(&command)
        .arg(&path)
        .spawn()
        .expect("Failed to process editor command");
}

pub fn path_exists(path: String) -> bool {
    match fs::metadata(&path) {
        Ok(_some) => true,
        Err(_) => false,
    }
}

fn prompt(question: &str) -> String {
    format!("{} {}", "?".green(), question)
}

fn bold(text: &str) -> String {
    format!("{}", text.bold())
}