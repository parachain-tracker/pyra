use rand;

use rand::seq::SliceRandom;
use rand::Rng;

use std::env;
use std::fs;


use std::process;
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process;
#[cfg(target_os = "windows")]
use std::os::windows::process::Command;
use std::thread;
use std::time::{Duration, Instant};
use std::cmp::min;
use std::io::{BufRead, BufReader};


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


static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static TRUCK: Emoji<'_, '_> = Emoji("🚚  ", "");
static GEAR: Emoji<'_, '_> = Emoji("⚙️  ", "");
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
    new_git_clone("Substrate node template",
    "https://github.com/hskang9/haski.git",
    "patch-1", 
    &format!("{}-node", &project_name)
    );
    new_git_clone("Substrate frontend template", 
    "https://github.com/substrate-developer-hub/substrate-front-end-template.git",
    "master",
    &format!("{}-frontend", &project_name)
    );
    new_git_clone("Polkadot-js apps", 
    "https://github.com/polkadot-js/apps.git",
    "master",
    &format!("{}-polkadotjs-apps", &project_name)
    );
    
    let project_name2 = project_name.clone();
    println!(
        "{} {}Setting WASM environment...",
        style("[3/4]").bold().dim(),
        GEAR
    );
    
    initialize_wasm_environment(path.clone(), project_name.clone());

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
        .template("{prefix:.bold.dim} [{elapsed_precise}] {spinner:.dim.bold} substrate: {wide_msg}"),
    );
    pb.set_prefix(&format!("[{}/?]", 1));
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
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:.bold.dim} [{elapsed_precise}] {spinner:.magenta} {wide_msg}");
    for i in 1..3 {
        let temp0 = project_name.clone();
        let temp1 = project_name.clone();
        let path_temp0 = path.clone();
        let path_temp1 = path.clone();
        let pb = m.add(ProgressBar::new(100));
        pb.set_style(sty.clone());
        pb.set_prefix(&format!("[{}/?]", i + 1));  
                    
        let _ = thread::spawn(move || {
            pb.enable_steady_tick(100);  
            match i {
                1 => {
                    pb.set_message("substrate-frontend-template: yarn");
                    let frontend_path = format!("{}/{}-frontend", path_temp0, temp0.clone());
                    let mut p = build_substrate_frontend(frontend_path);   
                    p.wait().unwrap();
                    },
                2 => {
                    pb.set_message("polkadot-js-apps: yarn");
                    let apps_path = format!("{}/{}-polkadotjs-apps", path_temp1, temp1.clone());
                    let mut p = build_substrate_frontend(apps_path);       
                    p.wait().unwrap();
                    },
                _ => panic!("nothing other than 1 or 2"),
            }
            
            pb.finish_with_message("waiting...");
        });
    }
    m.join_and_clear().unwrap();

    

    println!("{} Done in {}", SPARKLE, HumanDuration(started.elapsed()));
}

/* TODO: Ask online crate maintainer to update to working version
pub fn check_network() -> bool {
    // with timeout
    let timeout = Duration::new(6, 0);
    assert_eq!(online(Some(timeout)), Ok(true));
}
*/

pub fn new_git_clone(repo: &str, link: &str, branch: &str, directory: &str) {
   let n = 10000;
   println!("Fetching {}...", repo);
   let pb = ProgressBar::new(n);
   
   let started = Instant::now();
   
   /* Check network connection
   if !check_network() {
       panic!("Invalid connection: network not connected");
   }
   */

   let mut p = Command::new("git")
        .args(&["clone", link, "--branch", branch, directory])
        .stderr(process::Stdio::piped())
        .spawn()
        .unwrap();

   for _ in 0..n {
       pb.inc(1);
       if p.wait().unwrap().success() {
           break
       }
    }
    pb.finish();

    let finished = started.elapsed();
    
    println!(
        "✅  Fetched {} in {}",
        repo,
        HumanDuration(finished)
    )
}

