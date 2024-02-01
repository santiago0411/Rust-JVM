use std::collections::VecDeque;
use std::io;
use std::io::{Cursor, Read};
use crate::class_file::*;
use crate::utils::{read_attributes, ReadFromCursor};

pub const OP_CODE_GET_STATIC: u8 = 0xB2;
pub const OP_CODE_LDC: u8 = 0x12;
pub const OP_CODE_INVOKE_VIRTUAL: u8 = 0xB6;
pub const OP_CODE_BI_PUSH: u8 = 0x10;
pub const OP_CODE_SI_PUSH: u8 = 0x11;
pub const OP_CODE_RETURN: u8 = 0xB1;

pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exception_table: Vec<u8>, // NOT PARSED
    pub attributes: Vec<AttributeInfo>
}

impl CodeAttribute {
    pub fn new(attribute: &AttributeInfo) -> io::Result<Box<CodeAttribute>> {
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(attribute.info.clone());
        let max_stack = cursor.read_u16()?;
        let max_locals = cursor.read_u16()?;
        let code_length = cursor.read_u32()?;
        let code: Vec<u8> = cursor.read_bytes(code_length as usize)?;
        let ex_table_length = cursor.read_u16()?;
        let exception_table = cursor.read_bytes(ex_table_length as usize)?;
        let att_count = cursor.read_u16()?;
        let attributes: Vec<AttributeInfo> = read_attributes(cursor.by_ref(), att_count as usize)?;

        return Ok(Box::new(CodeAttribute {
            max_stack,
            max_locals,
            code,
            exception_table,
            attributes
        }))
    }
}

enum Instruction {
    Type(String),
    String(String),
    SByte(i8),
    Short(i16),
    Integer(i32),
    Float(f32)
}

fn get_name_of_class(class_file: &ClassFile, class_index: u16) -> &str {
    if let Some(Constant::Class(class)) = class_file.constants_pool.get((class_index - 1) as usize) {
         if let Some(Constant::Utf8(class_name)) = class_file.constants_pool.get((class.name_index - 1) as usize) {
             return class_name.data.as_str()
         }
    }

    ""
}

fn get_name_of_member(class_file: &ClassFile, name_and_type_index: u16) -> &str {
    if let Some(Constant::NameAndType(name_and_type)) = class_file.constants_pool.get((name_and_type_index - 1) as usize) {
        if let Some(Constant::Utf8(member_name)) = class_file.constants_pool.get((name_and_type.name_index - 1) as usize) {
            return member_name.data.as_str()
        }
    }

    ""
}

fn get_static(class_file: &ClassFile, cursor: &mut Cursor<Vec<u8>>, stack: &mut VecDeque<Instruction>) -> io::Result<()> {
    let index: u16 = cursor.read_u16()?;
    let field_ref: &ConstantFieldRef;
    match class_file.constants_pool.get((index - 1) as usize) {
        Some(Constant::FieldRef(fr)) => field_ref = fr,
        _ => panic!("GetStatic - Expected FieldRef!!")
    }

    let class_name = get_name_of_class(class_file, field_ref.class_index);
    let member_name = get_name_of_member(class_file, field_ref.name_and_type_index);

    if class_name.is_empty() || member_name.is_empty() {
        panic!("GetStatic - ClassName or MemberName not found!!")
    }

    // Only supports this type for now
    if class_name != "java/lang/System" && member_name != "out" {
        panic!("GetStatic - Unsupported class member {}.{}", class_name, member_name);
    }

    stack.push_back(Instruction::Type(String::from("FakePrintStream")));
    return Ok(())
}

fn ldc(class_file: &ClassFile, cursor: &mut Cursor<Vec<u8>>, stack: &mut VecDeque<Instruction>) -> io::Result<()> {
    let index: u8 = cursor.read_u8()?;
    let constant = class_file.constants_pool.get((index - 1) as usize);

    if let Some(Constant::String(string_constant)) = constant {
        if let Some(Constant::Utf8(string)) = class_file.constants_pool.get((string_constant.string_index - 1) as usize) {
            stack.push_back(Instruction::String(string.data.clone()));
            return Ok(())
        }
    }
    else if let Some(Constant::Integer(int_constant)) = constant {
        stack.push_back(Instruction::Integer(int_constant.value as i32));
    }
    else if let Some(Constant::Float(float_constant)) = constant {
        stack.push_back(Instruction::Float(float_constant.value));
    }

    panic!("LDC - Invalid constant type!!")
}

fn invoke_virtual(class_file: &ClassFile, cursor: &mut Cursor<Vec<u8>>, stack: &mut VecDeque<Instruction>) -> io::Result<()> {
    let index: u16 = cursor.read_u16()?;
    let method_ref: &ConstantMethodRef;
    match class_file.constants_pool.get((index - 1) as usize) {
        Some(Constant::MethodRef(mr)) => method_ref = mr,
        _ => panic!("InvokeVirtual - Expected MethodRef!!")
    }

    let class_name = get_name_of_class(class_file, method_ref.class_index);
    let member_name = get_name_of_member(class_file, method_ref.name_and_type_index);

    if class_name.is_empty() || member_name.is_empty() {
        panic!("InvokeVirtual - ClassName or MemberName not found!!")
    }

    if class_name == "java/io/PrintStream" && member_name == "println" {
        if stack.len() < 2 {
            panic!("InvokeVirtual - {}.{} expects two arguments", class_name, member_name);
        }

        let type_name: String;
        match stack.pop_front() {
            Some(Instruction::Type(tn)) => type_name = tn,
            _ => panic!("InvokeVirtual - Expected Instruction-Type!!")
        }

        if type_name != "FakePrintStream" {
            panic!("Unsupported stream type {}", type_name)
        }

        match stack.pop_front() {
            Some(Instruction::String(val)) => println!("{}", val),
            Some(Instruction::SByte(val)) => println!("{}", val),
            Some(Instruction::Short(val)) => println!("{}", val),
            Some(Instruction::Integer(val)) => println!("{}", val),
            Some(Instruction::Float(val)) => println!("{}", val),
            _ => panic!("InvokeVirtual - Expected Instruction-Constant!!")
        }

        return Ok(())
    }

    panic!("InvokeVirtual - Unsupported class method {}.{}", class_name, member_name);
}

pub fn execute_code(class_file: &ClassFile, code: Vec<u8>) -> io::Result<()> {
    let mut cursor = Cursor::new(code.clone());
    let mut stack: VecDeque<Instruction> = VecDeque::new();

    while cursor.position() < code.len() as u64 {
        let opcode = cursor.read_u8()?;
        match opcode {
            OP_CODE_GET_STATIC =>  {
                get_static(class_file, &mut cursor, &mut stack)?;
            }
            OP_CODE_LDC => {
                ldc(class_file, &mut cursor, &mut stack)?;
            },
            OP_CODE_INVOKE_VIRTUAL => {
                invoke_virtual(class_file, &mut cursor, &mut stack)?;
            },
            OP_CODE_BI_PUSH => {
                stack.push_back(Instruction::SByte(cursor.read_u8()? as i8));
            },
            OP_CODE_SI_PUSH => {
                stack.push_back(Instruction::Short(cursor.read_u16()? as i16));
            }
            OP_CODE_RETURN => {
                assert_eq!(stack.len(), 0, "Return - Stack was not empty!");
                return Ok(())
            }
            _ => {
                unimplemented!("OP CODE 0x{:x} NOT IMPLEMENTED", opcode);
            }
        }
    }

    unreachable!("execute_code should exit in OP_CODE_RETURN")
}
