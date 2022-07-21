use std::fmt::{Formatter, Debug, Display, Result as FormatResult};
use erased_serde::serialize_trait_object;
use std::any::Any;
use serde::Serialize;
use crate::model::cpp_object::CodeType::{CPP_CLASS, CPP_ENUM};
use crate::model::cpp_object::ObjectType::{Declaration, Definition};

#[derive(Debug, Serialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum CodeType {
    CPP_PACKAGE,
    CPP_CLASS,
    CPP_ENUM,
    CPP_CONSTRUCTOR,
    CPP_METHOD,
    Default
}

impl CodeType {
    pub fn type_codes() -> Vec<String>{
        let mut type_codes = Vec::new();
        type_codes.push(CodeType::CPP_PACKAGE.to_string());
        type_codes.push(CodeType::CPP_CLASS.to_string());
        type_codes.push(CodeType::CPP_ENUM.to_string());
        type_codes.push(CodeType::CPP_CONSTRUCTOR.to_string());
        type_codes.push(CodeType::CPP_METHOD.to_string());
        type_codes
    }
}
impl Display for CodeType {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        Debug::fmt(self, f)
    }
}

impl Default for CodeType{
    fn default() -> Self { CodeType::Default }
}

#[derive(Debug, Serialize)]
pub enum MethodReturnType {
    Value,
    Reference,
    Pointer,
    Default
}

impl Display for MethodReturnType {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        Debug::fmt(self, f)
    }
}

impl Default for MethodReturnType{
    fn default() -> Self { MethodReturnType::Default }
}

#[derive(Debug, Serialize)]
pub enum ObjectType {
    Declaration,
    Definition
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        Debug::fmt(self, f)
    }
}

impl Default for ObjectType{
    fn default() -> Self { Definition }
}


/* Object data (field inheritance) */
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ObjectData {
    name: String,
    line_number: usize,
    type_code: CodeType,
    children: Vec<Box<dyn CppObject>>,
    modifiers: Vec<String>
}

impl ObjectData {

    fn new(name: String, type_code: CodeType, line_number: usize) -> ObjectData {
        ObjectData {
            name,
            line_number,
            type_code,
            children: vec![],
            modifiers: vec![]
        }
    }


    fn mut_children(& mut self) -> & mut  Vec<Box<dyn CppObject>> {
        & mut self.children
    }

    fn add_child(& mut self, child: Box<dyn CppObject>) {
        self.children.push(child);
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn type_code(&self) -> &CodeType {
        &self.type_code
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn set_type_code(&mut self, type_code: CodeType) {
        self.type_code = type_code;
    }

    fn set_line_code(&mut self, line_code: usize) {
        self.line_number = line_code;
    }

    fn get_line_code(&self) -> usize {
        self.line_number
    }

    fn modifiers(&self) -> &Vec<String> {
        &self.modifiers
    }

    fn set_modifiers(&mut self, modifiers: Vec<String>) {
        self.modifiers = modifiers;
    }

    fn add_modifier(&mut self, modifier: String) {
        self.modifiers.push(modifier);
    }

    fn take(self) -> (String, usize, CodeType, Vec<Box<dyn CppObject>>, Vec<String>) {
        (self.name, self.line_number, self.type_code, self.children, self.modifiers)
    }

}

/* CppObject interface */
serialize_trait_object!(CppObject);
pub trait CppObject: erased_serde::Serialize {

    fn as_any(&self) -> &dyn Any;

    fn to_any(self: Box<Self>) -> Box<dyn Any>;

    fn mut_children(& mut self) -> & mut Vec<Box<dyn CppObject>>;

    fn add_child(& mut self, child: Box<dyn CppObject>);

    fn get_name(&self) -> String;

    fn set_name(&mut self, name: String);

    fn set_type_code(&mut self, type_code: CodeType);

    fn type_code(&self) -> &CodeType;

    fn name(&self) -> &String;

    fn set_line_code(&mut self, line_code: usize);

    fn get_line_code(&self) -> usize;

    fn modifiers(&self) -> &Vec<String>;

    fn set_modifiers(&mut self, modifiers: Vec<String>);

    fn add_modifier(& mut self, modifier: String);

    fn to_json(&self) -> String;

}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PackageObject {
    object_data: ObjectData,
}

impl CppObject for PackageObject {

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_any(self: Box<Self>) -> Box<dyn Any> {
        return self;
    }

    fn mut_children(& mut self) -> & mut Vec<Box<dyn CppObject>> {
        self.object_data.mut_children()
    }

    fn add_child(& mut self, child: Box<dyn CppObject>) {
        self.object_data.add_child(child);
    }

    fn get_name(&self) -> String {
        self.object_data.get_name()
    }

    fn set_name(&mut self, name: String) {
        self.object_data.set_name(name)
    }

    fn set_type_code(&mut self, type_code: CodeType) {
        self.object_data.set_type_code(type_code)
    }

    fn type_code(&self) -> &CodeType {
        self.object_data.type_code()
    }

    fn name(&self) -> &String {
        self.object_data.name()
    }

    fn set_line_code(&mut self, line_code: usize) {
        self.object_data.set_line_code(line_code)
    }

    fn get_line_code(&self) -> usize {
        self.object_data.get_line_code()
    }

    fn modifiers(&self) -> &Vec<String> {
        self.object_data.modifiers()
    }

    fn set_modifiers(&mut self, modifiers: Vec<String>) {
        self.object_data.set_modifiers(modifiers);
    }

    fn add_modifier(&mut self, modifier: String) {
        self.object_data.add_modifier(modifier);
    }

    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or("".to_string())
    }
}

impl PackageObject {

