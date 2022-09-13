

use chrono::{Weekday, NaiveDate};
use std::convert::TryFrom;
use inquire::{Confirm, Text, Editor, Password, DateSelect, Select, MultiSelect, PasswordDisplayMode};
use clap::Parser;
use std::path::Path;
use std::io;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;

use linked_hash_map::LinkedHashMap;

use yaml_rust::{YamlLoader, YamlEmitter, Yaml};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// hjson configuration string
    #[clap(short, long)]
    config: Option<String>,

    /// hjson configuration file path
    #[clap(short('f'), long)]
    config_file: Option<String>,

    /// answer output file path
    #[clap(short, long)]
    output_answers_file: Option<String>
}

fn main() {
    let args = Args::parse();
    let mut cfgs;
    cfgs = if let Some(c) = &args.config{
        let conf = YamlLoader::load_from_str(&c).unwrap();
        conf[0].clone()
    } 
    // else if let Some(c) = &args.config_file{
    //     // println!("config {}!", args.get_long_help());

    //     println!("config_file {}!", c);

    // } 
    else {
        panic!("Must either be a config or config_file!");
    };
    
    let mut answer_conf = read_answer_conf(&args);

    // dbg!(&answer_conf);

    let answer_array = parse_cli(&args, &cfgs, answer_conf.clone());

    // dbg!(&answer_array);

    write_output(&args, &cfgs, answer_array, answer_conf);
}

#[derive(Debug)]
struct Answer {
    name   : String,
    answer : Vec<String>
}

fn read_answer_conf(args: &Args) -> Yaml {
    let mut answer_conf;
    if let Some(output_answers_file) = args.output_answers_file.clone() {
        let path = Path::new(&output_answers_file);
        let output_answers_dir = path.parent().unwrap();
        if output_answers_dir.exists() && output_answers_dir.is_dir() {
            if path.exists() == false { //create the file
                //set a empty yaml
                File::create(output_answers_file);
                answer_conf = Yaml::Hash(LinkedHashMap::new());
            } else {
                //read/parse the file to yaml
                let file_content = std::fs::read_to_string(output_answers_file).unwrap();
                if file_content.len() == 0 {
                    answer_conf = Yaml::Hash(LinkedHashMap::new())
                }else{
                    answer_conf = YamlLoader::load_from_str(&file_content).unwrap()[0].clone()
                }
            }
        } else {
            panic!("Output answers {} base directory must exist ", output_answers_dir.to_string_lossy());
        }
    } else {
        panic!("output_answers_file option must be defined.");
    }
    answer_conf
}

fn parse_cli(args: &Args, cfgs: &Yaml , mut answer_conf: Yaml)->Vec<Option<Answer>> {

    let mut result: Vec<Option<Answer>> = Vec::new();    
    // dbg!(&cfgs);    
    for cfg in cfgs.as_vec().unwrap() {
        if let Some(output_answers_file) = args.output_answers_file.clone() {
            let path = Path::new(&output_answers_file);
            let output_answers_dir = path.parent().unwrap();
            if output_answers_dir.exists() && output_answers_dir.is_dir() {
                if path.exists() == false { //create the file
                    //set a empty yaml
                    File::create(output_answers_file);
                    answer_conf = Yaml::Hash(LinkedHashMap::new());
                } else {
                    //read/parse the file to yaml
                    let file_content = std::fs::read_to_string(output_answers_file).unwrap();
                    if file_content.len() == 0 {
                        answer_conf = Yaml::Hash(LinkedHashMap::new())
                    }else{
                        answer_conf = YamlLoader::load_from_str(&file_content).unwrap()[0].clone()
                    };
                };
            } else {
                panic!("Output answers {} base directory must exist ", output_answers_dir.to_string_lossy());
            }
        } else {
            panic!("output_answers_file option must be defined.");
        }
        
        let name = if  cfg["name"].is_badvalue() == false {
            if let Some(name) = cfg["name"].as_str(){
                name
            }else{
                panic!("Could not get name attribute value.")
            }
        }else{
            panic!("A name is required for the question to tie this answer to an attribute.");
        };
        
        if cfg["type"].is_badvalue() == false {
            if let Some(t) = cfg["type"].as_str(){
                
                result.push(Some(
                    Answer{
                        answer : match t {
                            "confirm"      => confirm(&args, cfg),
                            "text"         => text(&args, cfg),
                            "editor"       => editor(&args, cfg),
                            "password"     => password(&args, cfg),
                            "date_select"  => date_select(&args,cfg),
                            "select"       => select(&args,cfg),
                            "multi_select" => multi_select(&args,cfg),
                            _              => panic!("Unknown type \"{}\" must be either text, editor, date_select, select, multi_select, confirm, password !", t),
                        },
                        name : name.to_string()
                    }

                ));

                // dbg!(&result);
                let mut res = Vec::new();
                for rz in &result {
                    if let Some(results) = rz{
                        for r in &results.answer{
                            res.push(Yaml::String(r.clone()));
                        }
                    }
                }

            }else{
                panic!("type attribute must be a string!");
            };
        }else{
            panic!("type attribute must be defined!");
        }
        
        }
    return result;
}

