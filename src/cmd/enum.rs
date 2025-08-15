use crate::cmd::file;
use clap::{Arg, ArgAction, ArgMatches, Command};
use minijinja::Environment;
use regex::Regex;
use serde::Serialize;
use std::{fs, path::Path};

// cargo run gen code --out="./errcode/mod.rs" --source="./yaml"
// cargo run gen enum --out="./enums" --name="-e=order_flow -f=发起订单:selling_assistant-1-销售内勤,sale-2-销售"
// cargo run gen enum --out="./enums" --file="./src/cmd/enum.md"
#[allow(dead_code)]
pub(crate) fn command() -> Command {
    Command::new("enum")
        .short_flag('E')
        .long_flag("enum")
        .about("enum.")
        .arg(
            Arg::new("out")
                .required(true)
                .short('o')
                .long("out")
                .action(ArgAction::Set)
                .help("Output Directory"),
        )
        .arg(
            Arg::new("name")
                .conflicts_with("file")
                .short('n')
                .long("name")
                .help("name"),
        )
        .arg(
            Arg::new("file")
                .conflicts_with("name")
                .short('f')
                .long("file")
                .help("file"),
        )
}

#[derive(Serialize)]
struct EnumStub {
    pub doc: String,
    pub rows: Vec<Enum>,
}

#[derive(Serialize)]
struct Enum {
    pub name: String,
    pub code: i32,
    pub message: String,
}
#[allow(dead_code)]
pub(crate) fn execute(arg_matches: &ArgMatches) {
    let out_file = crate::cmd::r#gen::out_file(arg_matches);
    let import_mod_file = out_file.clone() + "/mod.rs";

    let file = crate::cmd::r#gen::file(arg_matches);
    if !file.is_empty() {
        let content = fs::read_to_string(file.clone().as_str())
            .expect(format!("读取{}文件失败", import_mod_file.clone()).as_str());
        for line in content.lines() {
            let pattern = format!("--name={}-e=(.*)+-f=(.*){}", '"', '"');
            let re = Regex::new(pattern.as_str()).unwrap();
            match re.captures(line) {
                Some(c) => {
                    let mut name: String = c.get(0).map_or("", |m| m.as_str()).parse().unwrap();
                    name = name.replace("--name=\"", "");
                    name = name.replace("\"", "");
                    single(out_file.clone(), import_mod_file.clone(), name.clone())
                }
                None => {}
            }
        }
    } else {
        let name: String = crate::cmd::r#gen::name(arg_matches);

        single(out_file.clone(), import_mod_file.clone(), name)
    }
}

fn single(mut out_file: String, import_mod_file: String, mut name: String) {
    name = name.replace("\\", "");
    name = name.replace("\n", "");
    name = name.replace("\r", "");
    name = name.trim().parse().unwrap();

    let re = Regex::new(r"(-e=[a-zA-Z_]+\s*-f=).*").unwrap();
    if !re.is_match(&*name) {
        panic!("regexp err")
    }
    name = name.replace("-e=", "");
    let names: Vec<String> = name.split("-f=").map(|x| x.to_string()).collect();
    if names.len() != 2 {
        panic!("name err")
    }
    let crate_dir: String = names[0].trim().parse().unwrap();

    let docs: Vec<String> = names[1].split(":").map(|x| x.to_string()).collect();
    let doc: String = docs[0].trim().parse().unwrap();

    let mut rows: Vec<Enum> = vec![];
    let es: Vec<String> = docs[1].split(",").map(|x| x.to_string()).collect();
    for e in es {
        let em: Vec<String> = e.split("-").map(|x| x.to_string()).collect();
        let name: String = em[0].trim().to_string();
        let code: i32 = em[1].trim().parse().unwrap();
        let message: String = em[2].replace("\n", "").replace("\r", "").trim().to_string();
        rows.push(Enum {
            name: name.to_uppercase(),
            code,
            message,
        });
    }

    let stub = EnumStub { doc, rows };

    let binding = file!();
    let path = Path::new(&binding);
    let stub_path = format!("{}", path.parent().unwrap().display());

    let stub_template = fs::read_to_string(stub_path + "/../stubs/enum.stub").unwrap();

    let mut env = Environment::new();
    env.add_template("stub", stub_template.as_str()).unwrap();
    let template = env.get_template("stub").unwrap();

    let mod_content = template.render(stub).unwrap();

    out_file = out_file + "/" + crate_dir.clone().as_str();

    let out_file_dir_list: Vec<String> =
        out_file.clone().split("/").map(|x| x.to_string()).collect();
    let mut out_file_dir_temp = String::new();
    for out_file_dir in out_file_dir_list {
        out_file_dir_temp =
            String::from(out_file_dir_temp.to_owned() + out_file_dir.as_str() + "/");
        if !std::path::Path::new(out_file_dir_temp.clone().as_str()).exists() {
            fs::create_dir(out_file.as_str())
                .expect(format!("创建{}目录失败", out_file_dir_temp.clone()).as_str());
        }
    }

    out_file = out_file + "/mod.rs";
    if !std::path::Path::new(out_file.clone().as_str()).exists() {
        file::create_file(out_file.as_str());
        // 写文件
        file::write_file(out_file.as_str(), mod_content.as_str());
    } else {
        println!("{}已经存在", out_file.clone());
    }

    // mod导入
    if !std::path::Path::new(import_mod_file.clone().as_str()).exists() {
        file::create_file(import_mod_file.clone().as_str());
    }
    let content = fs::read_to_string(import_mod_file.clone().as_str())
        .expect(format!("读取{}文件失败", import_mod_file.clone()).as_str());
    if !content.contains(format!("pub mod {};", crate_dir.clone()).as_str()) {
        // 写文件
        file::write_file(
            import_mod_file.as_str(),
            format!(
                "{}\n{}",
                content,
                format!("pub mod {};", crate_dir.clone()).as_str()
            )
            .as_str(),
        );
    }
}
