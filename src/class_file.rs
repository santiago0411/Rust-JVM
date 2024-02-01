use std::io;
use std::io::{Cursor};
use crate::utils::{read_attributes, ReadFromCursor};

pub const CONSTANT_UTF8: u8                 = 1;
pub const CONSTANT_INTEGER: u8              = 3;
pub const CONSTANT_FLOAT: u8                = 4;
pub const CONSTANT_LONG: u8                 = 5;
pub const CONSTANT_DOUBLE: u8               = 6;
pub const CONSTANT_CLASS: u8                = 7;
pub const CONSTANT_STRING: u8               = 8;
pub const CONSTANT_FIELD_REF: u8            = 9;
pub const CONSTANT_METHOD_REF: u8           = 10;
pub const CONSTANT_INTERFACE_METHOD_REF: u8 = 11;
pub const CONSTANT_NAME_AND_TYPE: u8        = 12;
pub const CONSTANT_METHOD_HANDLE: u8        = 15;
pub const CONSTANT_METHOD_TYPE: u8          = 16;
pub const CONSTANT_INVOKE_DYNAMIC: u8       = 18;

pub enum Constant {
    Utf8(ConstantUft8),
    Integer(ConstantInteger),
    Float(ConstantFloat),
    Class(ConstantClass),
    String(ConstantString),
    FieldRef(ConstantFieldRef),
    MethodRef(ConstantMethodRef),
    InterfaceMethodRef(ConstantInterfaceMethodRef),
    NameAndType(ConstantNameAndType)
}

pub struct ConstantUft8 {
    pub tag: String,
    pub data: String
}

pub struct ConstantInteger {
    pub tag: String,
    pub value: u32
}

pub struct ConstantFloat {
    pub tag: String,
    pub value: f32
}

pub struct ConstantClass {
    pub tag: String,
    pub name_index: u16
}

pub struct ConstantString {
    pub tag: String,
    pub string_index: u16
}

pub struct ConstantFieldRef {
    pub tag: String,
    pub class_index: u16,
    pub name_and_type_index: u16
}

pub struct ConstantMethodRef {
    pub tag: String,
    pub class_index: u16,
    pub name_and_type_index: u16
}

pub struct ConstantInterfaceMethodRef {
    pub tag: String,
    pub class_index: u16,
    pub name_and_type_index: u16
}

pub struct ConstantNameAndType {
    pub tag: String,
    pub name_index: u16,
    pub descriptor_index: u16
}

pub enum ClassAccessFlags {
    PUBLIC =	    0x0001,
    FINAL =	        0x0010,
    SUPER =	        0x0020,
    INTERFACE =	    0x0200,
    ABSTRACT =	    0x0400,
    SYNTHETIC =	    0x1000,
    ANNOTATION =    0x2000,
    ENUM =	        0x4000,
}

pub enum FieldsAccessFlags {
    PUBLIC =	0x0001,
    PRIVATE =	0x0002,
    PROTECTED =	0x0004,
    STATIC =	0x0008,
    FINAL =	    0x0010,
    VOLATILE =	0x0040,
    TRANSIENT =	0x0080,
    SYNTHETIC =	0x1000,
    ENUM =	    0x4000,
}

pub enum MethodsAccessFlags {
    PUBLIC =	    0x0001,
    PRIVATE =	    0x0002,
    PROTECTED =	    0x0004,
    STATIC =	    0x0008,
    FINAL =	        0x0010,
    SYNCHRONIZED =	0x0020,
    BRIDGE =	    0x0040,
    VARARGS =	    0x0080,
    NATIVE =	    0x0100,
    ABSTRACT =	    0x0400,
    STRICT =	    0x0800,
    SYNTHETIC =	    0x1000,
}

pub struct MethodInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>
}

pub struct AttributeInfo {
    pub attribute_name_index: u16,
    pub info: Vec<u8>
}

pub struct ClassFile {
    pub magic: u32,
    pub minor: u16,
    pub major: u16,
    pub constants_pool: Vec<Constant>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    // interfaces
    // fields
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<AttributeInfo>
}

impl ClassFile {
    pub fn new(cursor: &mut Cursor<Vec<u8>>) -> io::Result<Box<ClassFile>> {
        let magic = cursor.read_u32()?;
        let minor = cursor.read_u16()?;
        let major = cursor.read_u16()?;

        let constants_pool: Vec<Constant>;
        match ClassFile::read_constants_pool(cursor) {
            Err(err) => {
                eprintln!("Error reading constants pool: {}", err);
                return Err(err)
            }
            Ok(pool) => constants_pool = pool
        }

        let access_flags = cursor.read_u16()?;
        let this_class = cursor.read_u16()?;
        let super_class = cursor.read_u16()?;

        let interfaces_count = cursor.read_u16()?;
        assert_eq!(interfaces_count, 0, "Interfaces are not supported");

        let fields_count = cursor.read_u16()?;
        assert_eq!(fields_count, 0, "Fields are not supported");

        let methods: Vec<MethodInfo>;
        match ClassFile::read_methods(cursor) {
            Err(err) => {
                eprintln!("Error reading methods: {}", err);
                return Err(err)
            }
            Ok(mds) => methods = mds
        }

        let attributes_count = cursor.read_u16()?;
        let attributes: Vec<AttributeInfo>;
        match read_attributes(cursor, attributes_count as usize) {
            Err(err) => {
                eprintln!("Error reading class attributes: {}", err);
                return Err(err)
            }
            Ok(ats) => attributes = ats
        }

        return Ok(Box::new(ClassFile {
            magic,
            minor,
            major,
            constants_pool,
            access_flags,
            this_class,
            super_class,
            methods,
            attributes
        }));
    }