// if the answer_conf has a hash key that the answer list does not have then add it 
fn write_output(args:&Args, cfg:&Yaml, answer_list:Vec<Option<Answer>>, answer_conf:Yaml){
    let mut output_conf = answer_conf.clone();

    for some_answer in answer_list {
        if let Some(answer) = some_answer {
            output_conf = if let Yaml::Hash(mut x) = output_conf.clone() {
                let mut a = Vec::new();
                for v in answer.answer{
                    a.push(Yaml::String(v))
                }
                x.insert(Yaml::String(answer.name.clone()), Yaml::Array(a));
                Yaml::Hash(x)
            }else{
                panic!("Should be unreachable");
            };
        }
    }
    
    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.dump(&output_conf).unwrap(); // dump the YAML object to a String

    let mut file = OpenOptions::new()
        .read(false)
        .write(true)
        .create(false)
        .truncate(true)
        .open(args.output_answers_file.clone().unwrap())
        .unwrap();
    
    
    file.write_all(out_str.as_bytes()).unwrap();
    file.sync_all();
    // dbg!(output_conf);
    // dbg!(out_str);

}

fn confirm(args:&Args, cfg:&Yaml)-> Vec<String>{
    let mut inq = if let Some(msg) = cfg["message"].as_str(){
        Confirm::new(msg)
    }else {
        panic!("Error: A message is required to inquired.");
    };


    if cfg["help"].is_badvalue() == false {
        inq = if let Some(x) = cfg["help"].as_str(){
            inq.with_help_message(x)
        }else{
            panic!("help attribute must be a string!");
        };
    }

    if cfg["default"].is_badvalue() == false {
        inq = if let Some(default) = cfg["default"].as_bool(){
            inq.with_default(default)
        }else{
            panic!("default attribute must be a boolean!");
        };
    }

    if cfg["placeholder"].is_badvalue() == false {
        inq = if let Some(placeholder) = cfg["placeholder"].as_str(){
            inq.with_placeholder(placeholder)
        }else{
            panic!("placeholder attribute must be a string!");
        };
    }

    if cfg["skippable"].is_badvalue() == false {
        if let Some(x) = cfg["skippable"].as_bool(){
            if x {
                if let Some(r) = inq.prompt_skippable().unwrap(){
                    let val = if r {
                        "true"
                    } else {
                        "false"
                    };
                    vec![val.to_string()]
                }else{
                    Vec::new()
                }
            }else{
                let val = if inq.prompt().unwrap() {
                    "true"
                } else {
                    "false"
                };
                vec![val.to_string()]
            }
        }else{
            panic!("skippable attribute must be a bool!");
        }
    }else{
        let val = if inq.prompt().unwrap() {
            "true"
        } else {
            "false"
        };
        vec![val.to_string()]
    }
}