    pub fn new(package_name: String) -> PackageObject{
        PackageObject{
            object_data: ObjectData {
                name: package_name,
                line_number: 0,
                type_code: CodeType::CPP_PACKAGE,
                children: vec![],
                modifiers: vec![]
            }
        }
    }

    pub fn take(self) -> (String, usize, CodeType, Vec<Box<dyn CppObject>>, Vec<String>) {

        let (name,
            line_code,
            type_code,
            children,
            modifiers) = self.object_data.take();

        return (name,
                line_code,
                type_code,
                children,
                modifiers);
    }

}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClassObject {
    object_data: ObjectData,
    class_type: ObjectType,
}

impl CppObject for ClassObject {

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_any(self: Box<Self>) -> Box<dyn Any> {
        return self;
    }

    fn mut_children(& mut self) -> & mut Vec<Box<dyn CppObject>> {
        self.object_data.mut_children()
    }

    fn add_child(& mut self, child: Box<dyn CppObject>) {
        self.object_data.add_child(child);
    }

    fn get_name(&self) -> String {
        self.object_data.get_name()
    }

    fn set_name(&mut self, name: String) {
        self.object_data.set_name(name)
    }

    fn set_type_code(&mut self, type_code: CodeType) {
        self.object_data.set_type_code(type_code)
    }

    fn type_code(&self) -> &CodeType {
        self.object_data.type_code()
    }

    fn name(&self) -> &String {
        self.object_data.name()
    }

    fn set_line_code(&mut self, line_code: usize) {
        self.object_data.set_line_code(line_code)
    }

    fn get_line_code(&self) -> usize {
        self.object_data.get_line_code()
    }

    fn modifiers(&self) -> &Vec<String> {
        self.object_data.modifiers()
    }

    fn set_modifiers(&mut self, modifiers: Vec<String>) {
        self.object_data.set_modifiers(modifiers);
    }

    fn add_modifier(&mut self, modifier: String) {
        self.object_data.add_modifier(modifier);
    }

    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or(String::new())
    }
}

impl ClassObject {

    pub fn new_class(name: String, line_number: usize) -> Self {
        Self {
            object_data: ObjectData::new(name, CPP_CLASS, line_number),
            class_type: Definition
        }
    }

    pub fn new_enum(name: String, line_number: usize) -> Self {
        Self {
            object_data: ObjectData::new(name, CPP_ENUM, line_number),
            class_type: Declaration
        }
    }

    pub fn set_object_type(& mut self, object_type: ObjectType){
        self.class_type = object_type;
    }

    pub fn take(self) -> (String, usize, CodeType, Vec<Box<dyn CppObject>>, Vec<String>, ObjectType) {

        let (name,
            line_code,
            type_code,
            children,
            modifiers) = self.object_data.take();

        return (name,
                line_code,
                type_code,
                children,
                modifiers,
                self.class_type);
    }

}

#[derive(Serialize, Default)]
pub struct MethodObject {
    object_data: ObjectData,
    namespace: String,
    parameters: Vec<String>,
    output_parameter: String,
    method_type: ObjectType,
    return_type: MethodReturnType
}

impl CppObject for MethodObject {


    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_any(self: Box<Self>) -> Box<dyn Any> {
        return self;
    }

    fn mut_children(& mut self) -> & mut Vec<Box<dyn CppObject>> {
        self.object_data.mut_children()
    }

    fn add_child(& mut self, child: Box<dyn CppObject>) {
        self.object_data.add_child(child);
    }

    fn get_name(&self) -> String {
        self.object_data.get_name()
    }

    fn set_name(&mut self, name: String) {
        self.object_data.set_name(name)
    }

    fn set_type_code(&mut self, type_code: CodeType) {
        self.object_data.set_type_code(type_code)
    }

    fn type_code(&self) -> &CodeType {
        self.object_data.type_code()
    }

    fn name(&self) -> &String {
        self.object_data.name()
    }

    fn set_line_code(&mut self, line_code: usize) {
        self.object_data.set_line_code(line_code)
    }

    fn get_line_code(&self) -> usize {
        self.object_data.get_line_code()
    }

    fn modifiers(&self) -> &Vec<String> {
        self.object_data.modifiers()
    }

    fn set_modifiers(&mut self, modifiers: Vec<String>) {
        self.object_data.set_modifiers(modifiers);
    }

    fn add_modifier(&mut self, modifier: String) {
        self.object_data.add_modifier(modifier);
    }

    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or("None".parse().unwrap())
    }
}

impl MethodObject {

    pub fn new(name: String, type_code: CodeType, output_parameter: String,
               parameters: Vec<String>, namespace: String, return_type: MethodReturnType,
               line_code: usize, method_type: ObjectType) -> Self {

        Self {
            object_data: ObjectData { name, line_number: line_code, type_code, children: vec![], modifiers: vec![] },
            namespace,
            parameters,
            output_parameter,
            method_type,
            return_type
        }

    }

    pub fn take(self) -> (String, usize, CodeType, Vec<Box<dyn CppObject>>,
                          Vec<String>, Vec<String>, String, String, MethodReturnType, ObjectType) {

        let (name,
            line_code,
            type_code,
            children,
            modifiers) = self.object_data.take();

        return (name,
                line_code,
                type_code,
                children,
                modifiers,
                self.parameters,
                self.namespace,
                self.output_parameter,
                self.return_type,
                self.method_type);
    }

}




