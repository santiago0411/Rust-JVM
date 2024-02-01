mod utils;
mod class_file;
mod code;

use utils::*;

use std::io::{Cursor};
use crate::class_file::*;
use crate::code::*;

fn main() {
    let file_path = "etc/HelloWorld.class";
    let mut cursor: Cursor<Vec<u8>>;

    match read_file_to_buf(file_path) {
        Err(err) => {
            eprintln!("Error reading file {}: {}", file_path, err);
            return;
        }
        Ok(buffer) => {
            cursor = Cursor::new(buffer);
        }
    }

    let class_file: Box<ClassFile>;
    match ClassFile::new(&mut cursor) {
        Err(err) => {
            eprintln!("Error creating ClassFile: {}", err);
            return;
        }
        Ok(cf) => class_file = cf
    }

    if let Some(main_method) = class_file.find_method_by_name("main") {
        if let Some(att) = class_file.find_attribute_by_name(&main_method.attributes, "Code") {
            if let Ok(code_att) = CodeAttribute::new(att) {
                let _ = execute_code(&class_file, code_att.code);
            }
        }
    }
}