fn text(args:&Args, cfg:&Yaml)-> Vec<String>{
    let mut inq = if let Some(msg) = cfg["message"].as_str(){
        Text::new(msg)
    }else {
        panic!("Error: A message is required to inquired.");
    };

    if cfg["help"].is_badvalue() == false {
        inq = if let Some(x) = cfg["help"].as_str(){
            inq.with_help_message(x)
        }else{
            panic!("help attribute must be a string!");
        };
    }

    if cfg["default"].is_badvalue() == false {
        inq = if let Some(x) = cfg["default"].as_str(){
            inq.with_default(x)
        }else{
            panic!("default attribute must be a string!");
        };
    }

    if cfg["page_size"].is_badvalue() == false {
        inq = if let Some(x) = cfg["page_size"].as_i64(){
            inq.with_page_size(x.try_into().unwrap())
        }else{
            panic!("help attribute must be a string!");
        };
    }

    if cfg["initial_value"].is_badvalue() == false {
        inq = if let Some(x) = cfg["initial_value"].as_str(){
            inq.with_initial_value(x)
        }else{
            panic!("help attribute must be a string!");
        };
    }
    let sub;
    if cfg["suggestions"].is_badvalue() == false {
        inq = if let Some(x) = cfg["suggestions"].as_vec(){
            
            let mut suggestions = Vec::new();
            for a in x {
                let z = String::from(a.as_str().unwrap());
                suggestions.push(z);
            }            

            sub = move |val :&str| {
                let sugg = suggestions.clone();
                let val_lower = val.to_lowercase();
                sugg
                    .iter()
                    .filter(|s| s.to_lowercase().contains(&val_lower))
                    .map(|s| String::from(s))
                    .collect()
            };
            inq.with_suggester(&sub)
        }else{
            panic!("help attribute must be a string!");
        };
    }

    if cfg["placeholder"].is_badvalue() == false {
        inq = if let Some(placeholder) = cfg["placeholder"].as_str(){
            inq.with_placeholder(placeholder)
        }else{
            panic!("placeholder attribute must be a string!");
        };
    }

    if cfg["skippable"].is_badvalue() == false {
        if let Some(x) = cfg["skippable"].as_bool(){
            if x {
                if let Some(r) = inq.prompt_skippable().unwrap(){
                    vec![r]
                }else{
                    Vec::new()
                }
            }else{
                vec![inq.prompt().unwrap()]
            }
        }else{
            panic!("skippable attribute must be a bool!");
        }
    }else{
        vec![inq.prompt().unwrap()]
    }

}


fn password(args:&Args, cfg:&Yaml)-> Vec<String>{
    let mut inq = if let Some(msg) = cfg["message"].as_str(){
        Password::new(msg)
    }else {
        panic!("Error: A message is required to inquired.");
    };

    if cfg["help"].is_badvalue() == false {
        inq = if let Some(x) = cfg["help"].as_str(){
            inq.with_help_message(x)
        }else{
            panic!("help attribute must be a string!");
        };
    }

    if cfg["display_mode"].is_badvalue() == false {
        inq = if let Some(x) = cfg["display_mode"].as_str(){
            let mode = match x {
                "hidden" => PasswordDisplayMode::Hidden,
                "masked" => PasswordDisplayMode::Masked,
                "full"   => PasswordDisplayMode::Full,
                _ => panic!("unknown display_mode {} must be wither hidden, masked, full", x)
            };
            inq.with_display_mode(mode)
        }else{
            panic!("display_mode attribute must be a string!");
        };
    }

    if cfg["enable_display_toggle"].is_badvalue() == false {
        inq = if let Some(x) = cfg["enable_display_toggle"].as_bool(){
            if x {
                inq.with_display_toggle_enabled()
            }else{
                inq
            }
        }else{
            panic!("help attribute must be a string!");
        };
    }

    if cfg["skippable"].is_badvalue() == false {
        if let Some(x) = cfg["skippable"].as_bool(){
            if x {
                if let Some(r) = inq.prompt_skippable().unwrap(){
                    vec![r]
                }else{
                    Vec::new()
                }
            }else{
                vec![inq.prompt().unwrap()]
            }
        }else{
            panic!("skippable attribute must be a bool!");
        }

    }else{
        vec![inq.prompt().unwrap()]
    }

}


