


pub fn init_substrate(settings_data: serde_json::value::Value, project_name: String, author: String) {
    
    let project_dir = format!("{}/{}", env::current_dir().unwrap().display(), project_name);

    pause("The installation will take long which is roughly about 15 minutes \u{1f422}.\nPress any key to continue and wait with \u{2615} or Ctrl + C to quit...".to_string());

    println!(
            "Generating project_directory at {}...",
            project_dir.clone()
    );
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
    reg_for_sigs();
    Command::new("bash")
        .args(&[&format!("{}-node/.cargo/bin/substrate-node-new", dirs::home_dir().unwrap().display()),&project_name, &author])
        .spawn()
        .expect("Failed to process substrate command");
}

pub fn new_substrate_ui(project_name: String) {
    reg_for_sigs();
    Command::new("bash")
        .args(&[&format!("{}/.cargo/bin/substrate-ui-new", dirs::home_dir().unwrap().display()),&project_name])
        .spawn()
        .expect("Failed to process substrate command");
}

pub fn run_substrate(settings_data: serde_json::value::Value, path: String, project_name: String) {
    let substrate_bin_path = format!("{}/{}-node/target/release/{}-node", path, project_name.clone(), project_name.clone());
    let command = Command::new(&substrate_bin_path)
        .arg("--dev")
        .spawn()
        .expect("Failed to run substrate binary");
    let pid = command.id().to_string().green().bold();
    println!("{}",format!("Substrate daemon running at pid {}. kill the process with `kill {}` command", pid, pid).magenta().bold().to_string());
}

pub fn purge_substrate(settings_data: serde_json::value::Value, path: String, project_name: String) {
    
    let substrate_bin_path = format!("{}/{}-node/target/release/{}-node", path, project_name.clone(), project_name.clone());
    if Confirmation::new()
        .with_text("\u{26A0} Are you sure you want to remove the whole chain data?")
        .interact()
        .unwrap()
    {
        Command::new(&substrate_bin_path)
        .args(&["purge-chain","--dev", "-y"])
        .spawn()
        .expect("Failed to purge Substrate chain data");
        println!("{}", format!("{} chain is now purging with significant update. Start fresh with the new blank slate", project_name).magenta().bold().to_string());
    } else {
        println!("It's okay, take your time :)");
        return;
    }
}

pub fn build_substrate(settings_data: serde_json::value::Value, path: String, project_name: String) {

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
    let pid = command.id().to_string().green().bold();
    format!("Substrate daemon running at pid {}. kill the process with `kill {}` command", pid, pid).magenta().bold().to_string();
}

pub fn run_substrate_ui(settings_data: serde_json::value::Value, path: String, project_name: String) {
    
    let substrate_ui_path = format!("{}/{}-ui", path, project_name.clone());
        match env::set_current_dir(&substrate_ui_path) {
        Ok(_) => (),
        Err(why) => panic!("Failed to set current dir: {}", why)
    }
    println!("{:?}", &substrate_ui_path);
    reg_for_sigs();
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