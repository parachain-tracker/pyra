use std::env;
extern crate dirs;
extern crate colored;
use std::process::Command;
use colored::*;
use webbrowser;
use std::fs;
use log::{debug, warn};

pub fn init_substrate(settings_data: serde_json::value::Value, project_name: String, author: String) {
    
    let project_dir = format!("{}/{}", env::current_dir().unwrap().display(), project_name);
    match fs::create_dir_all(project_dir.clone()) {
            Ok(_) => (),
            Err(why) => panic!("Failed to create dir: {}", why),
    }

    match env::set_current_dir(project_dir) {
        Ok(_) => (),
        Err(why) => panic!("Failed to set current dir: {}", why)
    }
    new_substrate_node(project_name.clone(), author);
    new_substrate_ui(project_name);  
}

pub fn new_substrate_node(project_name: String, author: String) {
    Command::new("bash")
        .args(&[&format!("{}-node/.cargo/bin/substrate-node-new", dirs::home_dir().unwrap().display()),&project_name, &author])
        .spawn()
        .expect("Failed to process substrate command");
}

pub fn new_substrate_ui(project_name: String) {
    Command::new("bash")
        .args(&[&format!("{}/.cargo/bin/substrate-ui-new", dirs::home_dir().unwrap().display()),&project_name])
        .spawn()
        .expect("Failed to process substrate command");
}

pub fn run_substrate(settings_data: serde_json::value::Value, path: String, project_name: String, substrate_bin_path: String) {
    let command = Command::new(&substrate_bin_path)
        .arg("--dev")
        .spawn()
        .expect("Failed to run substrate binary");
}

pub fn purge_substrate(settings_data: serde_json::value::Value, path: String, project_name: String, substrate_bin_path: String) {
    
    Command::new(&substrate_bin_path)
        .args(&["purge-chain","--dev", "-y"])
        .spawn()
        .expect("Failed to purge Substrate chain data");
}

pub fn build_substrate(settings_data: serde_json::value::Value, path: String, project_name: String) -> std::process::Child {

    let substrate_runtime_build_path = format!("{}/{}-node/scripts/build.sh", path.clone(), project_name.clone());
    // Build runtime WASM image
    Command::new("bash")
        .arg(substrate_runtime_build_path)
        .spawn()
        .expect("Failed to build Substrate runtime wasm image");
    
    let substrate_build_path = format!("{}/{}-node/Cargo.toml", path, project_name.clone());
    // Build Substrate binary from runtime wasm image
    let command = Command::new("cargo")
        .args(&["build".to_string(), "--release".to_string(), format!("--manifest-path={}", substrate_build_path)])
        .spawn()
        .expect("Failed to run substrate binary");
    command
}

pub fn run_substrate_ui(settings_data: serde_json::value::Value, path: String, project_name: String) {
    
    Command::new("yarn")
        .args(&["run".to_string(), "dev".to_string()])
        .spawn()
        .expect("Failed to run substrate ui");

    match webbrowser::open("http://localhost:8000") {
        Ok(_) => (),
        Err(why) => panic!("Failed to open webbrowser: {}", why)
    }  
}