fn editor(args:&Args, cfg:&Yaml)-> Vec<String>{
    let mut inq = if let Some(msg) = cfg["message"].as_str(){
        Editor::new(msg)
    }else {
        panic!("Error: A message is required to inquired.");
    };

    if cfg["help"].is_badvalue() == false {
        inq = if let Some(x) = cfg["help"].as_str(){
            inq.with_help_message(x)
        }else{
            panic!("help attribute must be a string!");
        };
    }

    if cfg["editor_command"].is_badvalue() == false {
        inq = if let Some(x) = cfg["editor_command"].as_str(){
            inq.with_editor_command(OsStr::new(x))
        }else{
            panic!("editor_command attribute must be a string!");
        };
    }

    let mut cmd_args = Vec::new();
    if cfg["editor_command_args"].is_badvalue() == false {
        inq = if let Some(x) = cfg["editor_command_args"].as_vec(){
            for a in x {
                cmd_args.push(OsStr::new(a.as_str().unwrap()));
            }
            inq.with_args(&cmd_args)
        }else{
            panic!("editor_command_args attribute must be an array!");
        };
    }

    if cfg["file_extension"].is_badvalue() == false {
        inq = if let Some(x) = cfg["file_extension"].as_str(){
            inq.with_file_extension(x)
        }else{
            panic!("help attribute must be a string!");
        };
    }

    if cfg["predefined_text"].is_badvalue() == false {
        inq = if let Some(x) = cfg["predefined_text"].as_str(){
            inq.with_predefined_text(x)
        }else{
            panic!("help attribute must be a string!");
        };
    }

    if cfg["skippable"].is_badvalue() == false {
        if let Some(x) = cfg["skippable"].as_bool(){
            if x {
                if let Some(r) = inq.prompt_skippable().unwrap(){
                    vec![r]
                }else{
                    Vec::new()
                }
            }else{
                vec![inq.prompt().unwrap()]
            }
        }else{
            panic!("skippable attribute must be a bool!");
        }
    }else{
        vec![inq.prompt().unwrap()]
    }

}



fn date_select(args:&Args, cfg:&Yaml)-> Vec<String>{
    let mut inq = if let Some(msg) = cfg["message"].as_str(){
        DateSelect::new(msg)
    }else {
        panic!("Error: A message is required to inquired.");
    };

    if cfg["help"].is_badvalue() == false {
        inq = if let Some(x) = cfg["help"].as_str(){
            inq.with_help_message(x)
        }else{
            panic!("help attribute must be a string!");
        };
    }

    if cfg["default"].is_badvalue() == false {
        inq = if let Some(x) = cfg["default"].as_str(){
            inq.with_default(NaiveDate::parse_from_str(x, "%Y-%m-%d").unwrap())
        }else{
            panic!("default attribute must be a string!");
        };
    }

    if cfg["min_date"].is_badvalue() == false {
        inq = if let Some(x) = cfg["min_date"].as_str(){
            inq.with_min_date(NaiveDate::parse_from_str(x, "%Y-%m-%d").unwrap())
        }else{
            panic!("min_date attribute must be a string!");
        };
    }
    if cfg["max_date"].is_badvalue() == false {
        inq = if let Some(x) = cfg["max_date"].as_str(){
            inq.with_max_date(NaiveDate::parse_from_str(x, "%Y-%m-%d").unwrap())
        }else{
            panic!("max_date attribute must be a string!");
        };
    }

    if cfg["week_start"].is_badvalue() == false {
        inq = if let Some(x) = cfg["week_start"].as_str(){
            let weekday = match x {
                "mon" => Weekday::Mon,
                "tue" => Weekday::Tue,
                "wed" => Weekday::Wed,
                "thu" => Weekday::Thu,
                "fri" => Weekday::Fri,
                "sat" => Weekday::Sat,
                "sun" => Weekday::Sun,
                _ => panic!("unknown week_start {} must be either: mon, tue, wed, thu, fri, sat, sun", x)
            };
            inq.with_week_start(weekday)
        }else{
            panic!("week_start attribute must be a string!");
        };
    }

    // if cfg["skippable"].is_badvalue() == false {
    //     if let Some(x) = cfg["skippable"].as_bool(){
    //         if x {
    //             if let Some(r) = inq.prompt_skippable().unwrap(){
    //                 vec![r.format("%Y-%m-%d").to_string()]
    //             }else{
    //                 Vec::new()
    //             }
    //         }else{
    //             vec![inq.prompt().unwrap().format("%Y-%m-%d").to_string()]
    //         }
    //     }else{
    //         panic!("skippable attribute must be a bool!");
    //     }
    // }else{
    //     vec![&inq.prompt().unwrap().format("%Y-%m-%d").to_string()]
    // }
    vec![inq.prompt().unwrap().format("%Y-%m-%d").to_string()]
}