    pub fn find_method_by_name(&self, name: &str) -> Option<&MethodInfo> {
        self.methods.iter().find(|method| {
            if let Some(Constant::Utf8(method_name)) = self.constants_pool.get((method.name_index - 1) as usize) {
                return method_name.data == name
            }
            return false
        })
    }

    pub fn find_attribute_by_name<'a>(&'a self, attributes: &'a Vec<AttributeInfo>, name: &str) -> Option<&'a AttributeInfo> {
        attributes.iter().find(|&att| {
            if let Some(Constant::Utf8(att_name)) = self.constants_pool.get((att.attribute_name_index - 1) as usize) {
                return att_name.data == name;
            }
            false
        })
    }

    fn read_constants_pool(cursor: &mut Cursor<Vec<u8>>) -> io::Result<Vec<Constant>> {
        let pool_count = cursor.read_u16()?;
        let mut pool: Vec<Constant> = Vec::with_capacity(pool_count as usize);

        for _ in 0..pool_count - 1 {
            let tag: u8  = cursor.read_u8()?;
            let constant: Constant = match tag {
                CONSTANT_UTF8 => {
                    let length: u16 = cursor.read_u16()?;
                    let data: String = cursor.read_string(length as usize)?;
                    Constant::Utf8(ConstantUft8 {
                        tag: String::from("CONSTANT_UTF8"),
                        data
                    })
                },
                CONSTANT_INTEGER => Constant::Integer(ConstantInteger {
                    tag: String::from("CONSTANT_INTEGER"),
                    value: cursor.read_u32()?
                }),
                CONSTANT_FLOAT => Constant::Float(ConstantFloat {
                    tag: String::from("CONSTANT_FLOAT"),
                    value: cursor.read_f32()?
                }),
                CONSTANT_STRING => Constant::String(ConstantString {
                    tag: String::from("CONSTANT_STRING"),
                    string_index: cursor.read_u16()?
                }),
                CONSTANT_CLASS => Constant::Class(ConstantClass {
                    tag: String::from("CONSTANT_CLASS"),
                    name_index: cursor.read_u16()?
                }),
                CONSTANT_FIELD_REF => Constant::FieldRef(ConstantFieldRef {
                    tag: String::from("CONSTANT_FIELD_REF"),
                    class_index: cursor.read_u16()?,
                    name_and_type_index: cursor.read_u16()?
                }),
                CONSTANT_METHOD_REF => Constant::MethodRef(ConstantMethodRef {
                    tag: String::from("CONSTANT_METHOD_REF"),
                    class_index: cursor.read_u16()?,
                    name_and_type_index: cursor.read_u16()?
                }),
                CONSTANT_INTERFACE_METHOD_REF => Constant::InterfaceMethodRef(ConstantInterfaceMethodRef {
                    tag: String::from("CONSTANT_INTERFACE_METHOD_REF"),
                    class_index: cursor.read_u16()?,
                    name_and_type_index: cursor.read_u16()?
                }),
                CONSTANT_NAME_AND_TYPE => Constant::NameAndType(ConstantNameAndType {
                    tag: String::from("CONSTANT_NAME_AND_TYPE"),
                    name_index: cursor.read_u16()?,
                    descriptor_index: cursor.read_u16()?
                }),
                _ => {
                    unimplemented!("Unsupported constant pool tag: {}", tag);
                }
            };

            pool.push(constant);
        }

        return Ok(pool)
    }

    fn read_methods(cursor: &mut Cursor<Vec<u8>>) -> io::Result<Vec<MethodInfo>> {
        let methods_count = cursor.read_u16()?;
        let mut methods: Vec<MethodInfo> = Vec::with_capacity(methods_count as usize);

        for _ in 0..methods_count {
            let access_flags: u16 = cursor.read_u16()?;
            let name_index: u16 = cursor.read_u16()?;
            let descriptor_index: u16 = cursor.read_u16()?;
            let attributes_count: u16 = cursor.read_u16()?;
            let attributes: Vec<AttributeInfo> = read_attributes(cursor, attributes_count as usize)?;
            methods.push(MethodInfo {
                access_flags,
                name_index,
                descriptor_index,
                attributes
            });
        }

        return Ok(methods)
    }
}