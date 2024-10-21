use sysinfo::System;

use online::check;


use std::process::Command;
use std::collections::HashMap;

use std::process::Output;

use sys_info::disk_info;

use byte_unit::{Byte, UnitType};


use figlet_rs::FIGfont;
use std::fs;

use compound_duration::format_dhms;
use regex::Regex;


// ANSI colored output codes
const CYAN: &str = "\x1b[0;36m";
const BLUE: &str = "\x1b[0;34m";
const GREEN: &str = "\x1b[0;32m";
const PURPLE: &str = "\x1b[0;35m";
const YELLOW: &str = "\x1b[0;33m";
const RED: &str = "\x1b[0;31m";
const RESET: &str = "\x1b[0m";


// Bar Constants
const BAR_LENGTH: usize = 25;



fn gen_figlet(hostname: &str) -> String {
    // Using non-standard font
    let doom_font = FIGfont::from_file("resources/Doom.flf").expect("Could not find font file");
    // Generating figlet from hostname
    let figure = doom_font.convert(hostname);
    assert!(figure.is_some());
    // Adding tab to begining of figlet
    let mut fig_text: String = "\t".to_owned() + &figure.unwrap().to_string();
    // Adding tabs to begining of each following lines
    fig_text = str::replace(&fig_text, "\n", "\n\t");
    // Removed extra tab
    fig_text.pop();
    return fig_text;
}


fn gen_uptime() -> String {
    let uptime = format_dhms(System::uptime());
    // Create a regular expression to match patterns like "1d", "16h", "5m", "3s"
    let re = Regex::new(r"(\d+[dhms])").unwrap();
    // Use the regex to find and replace matches with a space in between
    let result = re.replace_all(&uptime, |caps: &regex::Captures| {
        format!(" {}", &caps[0])
    }).trim().to_string(); // Trim any extra spaces at the start and end
    return result;

}


fn gen_names() -> String {
    let mut result: String = String::new();
    // User and Machine credentials
    let user_name: String = whoami::realname();
    let system_name: String = whoami::username();
    // Adds name
    result.push_str(&format!("Welcome Back, {}{}{}", YELLOW, &user_name, RESET));
    // Add if there is unique name
    if user_name == system_name {
        result.push_str("");
    } else {
        result.push_str(&format!(" ({}{}{})", CYAN, &system_name, RESET));  // Prints Username
    }
    return result;
}


fn gen_welcome() -> String {
    let mut result: String = String::new();
    let uptime: String = gen_uptime();
    // Finding and printing hostname
    let hostname = sysinfo::System::host_name();
    // Matching for returned hostname
    let formatted_uptime: String = match hostname {
        // Hostname found
        Some(x) => {
            // Adds figlet of hostname to result String
            result.push_str(&gen_figlet(&x));
            format!("Uptime on {GREEN}{x}{RESET}: {uptime}")
        } ,
        // Hostname not found
        None    => format!("Uptime: {uptime}") ,
    };
    result.push_str(&format!("{}\n{}\n{}", gen_names(), formatted_uptime, system_info()));
    return result;
}


fn system_info() -> String {
    // Fetching system OS name and kernel version
    let host_os = System::name().expect("No OS name found");
    let host_kernel = System::kernel_version().expect("No Kernal version found");
    return format!("OS: {YELLOW}{}{RESET} ({RED}{}{RESET})", host_os, host_kernel);
}


fn gradient_color(percentage: f32) -> String {
    if percentage < 50.0 {
        // Green to Yellow
        let red = (percentage / 50.0 * 255.0) as u8;  // Increases red as percentage increases
        let green = 255;  // Full green
        format!("\x1b[38;2;{};{};0m", red, green)  // RGB format
    } else {
        // Yellow to Red
        let red = 255;  // Full red
        let green = (255.0 - ((percentage - 50.0) / 50.0 * 255.0)) as u8;  // Decreases green to 0
        format!("\x1b[38;2;{};{};0m", red, green)  // RGB format
    }
}

fn gen_bar(name: &str, used: u64, total: u64) -> String {
    // Creating Bar String and Name
    let mut result: String = String::new();
    result.push_str(&format!("{}{}{}\t[", BLUE, name, RESET));

    let percent: f64 = used as f64 / total as f64 * 100.0; // Calculate percentage
    let num_bars: usize = (BAR_LENGTH as f64 * (percent / 100.0)) as usize;

    // Generate the colored bar segments
    for i in 0..BAR_LENGTH {
        if i < num_bars {
            let segment_percentage = (i as f32 / BAR_LENGTH as f32) * 100.0;
            result.push_str(&gradient_color(segment_percentage)); // Add the color
            result.push('█'); // Add the filled block
        } else {
            result.push(' '); // Add space for the empty part
        }
    }

    result.push_str(RESET); // Reset color after the bar
    result.push_str("]");

    result
}

//fn gen_bar(name: &str, used: u64, total: u64) -> String {
//    // Creating Bar String and Name
//    let mut result: String = String::new();
//    result.push_str(&format!("{}{}{}\t[", BLUE, name, RESET));
//
//    let percent: f64 = used as f64 / total as f64;
//    let num_bars: usize = (BAR_LENGTH as f64 * percent) as usize;
//
//    result.push_str(&"█".repeat(num_bars));
//    result.push_str(&" ".repeat(BAR_LENGTH - num_bars));
//
//    result.push_str("]");
//
//    return result;
//}


