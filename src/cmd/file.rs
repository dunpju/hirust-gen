use std::fs::OpenOptions;
use std::io::Write;

#[allow(dead_code)]
pub fn create_file(file_path: &str) {
    // 检查文件是否存在
    if std::path::Path::new(file_path).exists() {
        println!("文件已存在：{}", file_path);
    } else {
        // 文件不存在，尝试创建文件
        match std::fs::File::create(file_path) {
            Ok(_) => println!("文件创建成功：{}", file_path),
            Err(e) => println!("创建文件失败：{}", e),
        }
    }
}

#[allow(dead_code)]
pub fn create_and_append(file_path: &str, content: &str) {
    // 创建文件
    create_file(file_path);

    // 打开文件并追加内容
    match OpenOptions::new().append(true).open(file_path) {
        Ok(mut file) => {
            // 追加内容
            if let Err(e) = writeln!(file, "{}", content) {
                println!("追加内容失败：{}", e);
            }
        }
        Err(e) => println!("打开文件失败：{}", e),
    }
}

#[allow(dead_code)]
pub fn write_file(file_path: &str, content: &str) {
    match OpenOptions::new().write(true).open(file_path) {
        Ok(mut file) => {
            if let Err(e) = writeln!(file, "{}", content) {
                println!("写文件内容失败：{}", e);
            }
        }
        Err(e) => println!("打开文件失败：{}", e),
    }
}