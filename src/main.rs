use sysinfo::{
    Components, Disks, Networks, System,
};

use byte_unit::{Byte, Unit, UnitType};


use compound_duration::format_dhms;
use regex::Regex;


// ANSI colored output codes
const CYAN: &str = "\x1b[0;36m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[0;33m";
const RED: &str = "\x1b[0;31m";
const RESET: &str = "\x1b[0m";


fn output_uptime() {
    let uptime = format_dhms(System::uptime());
    // Create a regular expression to match patterns like "1d", "16h", "5m", "3s"
    let re = Regex::new(r"(\d+[dhms])").unwrap();
    // Use the regex to find and replace matches with a space in between
    let result = re.replace_all(&uptime, |caps: &regex::Captures| {
        format!(" {}", &caps[0])
    }).trim().to_string(); // Trim any extra spaces at the start and end

    // Finding and printing hostname
    let hostname = sysinfo::System::host_name();
    // Matching for returned hostname
    match hostname {
        Some(x) => println!("Uptime on {GREEN}{x}{RESET}: {result}"),
        None    => println!("Uptime: {result}"),
    }
}


fn system_info() -> String {
    // Fetching system OS name and kernel version
    let host_os = System::name().expect("No OS name found");
    let host_kernel = System::kernel_version().expect("No Kernal version found");
    return format!("OS: {YELLOW}{}{RESET} ({RED}{}{RESET})", host_os, host_kernel);
}


fn main() {
    // Original System Query
    let sys = System::new_all();
    // User and Machine credentials
    let user_name: String = whoami::realname();
    let system_name: String = whoami::username();

    print!("Welcome Back, {}{}{}", YELLOW, user_name, RESET); // Prints Name

    if user_name == system_name {
        println!("");
    } else {
        println!("({}{}{})", CYAN, whoami::username(), RESET);  // Prints Username
    }



    output_uptime();
    println!("{}", system_info());
    // collecting memory sizes (used and total)
    let used_memory: u64 = sys.used_memory();
    let total_memory: u64 = sys.total_memory();
    let percent_used: f64 = used_memory as f64/ total_memory as f64;
    // use correct memory sizes
    let bytes_used = Byte::from_u64(used_memory).get_appropriate_unit(UnitType::Binary);
    let bytes_total = Byte::from_u64(total_memory).get_appropriate_unit(UnitType::Binary);
    // formatting decimal places
    let string_used: String = format!("{bytes_used:.2}");
    let string_total: String = format!("{bytes_total:.2}");
    // printing memory usages
    println!("{} / {} [{:.2}%]", string_used, string_total, percent_used * 100.0);
}
