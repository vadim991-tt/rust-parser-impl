use tree_sitter::Parser;
use crate::dto::invocation_structure::InvocationStructure;
use crate::dto::repository_method_dto::RepositoryMethodDto;
use crate::visitor::java_invocation_visitor::get_file_structure;
use crate::visitor::java_declaration_visitor::get_repository_method_dto;

pub fn get_invocation_structure(mut file_data: String, path: String) -> InvocationStructure {

    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_java::language())
        .expect("ERROR: Unable to load Java grammar");

    let tree = parser
        .parse(& mut file_data, None)
        .expect(format!("ERROR: Error occurred during parsing. Path_file {} ", &path).as_str());

    return get_file_structure(file_data, tree, path);
}

pub fn get_method_dto(mut file_data: String, rep_id: i32, path: String) -> Vec<RepositoryMethodDto>{

    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_java::language())
        .expect("ERROR: Unable to load Java grammar");

    let tree = parser
        .parse(& mut file_data, None)
        .expect(format!("ERROR: Error occurred during parsing. Path_file {} ", &path).as_str());

    return get_repository_method_dto(file_data, tree, path, rep_id);
}
