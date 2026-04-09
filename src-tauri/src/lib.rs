use md5::{Md5, Digest};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

const SALT: &str = "GSCM_SAULT";

/// 将MD5字节数组转为十进制字符串（与Java BigInteger.toString()一致）
fn md5_to_decimal_string(bytes: &[u8]) -> String {
    // MD5 结果是 16 字节，转为大整数后输出十进制字符串
    // 与 Java: new BigInteger(1, md.digest()).toString() 行为一致
    let mut result: u128 = 0;
    for &byte in bytes {
        result = result * 256 + (byte as u128);
    }
    result.to_string()
}

/// 计算字符串的MD5值（加盐）
fn md5_string(content: &str) -> String {
    if content.is_empty() {
        return String::new();
    }
    let salted = format!("{}{}", content, SALT);
    let mut hasher = Md5::new();
    hasher.update(salted.as_bytes());
    let result = hasher.finalize();
    // 转换为十进制字符串（与Java BigInteger.toString()一致）
    md5_to_decimal_string(result.as_slice())
}

/// 计算文件内容的MD5值
/// 移除第一行已有的MD5注释行，然后计算剩余内容的MD5
fn calculate_file_md5(file_path: &Path) -> Result<String, String> {
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let content = fs::read_to_string(file_path).map_err(|e| e.to_string())?;

    // 移除第一行已有的MD5注释
    let lines: Vec<&str> = content.lines().collect();
    let content_without_md5 = if !lines.is_empty()
        && (lines[0].starts_with("-- MD5:") || lines[0].starts_with("//MD5:"))
    {
        // 移除第一行，保留其余内容
        // 根据文件类型处理
        if file_name.contains(".groovy") {
            // groovy文件：最后一行不加\r\n
            let mut sb = String::new();
            for (i, line) in lines[1..].iter().enumerate() {
                sb.push_str(line);
                if i < lines[1..].len() - 1 {
                    sb.push_str("\r\n");
                }
            }
            sb
        } else if file_name.contains(".sql") {
            // sql文件：每行都加\r\n（包括最后一行）
            let mut sb = String::new();
            for line in lines[1..].iter() {
                sb.push_str(line);
                sb.push_str("\r\n");
            }
            sb
        } else {
            lines[1..].join("\r\n")
        }
    } else {
        // 根据文件类型处理
        if file_name.contains(".groovy") {
            // groovy文件：最后一行不加\r\n
            let mut sb = String::new();
            for (i, line) in lines.iter().enumerate() {
                sb.push_str(line);
                if i < lines.len() - 1 {
                    sb.push_str("\r\n");
                }
            }
            sb
        } else if file_name.contains(".sql") {
            // sql文件：每行都加\r\n（包括最后一行）
            let mut sb = String::new();
            for line in lines.iter() {
                sb.push_str(line);
                sb.push_str("\r\n");
            }
            sb
        } else {
            content.clone()
        }
    };

    Ok(md5_string(&content_without_md5))
}

/// 将MD5值写入文件第一行
fn write_md5_to_file(file_path: &Path, md5_value: &str) -> Result<(), String> {
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    // 根据文件类型确定前缀
    let prefix = if file_name.ends_with(".sql") {
        "-- MD5:"
    } else if file_name.ends_with(".groovy") {
        "//MD5:"
    } else {
        "-- MD5:" // 默认使用SQL格式
    };

    // 读取文件内容
    let content = fs::read(file_path).map_err(|e| e.to_string())?;

    // 计算起始位置（如果第一行已有MD5，则跳过）
    let start_position = if !content.is_empty() {
        // 尝试读取第一行
        let cursor = std::io::Cursor::new(&content[..]);
        let mut reader = BufReader::new(cursor);
        let mut first_line = String::new();
        if reader.read_line(&mut first_line).is_ok() {
            let line = first_line.trim_end_matches('\n').trim_end_matches('\r');
            if line.starts_with("-- MD5:") || line.starts_with("//MD5:") {
                // 需要跳过第一行（包括换行符）
                first_line.len()
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    // 构建新内容
    let md5_line = format!("{}{}\n", prefix, md5_value);
    let md5_bytes = md5_line.as_bytes();

    let mut final_content = Vec::with_capacity(md5_bytes.len() + content.len() - start_position);
    final_content.extend_from_slice(md5_bytes);
    final_content.extend_from_slice(&content[start_position..]);

    fs::write(file_path, final_content).map_err(|e| e.to_string())?;

    Ok(())
}

/// 为文件添加MD5注释
#[tauri::command]
fn add_md5_to_file(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);

    // 计算MD5
    let md5_value = calculate_file_md5(path)?;

    // 写入文件
    write_md5_to_file(path, &md5_value)?;

    Ok(md5_value)
}

/// 仅计算文件的MD5值（不修改文件）
#[tauri::command]
fn calculate_md5(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);
    calculate_file_md5(path)
}

/// 计算字符串的MD5值
#[tauri::command]
fn md5_text(text: String) -> String {
    md5_string(&text)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            add_md5_to_file,
            calculate_md5,
            md5_text
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}