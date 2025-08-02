use std::collections::BTreeMap;
use std::fs;
use clap::{Arg, ArgAction, ArgMatches, Command};
use minijinja::Environment;
use regex::Regex;
use serde::Serialize;
use walkdir::WalkDir;
use crate::cmd::r#gen::out_file;
use crate::cmd::file;

#[allow(dead_code)]
pub(crate) fn command() -> Command {
    Command::new("code")
        .short_flag('C')
        .long_flag("code")
        .about("code.")
        .arg(Arg::new("out")
                 .required(true)
                 .short('o')
                 .long("out")
                 .action(ArgAction::Set)
                 .help("Output file"),
        )
        .arg(Arg::new("source")
                 .required(true)
                 .short('s')
                 .long("source")
                 .help("Source Directory"),
        )
}

#[derive(Serialize)]
struct CodeStub {
    pub doc: String,
    pub rows: Vec<Code>,
}

#[derive(Serialize)]
struct Code {
    pub name: String,
    pub code: i64,
    pub message: String,
}

#[allow(dead_code)]
pub(crate) fn execute(arg_matches: &ArgMatches) {
    let mut code_mod_content = String::from("use resp::ErrorCode;\n");
    code_mod_content.push("\n".parse().unwrap());

    let out_file = out_file(arg_matches);
    let mut source_dir = String::new();
    if let Some(source) = arg_matches.get_many::<String>("source") {
        source_dir = source.map(|s| s.as_str()).collect::<Vec<_>>().join("");
        println!("{:?}", source_dir);
    }

    let mut rows: Vec<Code> = vec![];
    for entry in WalkDir::new(source_dir) {
        let entry = entry.unwrap();
        let file_path = entry.path();
        let extension = file_path.extension().and_then(|s| s.to_str());

        // 检查扩展名是否为".yaml"
        if file_path.is_file() && extension == Some("yaml") {
            let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap();
            //println!("{}", file_name);
            let re = Regex::new(r"\d+").unwrap(); // 编译正则表达式，\d+ 匹配一个或多个数字
            let file_number = re.captures(file_name).unwrap(); // 提取所有匹配的数字

            //println!("找到的数字: {}", file_number.get(0).map_or("", |m| m.as_str()));

            let mut iota: i64 = file_number.get(0).map_or("", |m| m.as_str()).parse().unwrap();

            let yaml = fs::read_to_string(file_path).expect(format!("读取{}失败", file_name).as_str());

            let mut key_sort: Vec<String> = vec![];
            for row in yaml.as_str().lines() {
                let re = Regex::new(r"^[a-zA-Z]").unwrap();
                if re.is_match(row) {
                    key_sort.push(row.trim().replace(":", ""));
                }
            }
            let deserialized_map: BTreeMap<String, serde_yml::Value> = serde_yml::from_str(&yaml).unwrap();
            for key in key_sort {
                let value = deserialized_map.get(key.as_str());
                println!("{} k: {:?}, v: {:?}", iota, key, value);
                let mut message = key.clone().as_str().to_owned() + "错误";
                match value {
                    Some(mapping) => {
                        match mapping.as_mapping() {
                            Some(mapping) => {
                                if mapping.contains_key("message") {
                                    message = mapping.get("message").unwrap().as_str().unwrap().parse().unwrap();
                                }
                                if mapping.contains_key("code") {
                                    match mapping.get("code").unwrap().as_i64() {
                                        Some(code) => {
                                            iota = code;
                                            println!("{:?}", iota);
                                        }
                                        None => {}
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                    None => {}
                }

                rows.push(Code { name: key.to_uppercase(), code: iota, message });
                iota += 1;
            }
        }
    }

    let doc= String::from("错误码");
    let stub = CodeStub {
        doc,
        rows,
    };

    let stub_template = fs::read_to_string("./resources/stubs/code.stub").unwrap();

    let mut env = Environment::new();
    env.add_template("stub", stub_template.as_str()).unwrap();
    let template = env.get_template("stub").unwrap();

    let mod_content = template.render(stub).unwrap();

    println!("{}", mod_content);
    // 写文件
    file::write_file(out_file.as_str(), mod_content.as_str());
}