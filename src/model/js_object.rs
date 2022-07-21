use serde::{Serialize};
use erased_serde::serialize_trait_object;
use std::any::Any;
use std::fmt::{Display, Formatter, Debug, Result as FormatResult};

#[derive(Debug, Serialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum CodeType {
    JS_PACKAGE,
    JS_CLASS,
    JS_METHOD,
    JS_CONSTRUCTOR,
    DEFAULT,
}

impl CodeType {
    pub fn type_codes() -> Vec<String>{
        let mut type_codes = Vec::new();
        type_codes.push(CodeType::JS_PACKAGE.to_string());
        type_codes.push(CodeType::JS_CLASS.to_string());
        type_codes.push(CodeType::JS_METHOD.to_string());
        type_codes.push(CodeType::JS_CONSTRUCTOR.to_string());
        type_codes
    }
}

impl Display for CodeType {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        Debug::fmt(self, f)
    }
}

impl Default for CodeType {
    fn default() -> Self { CodeType::DEFAULT }
}


/* Object data (inheritance) */
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ObjectData {
    name: String,
    line_code: usize,
    type_code: CodeType,
    children: Vec<Box<dyn JsObject>>
}

impl ObjectData {

    fn new(name: String, type_code: CodeType) -> Self {
        Self {
            line_code: 0,
            type_code,
            children: vec![],
            name,
        }
    }

    fn children(&self) -> &Vec<Box<dyn JsObject>> {
        &self.children
    }

    fn add_child(&mut self, child: Box<dyn JsObject>) {
        self.children.push(child);
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn type_code(&self) -> &CodeType {
        &self.type_code
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

    fn take(self) -> (String, usize, CodeType, Vec<Box<dyn JsObject>>) {
        (self.name, self.line_code, self.type_code, self.children)
    }
}


/* JsObject interface */
serialize_trait_object!(JsObject);
pub trait JsObject: erased_serde::Serialize {
    fn as_trait(&self) -> &dyn JsObject;

    fn as_any(&self) -> &dyn Any;

    fn children(& mut self) -> &Vec<Box<dyn JsObject>>;

    fn add_child(& mut self, child: Box<dyn JsObject>);

    fn get_name(&self) -> String;

    fn set_name(&mut self, name: String);

    fn set_type_code(&mut self, type_code: CodeType);

    fn type_code(&self) -> &CodeType;

    fn name(&self) -> &String;

    fn set_line_code(&mut self, line_code: usize);

    fn get_line_code(&self) -> usize;

    fn to_json(&self) -> String;

    fn to_any(self: Box<Self>) -> Box<dyn Any>;

}


/* Package object */
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PackageObject {
    object_data: ObjectData,
}

impl JsObject for PackageObject {

    fn as_trait(&self) -> &dyn JsObject {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn children(&mut self) -> &Vec<Box<dyn JsObject>> {
        self.object_data.children()
    }

    fn add_child(&mut self, child: Box<dyn JsObject>) {
        self.object_data.add_child(child)
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

    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or("None".parse().unwrap())
    }

    fn to_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl PackageObject {

    pub fn new_name (name: String) -> Self {
        Self { object_data: ObjectData::new(name, CodeType::JS_PACKAGE) }
    }

    pub fn take(self) -> (String, usize, CodeType, Vec<Box<dyn JsObject>>) {

        let (name,
            line_code,
            type_code,
            children) = self.object_data.take();

        return (name,
                line_code,
                type_code,
                children);
    }

}

/* Class object */
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClassObject {
    object_data: ObjectData,
}

impl JsObject for ClassObject {
    fn as_trait(&self) -> &dyn JsObject {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn children(&mut self) -> &Vec<Box<dyn JsObject>> {
        self.object_data.children()
    }

    fn add_child(&mut self, child: Box<dyn JsObject>) {
        self.object_data.add_child(child)
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

    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or("None".parse().unwrap())
    }

    fn to_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }


}

impl ClassObject {

    pub fn new_name(name: String) -> Self{
        Self { object_data: ObjectData::new(name, CodeType::JS_CLASS) }
    }

    pub fn take(self) -> (String, usize, CodeType, Vec<Box<dyn JsObject>>) {

        let (name,
            line_code,
            type_code,
            children) = self.object_data.take();

        return (name,
                line_code,
                type_code,
                children);
    }

}

/* Method object */
#[derive(Serialize, Default)]
pub struct MethodObject {
    object_data: ObjectData,
    parameters: Vec<String>,
}

impl JsObject for MethodObject {

    fn as_trait(&self) -> &dyn JsObject {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn children(&mut self) -> &Vec<Box<dyn JsObject>> {
        self.object_data.children()
    }

    fn add_child(&mut self, child: Box<dyn JsObject>) {
        self.object_data.add_child(child)
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

    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or("None".parse().unwrap())
    }

    fn to_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

}

impl MethodObject {

    pub fn new(name: String) -> Self {
        Self { object_data: ObjectData::new(name, CodeType::JS_METHOD), parameters: vec![] }
    }

    pub fn new_code(name: String, code: CodeType) -> Self {
        Self { object_data: ObjectData::new(name, code), parameters: vec![] }
    }

    pub fn set_parameters(&mut self, params: Vec<String>) {
        self.parameters = params
    }

    pub fn take(self) -> (String, usize, CodeType, Vec<Box<dyn JsObject>>, Vec<String>) {

        let (name,
            line_code,
            type_code,
            children) = self.object_data.take();

        return (name,
                line_code,
                type_code,
                children,
                self.parameters);
    }
}
