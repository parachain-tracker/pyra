use std::env;
extern crate dirs;
extern crate colored;
use std::process::Command;
use dialoguer::{
    Confirmation
};
use colored::*;
use webbrowser;
use std::fs;
use log::{debug, warn};

pub fn init_substrate(settings_data: serde_json::value::Value, project_name: String) {
    
    let project_dir = format!("{}/{}", env::current_dir().unwrap().display(), project_name);
    match fs::create_dir_all(project_dir.clone()) {
            Ok(_) => (),
            Err(why) => panic!("Failed to create dir: {}", why),
    }

    match env::set_current_dir(project_dir) {
        Ok(_) => (),
        Err(why) => panic!("Failed to set current dir: {}", why)
    }
    new_substrate_node(project_name.clone());
    new_substrate_ui(project_name.clone());  
    new_polkadot_js_app(project_name);

    
}

pub fn new_substrate_node(project_name: String) {
    Command::new("git")
        .args(&["clone", "https://github.com/paritytech/substrate.git", "--branch", "v1.0", &format!("{}-node", &project_name)])
        .spawn()
        .expect("Failed to clone substrate");
}

pub fn new_substrate_ui(project_name: String) {
    Command::new("git")
        .args(&["clone", "https://github.com/substrate-developer-hub/substrate-front-end-template.git", "--branch", "master", &format!("{}-ui", &project_name)])
        .spawn()
        .expect("Failed to clone substrate");
}

pub fn new_polkadot_js_app(project_name: String) {
    Command::new("git")
        .args(&["clone", "https://github.com/polkadot-js/apps.git", "--branch", "master", &format!("{}-polkadotjs-app", &project_name)])
        .spawn()
        .expect("Failed to process git command");
}

pub fn run_substrate(project_name: String, path: String) {
    let substrate_bin_path = format!(
        "{}/{}-node/target/release/{}-node",
        path,
        project_name.clone(),
        project_name.clone()
    );
    println!("{:?}", substrate_bin_path);
    let command = Command::new(&substrate_bin_path)
        .arg("--dev")
        .spawn()
        .expect("Failed to run substrate binary");
    let pid = command.id().to_string().green().bold();
    println!(
        "{}",
        format!(
            "Substrate daemon running at pid {}. kill the process with `kill {}` command",
            pid, pid
        )
        .magenta()
        .bold()
        .to_string()
    );
}

pub fn purge_substrate(project_name: String, path: String) {
    let substrate_bin_path = format!(
        "{}/{}-node/target/release/{}-node",
        path,
        project_name.clone(),
        project_name.clone()
    );
    if Confirmation::new()
        .with_text("\u{26A0} Are you sure you want to remove the whole chain data?")
        .interact()
        .unwrap()
    {
        Command::new(&substrate_bin_path)
            .args(&["purge-chain", "--dev", "-y"])
            .spawn()
            .expect("Failed to purge Substrate chain data");
        println!("{}", format!("{} chain is now purging with significant update. Start fresh with the new blank slate", project_name).magenta().bold().to_string());
    } else {
        println!("It's okay, take your time :)");
        return;
    }
}

pub fn build_substrate(project_name: String, path: String, target: String) {
    let substrate_runtime_build_path = format!(
        "{}/{}-node/scripts/build.sh",
        path.clone(),
        project_name.clone()
    );
    // Build runtime WASM image
    Command::new("bash")
        .arg(substrate_runtime_build_path)
        .spawn()
        .expect("Failed to build Substrate runtime wasm image");
    let substrate_build_path = format!("{}/{}-node/Cargo.toml", path, project_name.clone());
    
    if target == "runtime" {return;}

    // Build Substrate binary from runtime wasm image
    let command = Command::new("cargo")
        .args(&[
            "build".to_string(),
            "--release".to_string(),
            format!("--manifest-path={}", substrate_build_path),
        ])
        .spawn()
        .expect("Failed to run substrate binary");
    let pid = command.id().to_string().green().bold();
    format!(
        "Substrate daemon running at pid {}. kill the process with `kill {}` command",
        pid, pid
    )
    .magenta()
    .bold()
    .to_string();
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


