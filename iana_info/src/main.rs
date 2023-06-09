use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::exit;
use atty::Stream;

const DEFAULT_IANA_FILE_NAME:&str = "language-subtag-registry";
static mut USE_COLORS:bool = false;

fn set_use_colors(use_col:bool) {
    unsafe { USE_COLORS = use_col };
}

fn get_use_colors() -> bool {
    unsafe { return USE_COLORS };
}

fn read_lines(data_path:&Path) -> io::Lines<BufReader<File>> {
    let file = File::open(data_path).unwrap(); 
    return io::BufReader::new(file).lines(); 
}

fn vec_to_map(record: &Vec<String>) -> HashMap<String, String> {
    let mut result:HashMap<String, String> = HashMap::new();
    for line in record {
        let parts = line.split_once(": ");
        let prefix = parts.unwrap().0;
        let suffix = parts.unwrap().1;

        let key = prefix.to_string();
        let old_value = result.get(&key);
        let value: String = if old_value == None {
            suffix.to_string()
        } else {
            old_value.unwrap().to_string() + " ::<sep_tzu>:: " + suffix
        };
        result.insert(key, value);
    }
    return result;
}

fn print_record(section: &Vec<String>, to_matchm: &HashMap<String, String>) {
    let mut found_count = 0;
    let record = vec_to_map(section);
    for (key, value) in to_matchm {
        let z = record.get(key);
        if z != None {
            let searched = z.unwrap().to_uppercase();
            let to_search = value.to_uppercase();
            if to_search.starts_with("=") {
                if searched.eq(&to_search[1..]) {
                    found_count = found_count + 1;
                }
            } else {
                if searched.contains(&to_search) {
                    found_count = found_count + 1;
                }
            }
        }
    }
    if found_count == to_matchm.len() {
        println!("%%");
        for line in section {
            let parts = line.split_once(": ");
            let prefix = parts.unwrap().0;
            let postfix = parts.unwrap().1;
            if get_use_colors() {
                println!("  \x1b[93m{}:\x1b[m {}", prefix, postfix);
            } else {
                println!("  {}: {}", prefix, postfix);
            }
        }
    }
}

fn print_help() {
    println!("Usage: iana_info --key <value> [--key <value>] ...");
    println!("");
    println!("Find info in the IANA Language Subtag Registry");
    println!("Filtering:");
    println!("  -add     --added           <value> // yyyy-MM-dd");
    println!("  -dep     --deprecated      <value> // yyyy-MM-dd");
    println!("  -cmt     --comments        <value>");
    println!("  -d -desc --description     <value>");
    println!("  -mac     --macrolanguage   <value>");
    println!("  -pref    --preferred-value <value>");
    println!("  -px      --prefix          <value>");
    println!("  -scp     --scope           <value>");
    println!("      one of: collection, macrolanguage, private-use, special");
    println!("  -stg     --subtag          <value>");
    println!("  -ss      --suppress-script <value>");
    println!("  -tg      --tag             <value>");
    println!("  -t       --type            <value>");
    println!("      one of: extlang, grandfathered, language, redundant, region, script, variant");
    println!("Shorthands (for type=<option> & subtype / type=<value>):");
    println!("  -el      --extlang         <value>");
    println!("  -gf      --grandfathered   <value>");
    println!("  -l       --language        <value>");
    println!("  -red     --redundant       <value>");
    println!("  -r       --region          <value>");
    println!("  -s       --script          <value>");
    println!("  -v       --variant         <value>");
    println!("Other:");
    println!("  --color=always   : force to always use colors");
    println!("  --color=never    : force to never use colors");
    println!("  -h       --help  : this help");
    println!("where the value can be a substring or exact match if it starts with '='");
    exit(1);
}

