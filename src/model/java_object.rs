use serde::{Serialize};
use erased_serde::serialize_trait_object;
use std::any::Any;
use std::fmt::{Formatter, Debug, Display, Result as FormatResult};

#[derive(Debug, Serialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum CodeType {
    JAVA_PACKAGE,
    JAVA_CLASS,
    JAVA_INTERFACE,
    JAVA_ENUM,
    JAVA_CONSTRUCTOR,
    JAVA_METHOD,
    Default
}

impl CodeType {
    pub fn type_codes() -> Vec<String>{
        let mut type_codes = Vec::new();
        type_codes.push(CodeType::JAVA_PACKAGE.to_string());
        type_codes.push(CodeType::JAVA_CLASS.to_string());
        type_codes.push(CodeType::JAVA_INTERFACE.to_string());
        type_codes.push(CodeType::JAVA_ENUM.to_string());
        type_codes.push(CodeType::JAVA_CONSTRUCTOR.to_string());
        type_codes.push(CodeType::JAVA_METHOD.to_string());
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


/* Object data (inheritance) */
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ObjectData {
    name: String,
    line_code: usize,
    type_code: CodeType,
    children: Vec<Box<dyn JavaObject>>,
    modifiers: Vec<String>
}

impl ObjectData {

    fn new(type_code: CodeType) -> ObjectData {
        ObjectData {
            line_code: 0,
            type_code,
            children: vec![],
            name: String::new(),
            modifiers: vec![]
        }
    }
    
    fn new_name(name: String, type_code: CodeType) -> ObjectData {
        ObjectData {
            line_code: 0,
            type_code,
            children: vec![],
            name,
            modifiers: vec![]
        }
    }


    fn add_child(& mut self, child: Box<dyn JavaObject>) {
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
        self.line_code = line_code;
    }

    fn get_line_code(&self) -> usize {
        self.line_code
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

    fn take(self) -> (String, usize, CodeType, Vec<Box<dyn JavaObject>>, Vec<String>) {
        (self.name, self.line_code, self.type_code, self.children, self.modifiers)
    }

}


/* JavaObject interface */
serialize_trait_object!(JavaObject);
pub trait JavaObject: erased_serde::Serialize {

    fn as_trait(&self) -> &dyn JavaObject;

    fn as_any(&self) -> &dyn Any;

    fn to_any(self: Box<Self>) -> Box<dyn Any>;

    fn add_child(&mut self, child: Box<dyn JavaObject>);

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


/* Package object */
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PackageObject {
    object_data: ObjectData,
}

impl JavaObject for PackageObject {

    fn as_trait(&self) -> &dyn JavaObject {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_any(self: Box<Self>) -> Box<dyn Any> {
        return self;
    }

    fn add_child(&mut self, child: Box<dyn JavaObject>) {
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

impl PackageObject {

    pub fn new() -> PackageObject{
        PackageObject{
            object_data: ObjectData::new(CodeType::JAVA_PACKAGE)
        }
    }

    pub fn take(self) -> (String, usize, CodeType, Vec<Box<dyn JavaObject>>, Vec<String>) {
        return self.object_data.take();
    }
}

/* Class object */
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClassObject {
    object_data: ObjectData
}

impl JavaObject for ClassObject {

    fn as_trait(&self) -> &dyn JavaObject {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_any(self: Box<Self>) -> Box<dyn Any> {
        return self;
    }

    fn add_child(&mut self, child: Box<dyn JavaObject>) {
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

    pub fn new(name: String) -> ClassObject {
        ClassObject{ object_data: ObjectData::new_name(name, CodeType::JAVA_CLASS) }
    }

    pub fn take(self) -> (String, usize, CodeType, Vec<Box<dyn JavaObject>>, Vec<String>) {
        return self.object_data.take();
    }

}

/* Method object */
#[derive(Serialize, Default)]
pub struct MethodObject {
    object_data: ObjectData,
    parameters: Vec<String>,
    output_parameter: String
}

impl JavaObject for MethodObject {
    fn as_trait(&self) -> &dyn JavaObject {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_any(self: Box<Self>) -> Box<dyn Any> {
        return self;
    }

    fn add_child(&mut self, child: Box<dyn JavaObject>) {
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
    
    pub fn new(name: String, type_code: CodeType) -> MethodObject {
        MethodObject {
            object_data: ObjectData::new_name(name, type_code),
            parameters: vec![],
            output_parameter: String::new(),
        }
    }
    
    pub fn set_parameters(&mut self, params: Vec<String>) {
        self.parameters = params
    }

    pub fn set_output_parameter(&mut self, output_parameters: String) { self.output_parameter = output_parameters }

    pub fn take(self) -> (String, usize, CodeType, Vec<Box<dyn JavaObject>>, Vec<String>, Vec<String>, String) {

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
                self.output_parameter);
    }

}