fn select(args:&Args, cfg:&Yaml)-> Vec<String>{
    let mut inq = if let Some(msg) = cfg["message"].as_str(){
        let mut options = Vec::new();
        if cfg["options"].is_badvalue() == false {
            if let Some(x) = cfg["options"].as_vec(){
                for a in x {
                    options.push(a.as_str().unwrap());
                }
                Select::new(msg, options)
            }else{
                panic!("help attribute must be a string!");
            }
        }else{
            panic!("options are required!");
        }
    }else {
        panic!("Error: A message is required to inquired.");
    };

    if cfg["help"].is_badvalue() == false {
        inq = if let Some(x) = cfg["help"].as_str(){
            inq.with_help_message(x)
        }else{
            panic!("help attribute must be a string!");
        };
    }

    if cfg["page_size"].is_badvalue() == false {
        inq = if let Some(x) = cfg["page_size"].as_i64(){
            inq.with_page_size(x.try_into().unwrap())
        }else{
            panic!("page_size attribute must be a i64!");
        };
    }


    if cfg["starting_cursor"].is_badvalue() == false {
        inq = if let Some(x) = cfg["starting_cursor"].as_i64(){
            inq.with_starting_cursor(x.try_into().unwrap())
        }else{
            panic!("startings_cursor attribute must be a i64!");
        };
    }

    if cfg["skippable"].is_badvalue() == false {
        if let Some(x) = cfg["skippable"].as_bool(){
            if x {
                if let Some(r) = inq.prompt_skippable().unwrap(){
                    vec![r.to_string()]
                }else{
                    Vec::new()
                }
            }else{
                vec![inq.prompt().unwrap().to_string()]
            }
        }else{
            panic!("skippable attribute must be a bool!");
        }
    }else{
        vec![inq.prompt().unwrap().to_string()]
    }
}

fn multi_select(args:&Args, cfg:&Yaml)-> Vec<String>{
    let mut inq = if let Some(msg) = cfg["message"].as_str(){
        let mut options = Vec::new();
        if cfg["options"].is_badvalue() == false {
            if let Some(x) = cfg["options"].as_vec(){
                for a in x {
                    options.push(a.as_str().unwrap());
                }
                MultiSelect::new(msg, options)
            }else{
                panic!("help attribute must be a string!");
            }
        }else{
            panic!("options are required!");
        }
    }else {
        panic!("Error: A message is required to inquired.");
    };

    let mut defaults = Vec::new();
    if cfg["default"].is_badvalue() == false {
        inq = if let Some(x) = cfg["default"].as_vec(){
            for a in x {
                defaults.push(usize::try_from(a.as_i64().unwrap()).unwrap());
            }
            inq.with_default(&defaults)
        }else{
            panic!("default attribute must be a string!");
        };
    }

    if cfg["help"].is_badvalue() == false {
        inq = if let Some(x) = cfg["help"].as_str(){
            inq.with_help_message(x)
        }else{
            panic!("help attribute must be a string!");
        };
    }

    if cfg["page_size"].is_badvalue() == false {
        inq = if let Some(x) = cfg["page_size"].as_i64(){
            inq.with_page_size(x.try_into().unwrap())
        }else{
            panic!("page_size attribute must be a i64!");
        };
    }


    if cfg["starting_cursor"].is_badvalue() == false {
        inq = if let Some(x) = cfg["starting_cursor"].as_i64(){
            inq.with_starting_cursor(x.try_into().unwrap())
        }else{
            panic!("startings_cursor attribute must be a i64!");
        };
    }

        if cfg["keep_filter"].is_badvalue() == false {
        inq = if let Some(x) = cfg["keep_filter"].as_bool(){
            inq.with_keep_filter(x.try_into().unwrap())
        }else{
            panic!("keep_filter attribute must be a bool!");
        };
    }

    if cfg["skippable"].is_badvalue() == false {
        if let Some(x) = cfg["skippable"].as_bool(){
            if x {
                if let Some(r) = inq.prompt_skippable().unwrap(){
                    let mut v = Vec::new();
                    for x in r{
                        v.push(x.to_string());
                    }
                    v
                    
                }else{
                    Vec::new()
                }
            }else{
                let mut v = Vec::new();
                for x in inq.prompt().unwrap(){
                    v.push(x.to_string());
                }
                v
            }
        }else{
            panic!("skippable attribute must be a bool!");
        }
    }else{
        let mut v = Vec::new();
        for x in inq.prompt().unwrap(){
            v.push(x.to_string());
        }
        v
    }

    
}


