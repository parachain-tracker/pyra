use std::env;
extern crate dirs;
use dialoguer::{theme::ColorfulTheme, theme::CustomPromptCharacterTheme, Confirmation, Input, Select};
use serde_json::json;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::process::Command;
extern crate colored;
use serde::{Deserialize, Serialize};
use colored::*;
use webbrowser;

#[derive(Serialize, Deserialize, Debug)]
struct Project {
    name: String,
    path: String,
    editor: String,
}

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
        println!("{}", format!("Project does not exist :( Add it using {} or cd till the project folder and type {}",
     "`pyra add [projectPath]`".yellow(), 
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

pub fn open_project(settings_data: serde_json::value::Value, project: Option<String>) {
    let open_prompt: &str = &format!("{} Select project to open", "?".green());
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
            }
            let command = settings_data["commandToOpen"].as_str().unwrap();
            let path = find_project_path(project.clone().unwrap(), settings_data.clone());
            println!(">>> Opening {}...", x.green());
            open_editor(command.to_string(), path);
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
        .with_prompt(&"The name of the parachain".magenta().bold())
        .interact()
        .unwrap();
    let node_name = format!("{}-node", project_name);
    let author: String = Input::with_theme(&theme)
        .with_prompt(&"The name of the author".magenta().bold())
        .interact()
        .unwrap();
        
    if Confirmation::new().with_text("Do you want to add current directory as a project?").interact().unwrap() {
        add_project(settings_data);
    }

    pause("The installation will take long which is roughly about 15 minutes \u{1f391}.\nPress any key to continue and wait with \u{2615} or Ctrl + C to quit...".to_string());

    Command::new("bash")
        .args(&[&format!("{}/.cargo/bin/substrate-node-new", dirs::home_dir().unwrap().display()),&node_name, &author])
        .spawn()
        .expect("Failed to process substrate command");

    Command::new("bash")
        .args(&[&format!("{}/.cargo/bin/substrate-ui-new", dirs::home_dir().unwrap().display()),&project_name])
        .spawn()
        .expect("Failed to process substrate command");  
}

pub fn run_substrate(settings_data: serde_json::value::Value) {
    let substrate_prompt = "Which Substrate node would you like to operate?";
    let project_name = browse(substrate_prompt, settings_data.clone());
    let path = find_project_path(project_name.clone(), settings_data);
    
    let substrate_bin_path = format!("{}/{}-node/target/release/{}-node", path, project_name.clone(), project_name.clone());
    println!("{:?}", substrate_bin_path);
    let command = Command::new(&substrate_bin_path)
        .arg("--dev")
        .spawn()
        .expect("Failed to run substrate binary");
    let pid = command.id().to_string().green().bold();
    format!("Substrate daemon running at pid {}. kill the process with `kill {}` command", pid, pid).magenta().bold().to_string();
}

pub fn build_substrate(settings_data: serde_json::value::Value) {
    let substrate_prompt = "Which Substrate node would you like to build?";
    let project_name = browse(substrate_prompt, settings_data.clone());
    let path = find_project_path(project_name.clone(), settings_data);
    
    let substrate_build_path = format!("{}/{}-node/Cargo.toml", path, project_name.clone());
    println!("{:?}", substrate_build_path);
    let command = Command::new("cargo")
        .args(&["build".to_string(), "--release".to_string(), format!("--manifest-path={}", substrate_build_path)])
        .spawn()
        .expect("Failed to run substrate binary");
    let pid = command.id().to_string().green().bold();
    format!("Substrate daemon running at pid {}. kill the process with `kill {}` command", pid, pid).magenta().bold().to_string();
}

pub fn run_substrate_ui(settings_data: serde_json::value::Value) {
    let substrate_prompt = "Which Substrate node would you like to interact?";
    let project_name = browse(substrate_prompt, settings_data.clone());
    let path = find_project_path(project_name.clone(), settings_data);
    
    let substrate_ui_path = format!("{}/{}-ui", path, project_name.clone());
    env::set_current_dir(&substrate_ui_path);
    println!("{:?}", &substrate_ui_path);
    reg_for_sigs();
    let command = Command::new("yarn")
        .args(&["run".to_string(), "dev".to_string()])
        .spawn()
        .expect("Failed to run substrate ui");
    webbrowser::open("http://localhost:8000");  
}

pub fn pause(note: String) {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "{}", note).unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}


pub fn add_project(settings_data: serde_json::value::Value) {
    let mut next_settings = settings_data.clone();
    let path = env::current_dir();
    let hint = env::current_dir().unwrap().display().to_string().split("/").last().unwrap().to_string();
    let theme = CustomPromptCharacterTheme::new(':');
    let project_name: String = Input::with_theme(&theme)
        .with_prompt("Project Name \u{2692}")
        .allow_empty(true)
        .default(hint)
        .interact()
        .unwrap();
    let result = project_name.clone();

    // Check whether the project already exists
    if project_exists(result.clone(), next_settings.clone()) {
        println!("{}", format!("{}", "Project with this name already exists".red().bold()));
        panic!();   
    }

    let new_project: Project = Project {
        name: result.clone(),
        path: path.unwrap().display().to_string(),
        editor: settings_data["commandToOpen"].as_str().unwrap().to_string(),
    };
    let p = serde_json::to_value(new_project).unwrap();
    next_settings["projects"].as_array_mut().unwrap().push(p);

    // Save next settings file
    println!(
        "{}",
        format!("Project {} is successfully added", result.cyan().bold()).green()
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

    let remove_prompt: &str = &format!("{} Select project to remove", "?".green());
    let result = browse(remove_prompt, settings_data);

    // Remove the project in json file
    next_settings = delete_project_json(next_settings, result.to_string());
    println!(
        "{}",
        format!("Project {} is successfully removed", result.cyan().bold()).green()
    );
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

    let seteditor_prompt: &str = &format!("{} Select project to set editor", "?".green());
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
        "\nUsage: sup <command>

Options:
  -V, --version                output the version number
  -h, --help                   output usage information

Commands:
  init                         Initialize Substrate dev environment 
  open|o                       Open one of your saved projects
  add|save                     Save current directory as a project
  remove                       Remove the project
  seteditor                    Set text editor to use
  run                          Run built Substrate node
  deploy                       Deploy network in local/cloud environment
  interact                     Open Substrate ui in the doc
  publish                      Publish in parachaintracker\n"
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

// Listen to Ctrl-C
extern crate signal_hook;
use log::{warn, debug};

pub fn reg_for_sigs() {
    unsafe { signal_hook::register(signal_hook::SIGINT, || on_sigint()) }
        .and_then(|_| {
                debug!("Registered for SIGINT");
                Ok(())
        })
        .or_else(|e| {
            warn!("Failed to register for SIGINT {:?}", e);
            Err(e)
        })
        .ok();
}

fn on_sigint() {
    warn!("SIGINT caught - exiting");
    std::process::exit(128 + signal_hook::SIGINT);
}