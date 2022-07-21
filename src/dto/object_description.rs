use serde::Serialize;

pub trait Description {

    fn get_package_name(&self) -> String;

    fn package_name(&self) -> &String;

    fn set_package_name(&mut self, package_name: String);

    fn class_name(&self) -> &String;

    fn get_class_name(&self) -> String;

    fn set_class_name(&mut self, class_name: String);

    fn line(&self) -> usize;

    fn set_line(&mut self, line: usize);

    fn position(&self) -> usize;

    fn set_position(&mut self, position: usize);
}

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptionData {
    package_name: String,
    class_name: String,
    line: usize,
    position: usize,
}


impl DescriptionData {

    pub fn new(package_name: String, class_name: String, line: usize, position: usize) -> Self {
        DescriptionData { package_name, class_name, line, position }
    }

    pub fn package_name(&self) -> &String {
        &self.package_name
    }

    pub fn get_package_name(&self) -> String {
        self.package_name.clone()
    }

    pub fn class_name(&self) -> &String {
        &self.class_name
    }

    pub fn get_class_name(&self) -> String {
        self.class_name.clone()
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn set_package_name(&mut self, package_name: String) {
        self.package_name = package_name;
    }

    pub fn set_class_name(&mut self, class_name: String) {
        self.class_name = class_name;
    }

    pub fn set_line(&mut self, line: usize) {
        self.line = line;
    }

    pub fn set_position(&mut self, position: usize) {
        self.position = position;
    }

}

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MethodDescription {
    description_data: DescriptionData,
    var_name: String,
    method_name: String,
    count_param_input: usize,
}


impl MethodDescription {

    pub fn new (package_name: String, class_name: String, line: usize, position: usize,
                var_name: String, method_name: String, count_param_input: usize) -> Self{
        Self {
            description_data: DescriptionData::new(package_name, class_name, line, position),
            var_name,
            method_name,
            count_param_input
        }
    }

    pub fn set_var_name(&mut self, var_name: String) {
        self.var_name = var_name;
    }

    pub fn set_method_name(&mut self, method_name: String) {
        self.method_name = method_name;
    }

    pub fn set_count_param_input(&mut self, count_param_input: usize) {
        self.count_param_input = count_param_input;
    }

}

impl Description for MethodDescription {

    fn get_package_name(&self) -> String {
        self.description_data.get_package_name()
    }

    fn set_package_name(&mut self, package_name: String) {
        self.description_data.set_package_name(package_name);
    }

    fn package_name(&self) -> &String {
        self.description_data.package_name()
    }

    fn class_name(&self) -> &String {
        self.description_data.class_name()
    }

    fn get_class_name(&self) -> String {
        self.description_data.get_class_name()
    }

    fn set_class_name(&mut self, class_name: String) {
        self.description_data.set_class_name(class_name);
    }

    fn line(&self) -> usize {
        self.description_data.line()
    }

    fn set_line(&mut self, line: usize) {
        self.description_data.set_line(line)
    }

    fn position(&self) -> usize {
        self.description_data.position()
    }

    fn set_position(&mut self, position: usize) {
        self.description_data.set_position(position)
    }
}

#[derive(Default)]
pub struct PackageDescription {
    description_data: DescriptionData,
    parents: Vec<String>,
}

impl PackageDescription {

    pub fn new(package_name: String, class_name: String, line: usize, position: usize, parents: Vec<String>) -> Self{
        Self {
            description_data: DescriptionData::new(package_name, class_name, line, position),
            parents
        }
    }

    pub fn parents(&self) -> &Vec<String> {
        &self.parents
    }

    pub fn mut_parents(& mut self) -> & mut Vec<String> {
        & mut self.parents
    }

    pub fn add_parent(& mut self, parent: String) {
        self.parents.push(parent);
    }

}

impl Description for PackageDescription {

    fn get_package_name(&self) -> String {
        self.description_data.get_package_name()
    }

    fn package_name(&self) -> &String {
        self.description_data.package_name()
    }

    fn set_package_name(&mut self, package_name: String) {
        self.description_data.set_package_name(package_name);
    }

    fn class_name(&self) -> &String {
        self.description_data.class_name()
    }

    fn get_class_name(&self) -> String {
        self.description_data.get_class_name()
    }

    fn set_class_name(&mut self, class_name: String) {
        self.description_data.set_class_name(class_name);
    }

    fn line(&self) -> usize {
        self.description_data.line()
    }

    fn set_line(&mut self, line: usize) {
        self.description_data.set_line(line)
    }

    fn position(&self) -> usize {
        self.description_data.position()
    }

    fn set_position(&mut self, position: usize) {
        self.description_data.set_position(position)
    }
}

#[derive(Debug, Default, Serialize)]
pub struct VarDescription {
    description_data: DescriptionData,
    var_name: String,
}

impl VarDescription {

    pub fn new(package_name: String, class_name: String, line: usize, position: usize, var_name: String) -> Self {
        VarDescription {
            description_data: DescriptionData::new(package_name, class_name, line, position),
            var_name
        }
    }

    pub fn var_name(&self) -> &str {
        &self.var_name
    }

    pub fn get_var_name(&self) -> String {
        self.var_name.clone()
    }

    pub fn set_var_name(&mut self, var_name: String) {
        self.var_name = var_name;
    }
}

impl Description for VarDescription {

    fn get_package_name(&self) -> String {
        self.description_data.get_package_name()
    }

    fn package_name(&self) -> &String {
        self.description_data.package_name()
    }

    fn set_package_name(&mut self, package_name: String) {
        self.description_data.set_package_name(package_name);
    }

    fn class_name(&self) -> &String {
        self.description_data.class_name()
    }

    fn get_class_name(&self) -> String {
        self.description_data.get_class_name()
    }

    fn set_class_name(&mut self, class_name: String) {
        self.description_data.set_class_name(class_name);
    }

    fn line(&self) -> usize {
        self.description_data.line()
    }

    fn set_line(&mut self, line: usize) {
        self.description_data.set_line(line)
    }

    fn position(&self) -> usize {
        self.description_data.position()
    }

    fn set_position(&mut self, position: usize) {
        self.description_data.set_position(position)
    }
}