fn gen_percent(used: u64, total: u64) -> String {
    let mut result: String = String::new();
    let percent_used: f64 = used as f64/ total as f64;
    // use correct memory sizes
    let bytes_used = Byte::from_u64(used).get_appropriate_unit(UnitType::Binary);
    let bytes_total = Byte::from_u64(total).get_appropriate_unit(UnitType::Binary);
    // formatting decimal places
    let string_used: String = format!("{bytes_used:.2}");
    let string_total: String = format!("{bytes_total:.2}");
    // printing memory usages
    result.push_str(&format!("  {} / {} [{}{:.2}%{}]",
        string_used,
        string_total,
        get_warning_color(percent_used),
        percent_used * 100.0,
        RESET));
    return result;
}


fn gen_memory(sys: &System) -> String {
    let mut result: String = String::new();
    // collecting memory sizes (used and total)
    let used_memory: u64 = sys.used_memory();
    let total_memory: u64 = sys.total_memory();
    result.push_str(&format!("{}{}",
        &gen_bar("memory", used_memory, total_memory), // Bar of memory usage
        &gen_percent(used_memory, total_memory)));     // Percentage of memory used
    return result;
}


fn gen_disks() -> String {
    let mut result: String = String::new();
    let free_storage: u64 = match disk_info() {
        Ok(x) => x.free * 1000,
        _     => 0
    };
    let total_storage: u64 = match disk_info() {
        Ok(x) => x.total * 1000,
        _     => 0
    };

    let used_storage: u64 = total_storage - free_storage;
    result.push_str(&format!("{}{}", gen_bar(&"storage", used_storage, total_storage),
                                     gen_percent(used_storage, total_storage)));
    return result;
}


fn get_warning_color(percent: f64) -> String {
    let color: &str = match percent {
        x if x > 0.90 => RED,
        x if x > 0.70 => YELLOW,
        _ => GREEN
    };
    return color.to_string();
}


fn check_systemd(process_name: &str, command: &str) -> bool {
    return match
    Command::new("systemctl")
        .arg(command)
        .arg(process_name)
        .output()
        .unwrap()
        .status
        .code() {
        Some(0) => true,
        _ => false
    };
}

fn parse_processes() -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();
    // Fetches the processes listed in file
    let processes: String = fs::read_to_string("resources/processes.txt").expect("could not read processes file");
    // Inserts each process listed into result
    for process in processes.trim().split("\n") {
        // Splits on alias denominator
        let split: Vec<&str> = process.split(":").collect::<Vec<&str>>();
        if split.len() > 1 { // Has found alias
           result.insert(split[0].to_string(), split[1].to_string()); 
        } else { // Has no found alias
            result.insert(process.to_string(), process.to_string());
        }
    }
    return result;
}




fn check_processes() -> String {
    let programs: HashMap<String, String> = parse_processes();
    let mut result: String = String::new();
    // Tuple of active and enabled variables
    let (mut is_active, mut is_enabled): (bool, bool);
    // Tuple of active and enabled color variables
    let (mut active_color, mut enabled_color): (&str, &str);
    // Stores output of systemctl command
    let mut output: Output;
    // Used to store pid of current process
    let mut pid: String;

    // Checking status of each process listed
    for program in programs {
        // Checking status of process and setting correct color
        is_active = check_systemd(&program.0, "is-active");
        is_enabled = check_systemd(&program.0, "is-enabled");
        active_color = if is_active { GREEN } else { RED };
        enabled_color = if is_enabled { GREEN } else { RED };
        
        // Generates line of colored active and enabled program
        result.push_str(&format!("{:<15}\t[{}active{RESET}] [{}enabled{RESET}]", &program.1, active_color, enabled_color));

        if is_active {
            // Obtains PID of program running on system
            output = Command::new("systemctl")
                .arg("show")
                .arg("--property")
                .arg("MainPID")
                .arg("--value")
                .arg(program.0)
                .output()
                .expect("Error finding PID");
            pid = String::from_utf8_lossy(&output.stdout).trim().to_string();
        } else {
            pid = String::from("-".repeat(5));
        }
        result.push_str(&format!(" [{:>5}]\n", pid));
    }
    result.pop();
    return result;
}


async fn gen_package_check() -> String {
    let mut result: String = String::from("Network Status: ");
    // Checks online status of machine
    match check(None).is_ok() {
        true => result.push_str(&format!("{GREEN}Online{RESET}")) ,
        // Returns early is no network connection
        false => {result.push_str(&format!("{RED}Offline{RESET}")); return result}
    };
    // Checks updates via checkupdates (arch) command
    let output = Command::new("checkupdates")
        .output()
        .expect("Error finding packages");

    if output.status.success() {
        result.push_str("\nUpgradable Packages: ");
        let num_packages: usize = String::from_utf8_lossy(&output.stdout)
            .as_bytes()
            .iter()
            .filter(|&&c| c == b'\n') // Makes entry based on new lines
            .count();
        result.push_str(&format!("{YELLOW}{}{RESET}", num_packages));
    } else {
        result.push_str("\nNo packages found");
    }
    return result;
}

#[tokio::main]
async fn main() {
    // Creates lazy async spawn for gen_package_check to run while program executes
    let handle = tokio::spawn(async move {gen_package_check().await});

    let sys = System::new_all();
    println!("{}", gen_welcome());
    println!("{}", gen_memory(&sys));
    println!("{}", gen_disks());
    println!("{PURPLE}{:>32}{RESET}", "Applications:");
    println!("{}", check_processes());
    println!("{PURPLE}{:>32}{RESET}", "Connections:");
    // Waits for async call to gen_package_list to finish, then prints
    println!("{}", handle.await.unwrap());
}
