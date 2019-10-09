use rand;

use rand::seq::SliceRandom;
use rand::Rng;

use std::env;
use std::fs;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};
use std::cmp::min;
use std::io::{BufRead, BufReader};
use std::process;

use dialoguer::{
    Confirmation
};
extern crate dirs;
extern crate colored;
use colored::*;
use webbrowser;
use log::{debug, warn};
use console::{style, Emoji};
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};


static PACKAGES: &'static [&'static str] = &[
    "substrate-node-template",
    "substrate-frontend-template",
    "polkadot-js-apps",
];

static COMMANDS: &'static [&'static str] = &[
    "cargo build",
    "yarn",
    "yarn",
];

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static TRUCK: Emoji<'_, '_> = Emoji("🚚  ", "");
static CLIP: Emoji<'_, '_> = Emoji("🔗  ", "");
static PAPER: Emoji<'_, '_> = Emoji("📃  ", "");
static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");

pub fn init_substrate(path: String, project_name: String) {
    
    
    match fs::create_dir_all(path.clone()) {
            Ok(_) => (),
            Err(why) => panic!("Failed to create dir: {}", why),
    }

    match env::set_current_dir(path.clone()) {
        Ok(_) => (),
        Err(why) => panic!("Failed to set current dir: {}", why)
    }

    let started = Instant::now();
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}");

    println!(
        "{} {}Resolving packages...",
        style("[1/4]").bold().dim(),
        LOOKING_GLASS
    );

    println!(
        "{} {}Fetching packages...",
        style("[2/4]").bold().dim(),
        TRUCK
    );
    // Fetch packages
    let project_name1 = project_name.clone();
    let computation = thread::spawn(move || {
        new_substrate_node(project_name1.clone());
        new_substrate_frontend(project_name1.clone());
        new_polkadot_js_app(project_name1);
    });
    computation.join().unwrap();

    
    let project_name2 = project_name.clone();
    println!(
        "{} {}Linking dependencies...",
        style("[3/4]").bold().dim(),
        CLIP
    );


    let deps = 1232;
    let pb = ProgressBar::new(deps);
    for _ in 0..deps {
        pb.inc(1);
        thread::sleep(Duration::from_millis(3));
    }
    pb.finish_and_clear();

    println!(
        "{} {}Building fresh packages...",
        style("[4/4]").bold().dim(),
        PAPER
    );

    // Build packages
    let m = MultiProgress::new();
    let pb = m.add(ProgressBar::new_spinner());
    pb.enable_steady_tick(200);
    pb.set_style(ProgressStyle::default_spinner()
        .tick_chars("/|\\- ")
        .template("{prefix:.bold.dim} [{elapsed_precise}] {spinner:.dim.bold} cargo: {wide_msg}"),
    );
    pb.set_prefix(&format!("[{}/3]", 1));
    pb.set_message("cargo: build substrate_runtime");
    let temp0 = project_name.clone();
    let path_temp0 = path.clone();
    let _ = thread::spawn(move || {
        let substrate_build_path = format!("{}/{}-node/Cargo.toml", path_temp0, temp0.clone());
        
        let mut p = build_substrate_node(temp0, path_temp0);
        for line in BufReader::new(p.stderr.take().unwrap()).lines() {
        let line = line.unwrap();
        let stripped_line = line.trim();
        if !stripped_line.is_empty() {
            pb.set_message(stripped_line);
        }
        pb.tick();
    }

    p.wait().unwrap();

    pb.finish_with_message("waiting...");
    });


    let sty = ProgressStyle::default_spinner()
            .tick_chars("/|\\- ")
            .template("{prefix:.bold.dim} [{elapsed_precise}] {spinner:.dim.bold} {wide_msg}");
    for i in 1..3 {
        let temp0 = project_name.clone();
        let temp1 = project_name.clone();
        let path_temp0 = path.clone();
        let path_temp1 = path.clone();
        
        let mut rng = rand::thread_rng();
        let count = rng.gen_range(30, 80);
        let pb = m.add(ProgressBar::new(count));
        pb.set_style(sty.clone());
        pb.set_prefix(&format!("[{}/3]", i + 1));
        let _ = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let pkg = PACKAGES[i];
            let cmd = COMMANDS[i];
            pb.set_message(&format!("{}: {}", pkg, cmd));
            match i {
                0 => {
                    let frontend_path = format!("{}/{}-frontend", path_temp0, temp0.clone());
                    let mut p = build_substrate_frontend(frontend_path);
                    for line in BufReader::new(p.stderr.take().unwrap()).lines() {
                        let line = line.unwrap();
                        let stripped_line = line.trim();
                        if !stripped_line.is_empty() {
                            pb.set_message(stripped_line);
                        }
                        pb.tick();
                    }
                    p.wait().unwrap();
                    },
                1 => {
                    let apps_path = format!("{}/{}-polkadotjs-apps", path_temp1, temp1.clone());
                    let mut p = build_substrate_frontend(apps_path);
                    for line in BufReader::new(p.stderr.take().unwrap()).lines() {
                        let line = line.unwrap();
                        let stripped_line = line.trim();
                        if !stripped_line.is_empty() {
                            pb.set_message(stripped_line);
                        }
                        pb.tick();
                    }
                    p.wait().unwrap();
                    },
                _ => panic!("nothing other than 0,1"),
            }
            
            pb.finish_with_message("waiting...");
        });
    }
    m.join_and_clear().unwrap();

    

    println!("{} Done in {}", SPARKLE, HumanDuration(started.elapsed()));
}

