use serde::Serialize;
use crate::dto::object_description::MethodDescription;

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryImportDeclaration {
    package_name: String,
    class_import_list: Vec<String>,
}

impl RepositoryImportDeclaration {

    pub fn new(package_name: String) -> Self {
        Self { package_name, class_import_list: vec![] }
    }

    pub fn add_class(&mut self, class_name: String) {
        if !self.class_import_list.contains(&class_name) {
            self.class_import_list.push(class_name)
        }
    }
}

impl PartialEq<Self> for RepositoryImportDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.package_name == other.package_name
    }
}

impl Eq for RepositoryImportDeclaration {}



#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvocationStructure {
    repository_import_declarations: Vec<RepositoryImportDeclaration>,
    method_descriptions: Vec<MethodDescription>,
    type_codes: Vec<String>,
}

impl InvocationStructure {

    pub fn new(repository_import_declarations: Vec<RepositoryImportDeclaration>,
               method_descriptions: Vec<MethodDescription>, type_codes: Vec<String>) -> Self {
        Self {
            repository_import_declarations,
            method_descriptions,
            type_codes,
        }
    }

}