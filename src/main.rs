use sysinfo::{
    Components, Disks, Networks, System
};

use std::time::Duration;
use online::check;


use std::process::Command;


use std::process::Output;

use sys_info::disk_info;

use byte_unit::{Byte, UnitType};


use figlet_rs::FIGfont;

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


fn gen_bar(name: &str, used: u64, total: u64) -> String {
    // Creating Bar String and Name
    let mut result: String = String::new();
    result.push_str(&format!("{}{}{}\t[", BLUE, name, RESET));

    let percent: f64 = used as f64 / total as f64;
    let num_bars: usize = (BAR_LENGTH as f64 * percent) as usize;

    result.push_str(&"█".repeat(num_bars));
    result.push_str(&" ".repeat(BAR_LENGTH - num_bars));

    result.push_str("]");

    return result;
}


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


fn gen_disks() {
    let free_storage: u64 = match disk_info() {
        Ok(x) => x.free * 1000,
        _     => 0
    };
    let total_storage: u64 = match disk_info() {
        Ok(x) => x.total * 1000,
        _     => 0
    };

    let used_storage: u64 = total_storage - free_storage;
    print!("{}", gen_bar(&"storage", used_storage, total_storage));
    println!("{}", gen_percent(used_storage, total_storage));
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


fn check_processes(sys: System) {
    // Create a vector of string slices
    let programs: Vec<&str> = vec![
        "minecraftd",
        "jellyfin",
        "docker",
        "portainer",
        "qbittorrent",
        "sshd",
        "plex",
        "firewalld",
        "radarr",
        "sonarr",
        "readarr",
        "jellyseerr",
        "prowlarr",
        "grafana",
        "homepage",
    ];
    let mut is_active: bool;
    let mut is_enabled: bool;
    let mut active_color: &str;
    let mut enabled_color: &str;
    let mut output;
    let mut pid: String;

    // Print the vector
    for program in &programs {
        is_active = check_systemd(program, "is-active");
        is_enabled = check_systemd(program, "is-enabled");
        active_color = if is_active { GREEN } else { RED };
        enabled_color = if is_enabled { GREEN } else { RED };
        
        print!("{:<15}\t[{}active{RESET}] [{}enabled{RESET}]", program, active_color, enabled_color);

        if is_active {
            output = Command::new("systemctl")
                .arg("show")
                .arg("--property")
                .arg("MainPID")
                .arg("--value")
                .arg(program)
                .output()
                .expect("Error finding PID");

            pid = if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else { String::from("----") };
            print!(" [{:>4}]", pid);
        }
        println!();

    }
}


fn gen_package_check() -> String {
let mut result: String = String::from("Network Status: ");
// Checks online status of machine
match check(None).is_ok() {
    true => result.push_str(&format!("{GREEN}Online{RESET}")) ,
    // Returns early is no network connection
    false => {result.push_str(&format!("{RED}Offline{RESET}")); return result}
};
    let output = Command::new("checkupdates")
        .output()
        .expect("Error finding packages");

    if output.status.success() {
        result.push_str("\nUpgradable Packages: ");
        let num_packages: usize = String::from_utf8_lossy(&output.stdout)
            .as_bytes()
            .iter()
            .filter(|&&c| c == b'\n')
            .count();
        result.push_str(&format!("{YELLOW}{}{RESET}", num_packages));
    } else {
        result.push_str("\n{RED}Error{RESET} finding packages");
    }
    return result;
}


fn main() {
    let packages: String = gen_package_check();
    println!("{}", gen_welcome());
    // Original System Query
    let sys = System::new_all();
    println!("{}", gen_memory(&sys));
    gen_disks();
    println!("{PURPLE}{:>32}{RESET}", "Applications:");
    check_processes(sys);
    println!("{PURPLE}{:>32}{RESET}", "Connections:");
    println!("{}", packages);


}
