use std::fmt::{Formatter, Debug, Display, Result as FormatResult};
use erased_serde::serialize_trait_object;
use std::any::Any;
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum CodeType {
    PYTHON_PACKAGE,
    PYTHON_CLASS,
    PYTHON_ENUM,
    PYTHON_CONSTRUCTOR,
    PYTHON_METHOD,
    DEFAULT
}

impl CodeType {
    pub fn type_codes() -> Vec<String>{
        vec![
            CodeType::PYTHON_METHOD.to_string(),
            CodeType::PYTHON_PACKAGE.to_string(),
            CodeType::PYTHON_CLASS.to_string(),
            CodeType::PYTHON_ENUM.to_string(),
            CodeType::PYTHON_CONSTRUCTOR.to_string()
        ]
    }
}
impl Display for CodeType {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        Debug::fmt(self, f)
    }
}

impl Default for CodeType{
    fn default() -> Self { CodeType::DEFAULT }
}

/* Object data (field inheritance) */
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ObjectData {
    name: String,
    line_number: usize,
    type_code: CodeType,
    children: Vec<Box<dyn PythonObject>>,
}

impl ObjectData {

    fn new(name: String, type_code: CodeType) -> ObjectData {
        ObjectData {
            line_number: 0,
            type_code,
            children: vec![],
            name
        }
    }


    fn mut_children(& mut self) -> & mut  Vec<Box<dyn PythonObject>> {
        & mut self.children
    }

    fn add_child(& mut self, child: Box<dyn PythonObject>) {
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

    fn set_line_number(&mut self, line_number: usize) {
        self.line_number = line_number;
    }

    fn get_line_number(&self) -> usize {
        self.line_number
    }

    fn take(self) -> (String, usize, CodeType, Vec<Box<dyn PythonObject>>) {
        (self.name, self.line_number, self.type_code, self.children)
    }

}




/* PythonObject interface */
serialize_trait_object!(PythonObject);
pub trait PythonObject: erased_serde::Serialize {

    fn to_any(self: Box<Self>) -> Box<dyn Any>;

    fn mut_children(& mut self) -> & mut Vec<Box<dyn PythonObject>>;

    fn add_child(& mut self, child: Box<dyn PythonObject>);

    fn get_name(&self) -> String;

    fn set_name(&mut self, name: String);

    fn set_type_code(&mut self, type_code: CodeType);

    fn type_code(&self) -> &CodeType;

    fn name(&self) -> &String;

    fn set_line_number(&mut self, line_number: usize);

    fn get_line_number(&self) -> usize;

    fn to_json(&self) -> String;

}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PackageObject {
    object_data: ObjectData,
}


impl PythonObject for PackageObject {

    fn to_any(self: Box<Self>) -> Box<dyn Any> {
        return self;
    }

    fn mut_children(& mut self) -> & mut Vec<Box<dyn PythonObject>> {
        self.object_data.mut_children()
    }

    fn add_child(& mut self, child: Box<dyn PythonObject>) {
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

    fn set_line_number(&mut self, line_number: usize) {
        self.object_data.set_line_number(line_number)
    }

    fn get_line_number(&self) -> usize {
        self.object_data.get_line_number()
    }
    
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or("".to_string())
    }
}

impl PackageObject {

    pub fn new(package_name: String) -> PackageObject{
        PackageObject{ object_data: ObjectData::new(package_name, CodeType::PYTHON_PACKAGE) }
    }

    pub fn take(self) -> (String, usize, CodeType, Vec<Box<dyn PythonObject>>) {

        let (name,
            line_number,
            type_code,
            children) = self.object_data.take();

        return (name,
                line_number,
                type_code,
                children);
    }
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClassObject {
    object_data: ObjectData,
}

impl PythonObject for ClassObject {
    

    fn to_any(self: Box<Self>) -> Box<dyn Any> {
        return self;
    }

    fn mut_children(& mut self) -> & mut Vec<Box<dyn PythonObject>> {
        self.object_data.mut_children()
    }

    fn add_child(& mut self, child: Box<dyn PythonObject>) {
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

    fn set_line_number(&mut self, line_number: usize) {
        self.object_data.set_line_number(line_number)
    }

    fn get_line_number(&self) -> usize {
        self.object_data.get_line_number()
    }

    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or(String::new())
    }
}

impl ClassObject {

    pub fn take(self) -> (String, usize, CodeType, Vec<Box<dyn PythonObject>>) {

        let (name,
            line_number,
            type_code,
            children) = self.object_data.take();

        return (name,
                line_number,
                type_code,
                children);
    }

}

#[derive(Serialize, Default)]
pub struct MethodObject {
    object_data: ObjectData,
    parameters: Vec<String>,
    output_param: String
}

impl PythonObject for MethodObject {

    fn to_any(self: Box<Self>) -> Box<dyn Any> {
        return self;
    }

    fn mut_children(& mut self) -> & mut Vec<Box<dyn PythonObject>> {
        self.object_data.mut_children()
    }

    fn add_child(& mut self, child: Box<dyn PythonObject>) {
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

    fn set_line_number(&mut self, line_number: usize) {
        self.object_data.set_line_number(line_number)
    }

    fn get_line_number(&self) -> usize {
        self.object_data.get_line_number()
    }

    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or("None".parse().unwrap())
    }
}

impl MethodObject {

    pub fn new(name: String, type_code: CodeType, parameters: Vec<String>, line_number: usize, output_param: String) -> Self {

        Self {
            object_data: ObjectData {
                name,
                line_number,
                type_code,
                children: vec![]
            },
            parameters,
            output_param
        }

    }

    pub fn take(self) -> (String, usize, CodeType, Vec<Box<dyn PythonObject>>, Vec<String>, String) {

        let (name,
            line_number,
            type_code,
            children) = self.object_data.take();

        return (name,
                line_number,
                type_code,
                children,
                self.parameters,
                self.output_param);
    }

}