/*

RUST_BACKTRACE=1 cargo run -- -o /home/flopes/answers.yml -c '[{"name":"test", "type":"confirm", "message":"Are you from Mars?"}]'
RUST_BACKTRACE=1 cargo run -- -o /home/flopes/answers.yml -c '[{"name":"test", "type":"confirm", "message":"Are you from Mars?","help":"Some help message"}]'
RUST_BACKTRACE=1 cargo run -- -o /home/flopes/answers.yml -c '[{"name":"test", "type":"confirm", "message":"Are you from Mars?","help":"Some help message", "placeholder": "some placeholder"}]'
RUST_BACKTRACE=1 cargo run -- -o /home/flopes/answers.yml -c '[{"name":"test", "type":"confirm", "message":"Are you from Mars?","help":"Some help message", }]'

RUST_BACKTRACE=1 cargo run -- -o /home/flopes/answers.yml -c '[{"name":"test", "type":"text",    "message":"Where are you from?", "suggestions":["Colombia", "Brazil", "Argentina", "USA"] }]'

RUST_BACKTRACE=1 cargo run -- -o /home/flopes/answers.yml -c '[{"name":"test", "type":"editor",    "message":"Where are you from?", "predefined_text" : "Some predefined text", "editor_command":"vim" }]'

RUST_BACKTRACE=1 cargo run -- -o /home/flopes/answers.yml -c '[{"name":"test", "type":"password",    "message":"Please type the secret password?", "help" : "extra_help", "display_toggle":false, "display_mode":"masked" }]'

RUST_BACKTRACE=1 cargo run -- -o /home/flopes/answers.yml -c '[{"name":"test", "type":"date_select",    "message":"Please pick a date for the flight", "help" : "extra_help", "week_start":"mon", "min_date":"2022-5-17" }]'

RUST_BACKTRACE=1 cargo run -- -o /home/flopes/answers.yml -c '[{"name":"test", "type":"select",    "message":"Please pick a some food", "help" : "extra_help", "options":["pasta", "pizza", "meat_balls"] }]'
RUST_BACKTRACE=1 cargo run -- -o /home/flopes/answers.yml -c '[{"name":"test", "type":"select",    "message":"Please pick a some food \n multiple lines", "help" : "extra_help", "options":["pasta", "pizza", "meat_balls"] }]'

RUST_BACKTRACE=1 cargo run -- -o /home/flopes/answers.yml -c '[{"name":"test", "type":"multi_select",    "message":"Please pick a some food", "help" : "extra_help", "options":["pasta", "pizza", "meat_balls"], }]'
RUST_BACKTRACE=1 cargo run -- -o /home/flopes/answers.yml -c '[
    {"name":"test", "type":"multi_select",    "message":"Please pick a some food", "help" : "extra_help", "options":["pasta", "pizza", "meat_balls"], },
    {"name":"test2", "type":"multi_select",    "message":"Please pick a some drink", "help" : "extra_help", "options":["water", "pepsi", "cola"], }
    ]'




inquire --config='{
    "type":"Text" # Text | Editor | DateSelect | Select | MultiSelect | Confirm | CustomType | Password
    "message": "What is your name",
    "render": ?,
    "default_values": [],
    "placeholders": [],  # placeholder, text, confirm
    "validators": [
        {
        "function" : "answers_count_gte",
        "params"   : [1]
        }
    ],
    "formatters": [],
    "help": "",
    "auto_complete": [],
    "filter_function": ??



    # ====== EXCLUSIVE TO Select & MultiSelect ========
    "page_size" : 7, 
    "options" : [""],
    "starting_cursor" : 0,
    "display_option_indicies" : false
    

    # ====== EXCLUSIVE TO MULTISELECT ========
    "default_selection" : [""],
    "starting_cursor"   : 0,
    "keep_filter_flag"  : true,

    # ====== EXCLUSIVE TO DATESELECT ========
    "min_date" : ""  #FOR DATESELECT
    "max_date" : ""  #FOR DATESELECT
    "week_start" : "" # mon | tue | wed | thu | fri | sat | sun

    # ====== EXCLUSIVE TO EDITOR ========
    "editor_args" : ["nano"],
    "file_extension" : "",
    "predefined_text": "",


    # ====== EXCLUSIVE TO PASSWORD ========
    display_mode: "Hidden", # Hidden|Masked|Full
    mask_character: "*",
    toggle_display: false,


    validators:[{
        type: string      
        sub_type: regex_match | min_len | max_len | file_exist | dir_exists | dir_of_file_exists
        value: 1
    }
    ]
    validators:[{
        type: select
        sub_type: min_selection_count | max_selection_count
    }]



}'



*/