pub fn new_substrate_node(project_name: String) {
    println!("Fetching substrate node...");
    Command::new("git")
        .args(&["clone", "https://github.com/paritytech/substrate.git", "--branch", "v1.0", &format!("{}-node", &project_name)])
        .output()
        .expect("Failed to clone substrate");
}

pub fn new_substrate_frontend(project_name: String) {
    println!("Fetching substrate frontend template...");
    Command::new("git")
        .args(&["clone", "https://github.com/substrate-developer-hub/substrate-front-end-template.git", "--branch", "master", &format!("{}-frontend", &project_name)])
        .output()
        .expect("Failed to clone substrate");
}

pub fn new_polkadot_js_app(project_name: String) {
    println!("Fetching polkadot.js apps...");
    Command::new("git")
        .args(&["clone", "https://github.com/polkadot-js/apps.git", "--branch", "master", &format!("{}-polkadotjs-apps", &project_name)])
        .output()
        .expect("Failed to process git command");
}

pub fn run_substrate(project_name: String, path: String) {
    let substrate_bin_path = format!(
        "{}/{}-node/target/release/substrate",
        path,
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

pub fn build_substrate_runtime(project_name: String, path: String) -> std::process::Child {
    env::set_current_dir(format!("{}/{}-node", path.clone(), project_name.clone()));
    let substrate_runtime_init_path = format!(
        "{}/{}-node/scripts/init.sh",
        path.clone(),
        project_name.clone()
    );
    let substrate_runtime_build_path = format!(
        "{}/{}-node/scripts/build.sh",
        path.clone(),
        project_name.clone()
    );

    // Build runtime WASM image
    Command::new("bash")
        .arg(substrate_runtime_build_path)
        .spawn()
        .unwrap()
}

pub fn build_substrate_node(project_name: String, path: String) -> std::process::Child {
    let path_temp = path.clone();
    let temp = project_name.clone();
    let computation = thread::spawn(move || {
        let mut p = build_substrate_runtime(temp, path_temp);
    });
    
    computation.join().unwrap();
    
    let substrate_build_path = format!("{}/{}-node/Cargo.toml", path.clone(), project_name.clone());
    

    // Build Substrate binary from runtime wasm image
    Command::new("cargo")
        .args(&[
            "build".to_string(),
            "--release".to_string(),
            format!("--manifest-path={}", substrate_build_path)
        ])
        .spawn()
        .unwrap()
}

pub fn build_substrate(project_name: String, path: String, target: String) {
    if target=="node" {
        let mut p = build_substrate_node(project_name.clone(), path.clone());
        }
    if target=="runtime" {
        let mut p = build_substrate_runtime(project_name, path);
        }
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

pub fn build_substrate_frontend(path: String) -> std::process::Child {
    env::set_current_dir(path);
    Command::new("yarn")
        .stderr(process::Stdio::piped())
        .spawn()
        .unwrap()
}
