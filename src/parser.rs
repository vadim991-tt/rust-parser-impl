use crate::dto::repository_method_dto::RepositoryMethodDto;
use crate::dto::invocation_structure::InvocationStructure;
use crate::parser_impl::{java_parser, python_parser, ts_parser};
use crate::parser_impl::js_parser;
use crate::parser_impl::cpp_parser;

struct SupportedLanguages;
impl SupportedLanguages {
    pub const JS: &'static str= "JS";
    pub const JAVA: &'static str = "JAVA";
    pub const CPP: &'static str = "CPP";
    pub const PYTHON:&'static str = "PYTHON";
    pub const TS: &'static str = "TS";
}

pub fn parse_file_get_dto(file_data: String, rep_id: i32,
                          path: String, language: String) -> Vec<RepositoryMethodDto> {

    match language.as_str() {
        SupportedLanguages::JS => js_parser::get_method_dto(file_data, rep_id, path),
        SupportedLanguages::JAVA => java_parser::get_method_dto(file_data, rep_id, path),
        SupportedLanguages::CPP => cpp_parser::get_method_dto(file_data, rep_id, path),
        SupportedLanguages::PYTHON => python_parser::get_method_dto(file_data, rep_id, path),
        SupportedLanguages::TS => ts_parser::get_method_dto(file_data, rep_id, path),
        _ => vec![]
    }
}

pub fn parse_file_get_invocation_structure(file_data: String, path: String, language: String) -> InvocationStructure {
    match language.as_str() {
        SupportedLanguages::JS => js_parser::get_invocation_structure(file_data, path),
        SupportedLanguages::JAVA => java_parser::get_invocation_structure(file_data, path),
        SupportedLanguages::CPP => cpp_parser::get_invocation_structure(file_data, path),
        SupportedLanguages::PYTHON => python_parser::get_invocation_structure(file_data, path),
        SupportedLanguages::TS => ts_parser::get_invocation_structure(file_data, path),
        _ => InvocationStructure::default()
    }
}

