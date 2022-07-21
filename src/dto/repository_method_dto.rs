use serde::Serialize;

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryMethodDto {
    repository_id: i32,
    path_file: String,
    package_name: String,
    line_code: usize,
    class_name: String,
    method_name: String,
    blob_data: String,
    modifiers: String,
    #[serde(rename = "type")]
    method_type: String,
    count_of_parameters: usize,
}

impl RepositoryMethodDto {

    pub fn new(repository_id: i32, path_file: String,
               package_name: String, line_code: usize,
               class_name: String, method_name: String,
               blob_data: String, modifiers: String,
               method_type: String, count_of_parameters: usize) -> Self {

        Self {
            repository_id,
            path_file,
            package_name,
            line_code,
            class_name,
            method_name,
            blob_data,
            modifiers,
            method_type,
            count_of_parameters
        }
    }

}

#[derive(Default)]
pub struct RepositoryMethodDtoBuilder {
    repository_id: i32,
    path_file: String,
    package_name: String,
    line_code: usize,
    class_name: String,
    method_name: String,
    blob_data: String,
    modifiers: String,
    method_type: String,
    count_of_parameters: usize,
}

impl RepositoryMethodDtoBuilder {

    pub fn repository_id(mut self, repository_id: i32) -> RepositoryMethodDtoBuilder {
        self.repository_id = repository_id;
        self
    }

    pub fn path_file(mut self, path_file: String) -> RepositoryMethodDtoBuilder {
        self.path_file = path_file;
        self
    }

    pub fn package_name(mut self, package_name: String) -> RepositoryMethodDtoBuilder {
        self.package_name = package_name;
        self
    }

    pub fn line_code(mut self, line_code: usize) -> RepositoryMethodDtoBuilder {
        self.line_code = line_code;
        self
    }

    pub fn class_name(mut self, class_name: String) -> RepositoryMethodDtoBuilder {
        self.class_name = class_name;
        self
    }

    pub fn method_name(mut self, method_name: String) -> RepositoryMethodDtoBuilder {
        self.method_name = method_name;
        self
    }

    pub fn modifiers(mut self, modifiers: String) -> RepositoryMethodDtoBuilder {
        self.modifiers = modifiers;
        self
    }

    pub fn method_type(mut self, method_type: String) -> RepositoryMethodDtoBuilder {
        self.method_type = method_type;
        self
    }

    pub fn count_of_parameters(mut self, count_of_parameters: usize) -> RepositoryMethodDtoBuilder {
        self.count_of_parameters = count_of_parameters;
        self
    }

    pub fn build(self) -> RepositoryMethodDto {

        RepositoryMethodDto {
            repository_id: self.repository_id,
            path_file: self.path_file,
            package_name: self.package_name,
            line_code: self.line_code,
            class_name: self.class_name,
            method_name: self.method_name,
            blob_data: self.blob_data,
            modifiers: self.modifiers,
            method_type: self.method_type,
            count_of_parameters: self.count_of_parameters
        }
    }
}