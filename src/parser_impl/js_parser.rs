use std::path::Path;
use stringreader::StringReader;
use std::io::{BufReader, BufRead};
use tree_sitter::Parser;
use crate::dto::invocation_structure::InvocationStructure;
use crate::dto::repository_method_dto::RepositoryMethodDto;
use crate::visitor::js_invocation_visitor::get_file_structure;
use crate::visitor::js_declaration_visitor::get_repository_method_dto;

pub fn get_invocation_structure(mut file_data: String, path: String) -> InvocationStructure {

    if !is_source_code_valid(&file_data, &path) {
        return InvocationStructure::default();
    }

    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_javascript::language())
        .expect("ERROR: Unable to load JavaScript grammar");

    let tree = parser
        .parse(& mut file_data, None)
        .expect(format!("ERROR: Error occurred during parsing. Path_file {} ", &path).as_str());

    return get_file_structure(file_data, tree, path);
}

pub fn get_method_dto(mut file_data: String, rep_id: i32, path: String) -> Vec<RepositoryMethodDto>{

    if !is_source_code_valid(&file_data, &path) {
        return vec![];
    }

    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_javascript::language())
        .expect("ERROR: Unable to load JavaScript grammar");

    let tree = parser
        .parse(& mut file_data, None)
        .expect(format!("ERROR: Error occurred during parsing. Path_file {} ", &path).as_str());

    return get_repository_method_dto(file_data, tree, path, rep_id);
}

fn is_source_code_valid(source: &str, path_file: &str) -> bool {

    /* File name validation */
    let path = Path::new(path_file);
    let file_stem = path.file_stem();
    let file_stem_os_str = match file_stem {
        Some(file_stem) => file_stem.to_str(),
        None => return false
    };

    let name = match file_stem_os_str {
        Some(string) => string,
        None => return false
    };

    if name.contains(".min") || name.contains(".dev") {
        return false;
    }

    /* Line number validation */
    let line_number = source.matches('\n').count();
    if line_number == 0 {
        return false;
    }

    /* File content validation */
    let string_reader = StringReader::new(source);
    let buf_reader = BufReader::new(string_reader);
    let longest_line_length = match buf_reader
        .lines()
        .filter_map(Result::ok)
        .max_by(|x, y|{x.len().cmp(&y.len())}) {
            Some(line) => line.len(),
            None => return false,
    };

    const MAX_LINE_LENGTH: usize = 500;
    const MAX_LINE_LENGTH_WITH_SMALL_LINE_NUMBER:usize = 200;
    const REQUIRED_LINE_NUMBER:usize = 10;

    if longest_line_length > MAX_LINE_LENGTH {
        return false;
    } else if line_number < REQUIRED_LINE_NUMBER  && longest_line_length > MAX_LINE_LENGTH_WITH_SMALL_LINE_NUMBER {
        return false;
    } else {
        return true;
    }
}

#[cfg(test)]
mod js_parser_tests {
    use super::*;

    #[test]
    pub fn test_js_validate_name(){

        let min_path_file = String::from("./some_dir.min/some.dir_again/some_file.min.js");
        assert!(!is_source_code_valid("", &min_path_file));

        let dev_path_file = String::from("./some_dir.dev/some.dir_again/some_file.dev.js");
        assert!(!is_source_code_valid("", &dev_path_file));

        let zero_lines_file = String::from("./normal_dir/file.js");
        assert!(!is_source_code_valid("", &zero_lines_file));

        let normal_file = String::from("/normal_dir/normal_file.js");
        let normal_content = format!("{}\n", r#"
            const greet = function greet(name){
                console.log("Hello - ", name);
            }
        "#);
        assert!(is_source_code_valid(&normal_content, &normal_file));
    }
}