pub fn initialize_wasm_environment(path: String, project_name: String) {
    let substrate_runtime_init_path = format!(
        "{}/{}-node/scripts/init.sh",
        path.clone(), project_name.clone());

        

    // Init wasm environment
    let mut p = Command::new("bash")
        .arg(substrate_runtime_init_path)
        .stderr(process::Stdio::piped())
        .spawn()
        .unwrap();  
    p.wait().unwrap();  
    println!("✅  WASM environment is set");
}

pub fn build_substrate_runtime(project_name: String, path: String) -> std::process::Child {
    let substrate_runtime_build_path = format!("{}/{}-node/runtime/Cargo.toml", path.clone(), project_name.clone());
    env::set_current_dir(substrate_runtime_build_path.clone());
    return Command::new("cargo")
        .args(&[
            "build".to_string(),
            "--release".to_string(),
            format!("--manifest-path={}", substrate_runtime_build_path)
        ])
        .stderr(process::Stdio::piped())
        .spawn()
        .unwrap();
}

pub fn build_substrate_node(project_name: String, path: String) -> std::process::Child {
    let substrate_build_path = format!("{}/{}-node/Cargo.toml", path.clone(), project_name.clone());
    
    // Build Substrate binary from runtime wasm image
    return Command::new("cargo")
        .args(&[
            "build".to_string(),
            "--release".to_string(),
            format!("--manifest-path={}", substrate_build_path)
        ])
        .stderr(process::Stdio::piped())
        .spawn()
        .unwrap();
}

pub fn build_substrate(project_name: String, path: String, target: String) {
    if target=="node" {
        if Confirmation::new()
                .with_text("\u{26A0} The previous binary will be lost regardless of an error and the build time is estimated to be 20-30 minutes. Are you sure that all codes have been checked?")
                .interact()
                .unwrap()
        {
            let substrate_build_path = format!("{}/{}-node/Cargo.toml", path.clone(), project_name.clone());
            reg_for_sigs();
            // Build Substrate binary
            let p = thread::spawn(move || {
                Command::new("cargo")
                    .args(&[
                    "build".to_string(),
                    "--release".to_string(),
                    format!("--manifest-path={}", substrate_build_path)
                ])
                .spawn()
                .unwrap();
            });
            p.join().unwrap();
        } else {
            println!("It's okay, take your time :)");
            return;
        }
    }
    if target=="runtime" {
        let substrate_runtime_build_path = format!("{}/{}-node/runtime/Cargo.toml", path.clone(), project_name.clone());
        env::set_current_dir(substrate_runtime_build_path.clone());
        reg_for_sigs();
        let substrate_runtime_wasm_path = format!("{}/{}-node/target/debug/wbuild/node-template-runtime/", path.clone(), project_name.clone());
        println!("{}", format!("{} Runtime WASM file will be generated in {}", PAPER, substrate_runtime_wasm_path).green());
        let p = thread::spawn(move || {
            Command::new("cargo")
            .args(&[
                "build".to_string(),
                "--release".to_string(),
                format!("--manifest-path={}", substrate_runtime_build_path)
            ])
            .spawn()
            .unwrap();
        });
        p.join().unwrap();
    }
}

pub fn build_substrate_frontend(path: String) -> std::process::Child {
    return Command::new("yarn")
        .args(&["--cwd", &path])
        .stderr(process::Stdio::piped())
        .spawn()
        .unwrap();
}


// Post-initialization functions
pub fn run_substrate(project_name: String, path: String) {
    let substrate_bin_path = format!(
        "{}/{}-node/target/debug/node-template",
        path,
        project_name.clone()
    );
    println!("{:?}", substrate_bin_path);
    reg_for_sigs();
    let command = Command::new(&substrate_bin_path)
        .arg("--dev")
        .spawn()
        .expect("Failed to run substrate binary");
    let pid = command.id().to_string().green().bold();
    println!(
        "{}",
        format!(
            "Substrate daemon running at pid {}. kill the daemon with `kill {}` command",
            pid, pid
        )
        .magenta()
        .bold()
        .to_string()
    );
}

pub fn purge_substrate(project_name: String, path: String) {
    let substrate_bin_path = format!(
        "{}/{}-node/target/release/node-template",
        path,
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


// Listen to Ctrl-C
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