fn args_to_map(args: Vec<String>) -> HashMap<String, String> {
    let mut result:HashMap<String, String> = HashMap::new();
    let mut key = "";
    for arg in &args {
        match arg.as_str() {
            // Shorthands
            "-el" | "--extlang"            => { result.insert("Type".to_string(), "=extlang".to_string());       key = "Subtag" },
            "-gf" | "--grandfathered"      => { result.insert("Type".to_string(), "=grandfathered".to_string()); key = "Tag" },
            "-l" | "--language"            => { result.insert("Type".to_string(), "=language".to_string());      key = "Subtag" },
            "-red" | "--redundant"         => { result.insert("Type".to_string(), "=redundant".to_string());     key = "Tag" },
            "-r" | "--region"              => { result.insert("Type".to_string(), "=region".to_string());        key = "Subtag" },
            "-s" | "--script"              => { result.insert("Type".to_string(), "=script".to_string());        key = "Subtag" },
            "-v" | "--variant"             => { result.insert("Type".to_string(), "=variant".to_string());       key = "Subtag" },

            "-add"   | "--added"           => key = "Added",
            "-dep"   | "--deprecated"      => key = "Deprecated",
            "-cmt"   | "--comments"        => key = "Comments",
            "-d" | "-desc" | "--description" => key = "Description",
            "-mac"   | "--macrolanguage"   => key = "Macrolanguage",
            "-pref"  | "--preferred-value" => key = "Preferred-Value",
            "-px"    | "--prefix"          => key = "Prefix",
            "-scp"   | "--scope"           => key = "Scope",
            "-stg"   | "--subtag"          => key = "Subtag",
            "-ss"    | "--suppress-script" => key = "Suppress-Script",
            "-tg"    | "--tag"             => key = "Tag",
            "-t"     | "--type"            => key = "Type",
            "--color=always"               => set_use_colors(true),
            "--color=never"                => set_use_colors(false),
            "-h"     | "--help"            => print_help(),
            _ => {
                if !key.is_empty() {
                    result.insert(key.to_string(), arg.to_string());
                    key = "";
                } else {
                    if arg.starts_with('-') {
                        print_help();
                    }
                }
            },
        };
    }
    // dbg!(&result);
    return result;
}

fn get_data_path() -> PathBuf {

    // let xx = env::current_exe().unwrap().parent().unwrap().join(default_file_name);
    // dbg!(&xx);
    // dbg!(&xx.exists());

    // let xx = env::current_exe().unwrap().parent().unwrap().join("iana").join(default_file_name);
    // dbg!(&xx);
    // dbg!(&xx.exists());

    // let env_path = env::var("IANA_FILE");
    // if env_path.is_ok() {
    //     let wtf = env_path.unwrap();
    //     let xx = Path::new(&wtf);
    //     dbg!(&xx);
    //     dbg!(&xx.exists());
    // }

    return env::current_exe().unwrap()
            .parent().unwrap()
            .join("udata")
            .join(DEFAULT_IANA_FILE_NAME);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let data_path = get_data_path();
    // let data_path = Path::new(&args[0]).parent().unwrap().join("language-subtag-registry");
    // dbg!(&data_path);

    if atty::is(Stream::Stdout) {
        set_use_colors(true);
    } else {
        set_use_colors(false);
    }

    let to_matchm = args_to_map(args);
    if to_matchm.is_empty() {
        print_help();
    }

    // let mut to_matchm:HashMap<String, String> = HashMap::new();
    // to_matchm.insert("Subtag".to_string(), args.get(1).unwrap().to_string());

    let lines = read_lines(&data_path);

    // Iterate over the lines of the file, and in this case print them.
    let mut current_rec: Vec<String> = Vec::with_capacity(8);
    for line in lines {
        let line_content = line.unwrap();
        if line_content.starts_with("File-Date:") {
            println!("{}", line_content);
        } else if line_content.starts_with("%%") {
            print_record(&current_rec, &to_matchm);
            current_rec = Vec::with_capacity(8);
        } else if line_content.starts_with("  ") {
            let long_line = current_rec.pop().unwrap().to_owned() + &line_content[1..];
            current_rec.push(long_line);
        } else {
            current_rec.push(line_content);
        }
    }
    print_record(&current_rec, &to_matchm);
    println!("%%");
    if get_use_colors() {
        println!("\x1b[32mDONE!\x1b[m");
    } else {
        println!("DONE!");
    }
}

/*
    // Added: \d\d\d\d-\d\d-\d\d
    // Deprecated: \d\d\d\d-\d\d-\d\d
    // File-Date: \d\d\d\d-\d\d-\d\d
    // Comments: .+
    // Description: .+
    // Macrolanguage: [a-z]{2,3}
    // Preferred-Value: \S+
    // Prefix: \S+
    // Scope: \S+
    // Subtag: \S+
    // Suppress-Script: \S+
    // Tag: \S+
    // Type: \S+
 */

/*
TODO:
    * find data path: next to exe, in folder next to exe, env variable, current folder, command line option
    * search
        * color the found text
    * Convert IANA file to json?
 */
