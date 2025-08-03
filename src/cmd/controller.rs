use crate::cmd::file;
use clap::{Arg, ArgAction, ArgMatches, Command};
use minijinja::Environment;
use quote::{quote};
use serde::Serialize;
use std::collections::HashMap;
use std::{fs, path::Path};
use syn::{parse_file, parse_str, File, Item, ItemFn, ItemMod, Stmt};

#[allow(dead_code)]
pub(crate) fn command() -> Command {
    Command::new("controller")
        .short_flag('c')
        .long_flag("controller")
        .about("controller.")
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
                .required(true)
                .short('n')
                .long("name")
                .help("name"),
        )
        .arg(Arg::new("tag").short('t').long("tag").help("tag"))
}

#[derive(Serialize, Debug)]
struct ControllerStub {
    pub source_file: String,
    pub scope: String,
    pub tag_prefix: String,
}

#[allow(dead_code)]
pub(crate) fn execute(arg_matches: &ArgMatches) {
    let out_file = crate::cmd::r#gen::out_file(arg_matches);
    let import_mod_file = out_file.clone() + "/mod.rs";
    let name: String = crate::cmd::r#gen::name(arg_matches);
    let source_file = out_file.clone() + "/" + name.as_str() + ".rs";

    let tag: String = crate::cmd::r#gen::tag(arg_matches);

    let mut tag_prefix = tag.clone();

    if tag_prefix.is_empty() {
        let binding = out_file.clone();
        let binding = binding.as_str();
        let path = Path::new(&binding);
        tag_prefix = format!("{}::{}", path.file_name().unwrap().display(), name);
    }

    let stub = ControllerStub {
        source_file: source_file.clone(),
        scope: name.clone(),
        tag_prefix: tag_prefix.clone(),
    };

    let binding = file!();
    let path = Path::new(&binding);
    let stub_path = format!("{}", path.parent().unwrap().display());
    let stub_template = fs::read_to_string(stub_path + "/../stubs/controller.stub").unwrap();

    let mut env = Environment::new();
    env.add_template("stub", stub_template.as_str()).unwrap();
    let template = env.get_template("stub").unwrap();

    let mod_content = template.render(stub).unwrap();

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

    if !std::path::Path::new(source_file.clone().as_str()).exists() {
        file::create_file(source_file.as_str());
        // 写文件
        file::write_file(source_file.as_str(), mod_content.as_str());
    } else {
        println!("{}已经存在", source_file.clone());
        return;
    }

    // mod导入
    if !std::path::Path::new(import_mod_file.clone().as_str()).exists() {
        file::create_file(import_mod_file.clone().as_str());
    }
    let content = fs::read_to_string(import_mod_file.clone().as_str())
        .expect(format!("读取{}文件失败", import_mod_file.clone()).as_str());

    let mut new_mod_content = String::new();
    let mut is_new_mod_ok = false;

    let line_break = String::from("\n");

    // 解析文件内容为 AST
    let syntax: File = parse_file(&content).expect("Not valid Rust code");
    for item in syntax.items {
        if let Item::Mod(ItemMod { .. }) = item {
            if !is_new_mod_ok {
                let item_str = format!("pub mod {};", name.clone());
                new_mod_content.push_str(item_str.as_str());
                new_mod_content.push_str(line_break.clone().as_str());
                let item_str = quote! {#item}.to_string();
                new_mod_content.push_str(item_str.as_str());
                is_new_mod_ok = true;
            } else {
                let item_str = quote! {#item}.to_string();
                new_mod_content.push_str(item_str.as_str());
            }
        } else if let Item::Fn(ItemFn {
            ref vis,
            ref sig,
            ref attrs,
            ref block,
        }) = item
        {
            let fn_name = &sig.ident;
            // 寻找configure函数
            if fn_name.eq("configure") {
                // 函数tokens
                //let fn_sig_str = quote! {#sig}.to_string();
                // 提取函数参数
                let args_map = parse_extract_args(quote! {#sig});
                // 参数名
                let arg_name = args_map.values().next().unwrap();

                // 抽取函数体语句
                let statements = block.stmts.clone();

                let mut i = 0;
                let statements_len = &statements.len();

                let mut new_statements: Vec<Stmt> = vec![];

                // 组装scope
                for statement in &statements {
                    new_statements.push(statement.clone());
                    if i == statements_len - 2 {
                        // 将字符串转换成Stmt
                        let stmt = parse_str::<Stmt>(
                            format!(
                                "let {} = {}.configure({}::routes);",
                                arg_name.clone(),
                                arg_name.clone(),
                                name.clone()
                            )
                            .as_str(),
                        )
                        .unwrap();

                        new_statements.push(stmt.clone());
                    }
                    i += 1;
                }

                // 使用解析输入重构函数，然后输出
                let new_configure_fn_str = quote! {
                    // 在该函数上重复其他所有属性（保持不变）
                    #(#attrs)*
                    // 重构函数声明
                    #vis #sig {
                        #(#new_statements)*
                    }
                }
                .to_string();
                println!("{}", &new_configure_fn_str);
                new_mod_content.push_str(new_configure_fn_str.as_str());
            } else {
                // 其他函数
                let item_str = quote! {#item}.to_string();
                new_mod_content.push_str(item_str.as_str());
            }
        } else {
            let item_str = quote! {#item}.to_string();
            new_mod_content.push_str(item_str.as_str());
        }
        new_mod_content.push_str(line_break.clone().as_str());
    }

    //let syntax: File = parse_file(&new_mod_content).expect("Not valid Rust code");
    //eprintln!("new_syntax_content：{:#?}", quote! {#syntax}.to_string());
    // 写文件
    //file::write_file(import_mod_file.clone().as_str(), quote! {#syntax}.to_string().as_str());
    file::write_file(import_mod_file.clone().as_str(), &new_mod_content.as_str());
}

#[allow(unused)]
pub fn parse_extract_args(tokens: proc_macro2::TokenStream) -> HashMap<String, String> {
    let mut args_map = HashMap::<String, String>::new();
    for token in tokens.into_iter() {
        match token {
            // 遍历TokenTree::Group下的TokenStream
            proc_macro2::TokenTree::Group(ref group) => {
                let mut key = String::new();
                let mut value = String::new();
                let mut punctuation = String::new();
                let mut punctuation_counter = 0;

                // 获取组内的TokenStream并再次遍历
                let inner_tokens = group.stream();
                for inner_tt in inner_tokens {
                    match inner_tt {
                        // ref 模式 https://rustwiki.org/zh-CN/rust-by-example/scope/borrow/ref.html
                        proc_macro2::TokenTree::Ident(ref ident) => {
                            if punctuation.is_empty() {
                                value = ident.clone().to_string();
                            } else {
                                if punctuation_counter >= 1 {
                                    key = key + &*ident.clone().to_string();
                                }
                            }
                        }
                        proc_macro2::TokenTree::Punct(ref punct) => {
                            if punct.to_string() == ":" {
                                punctuation_counter += 1;
                                punctuation = punct.clone().to_string();
                                if punctuation_counter > 1 {
                                    key = key + &*punct.clone().to_string();
                                }
                            } else if punct.to_string() == "," {
                                args_map.insert(key.clone(), value.clone());
                                key = String::new();
                                value = String::new();
                                punctuation = String::new();
                                punctuation_counter = 0;
                            } else {
                                key = key + &*punct.clone().to_string();
                            }
                        }
                        // 可以根据需要处理更多类型...
                        _ => (), // 处理其他类型或忽略
                    }
                }
                if !key.is_empty() && !value.is_empty() {
                    args_map.insert(key.clone(), value.clone());
                }
            }
            // 处理其他类型的TokenTree...
            _ => (), // 或者忽略非Group类型的TokenTree
        }
    }
    args_map
}
