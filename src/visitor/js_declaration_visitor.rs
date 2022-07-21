use crate::model::js_object::{JsObject, MethodObject, ClassObject, PackageObject};
use crate::dto::repository_method_dto::RepositoryMethodDto;
use tree_sitter::{Node, Tree, TreeCursor};
use crate::unwrap_or_return;
use std::io::Read;
use crate::unwrap_or_empty_string;
use crate::model::js_object::CodeType::{JS_METHOD, JS_CONSTRUCTOR, JS_CLASS, JS_PACKAGE};

const MAX_TOKEN_LENGTH: usize = 250;

struct NodeKinds;

impl NodeKinds {
    const FUNCTION_DECLARATION: &'static str = "function_declaration";
    const VARIABLE_DECLARATOR: &'static str = "variable_declarator";
    const ASSIGNMENT_PATTERN: &'static str = "assignment_pattern";
    const CLASS_DECLARATION: &'static str = "class_declaration";
    const METHOD_DEFINITION: &'static str = "method_definition";
    const ARROW_FUNCTION: &'static str = "arrow_function";
    const IDENTIFIER: &'static str = "identifier";
    const FUNCTION: &'static str = "function";
}

struct NodeNames;

impl NodeNames {
    const PARAMETERS: &'static str = "parameters";
    const PARAMETER: &'static str = "parameter";
    const VALUE: &'static str = "value";
    const LEFT: &'static str = "left";
    const NAME: &'static str = "name";
    const BODY: &'static str = "body";
}

struct KeyWords;

impl KeyWords {
    const CONSTRUCTOR_IDENTIFIER: &'static str = "constructor";
    const EMPTY_STRING: &'static str = "";
}

struct DeclarationData {
    source_code: String,
    path_file: String,
}

impl DeclarationData {
    pub fn new(source_code: String, path_file: String) -> DeclarationData {
        DeclarationData {
            source_code,
            path_file,
        }
    }

    fn source_code(&self) -> &String {
        &self.source_code
    }


    fn path_file(&self) -> &String {
        &self.path_file
    }

}


/* Main function */
pub fn get_repository_method_dto(source: String, tree: Tree, path: String, rep_id: i32) -> Vec<RepositoryMethodDto> {

    let mut parse_data = DeclarationData::new(source, path.clone());
    let package_object = PackageObject::new_name(path.clone());
    let mut parent: Box<dyn JsObject> = Box::new(package_object);

    traverse_tree(&mut parent, tree.walk(), &mut parse_data);

    let current_package_name = KeyWords::EMPTY_STRING.to_string();
    let current_class_name = KeyWords::EMPTY_STRING.to_string();
    let mut method_dto_vec = vec![];

    prepare_output_data(
        &mut method_dto_vec,
        parent,
        rep_id,
        &path,
        &current_package_name,
        &current_class_name,
    );

    return method_dto_vec;
}


fn traverse_tree(parent: &mut Box<dyn JsObject>, mut tree_cursor: TreeCursor, parse_data: &mut DeclarationData) {

    let mut reached_root = false;

    while reached_root == false {

        walk_into(parent, tree_cursor.node(), parse_data);

        if tree_cursor.goto_first_child() { continue; }
        if tree_cursor.goto_next_sibling() { continue; }

        let mut retracing = true;
        while retracing {
            if !tree_cursor.goto_parent() {
                retracing = false;
                reached_root = true;
            }
            if tree_cursor.goto_next_sibling() {
                retracing = false;
            }
        }
    }
}


fn prepare_output_data(dto_vec: &mut Vec<RepositoryMethodDto>, js_object: Box<dyn JsObject>,
                       rep_id: i32, path: &String, package_name: &String, class_name: &String) {

    match js_object.type_code() {
        JS_PACKAGE => prepare_package_object(dto_vec, js_object, rep_id, path, class_name),
        JS_CLASS => prepare_class_object(dto_vec, js_object, rep_id, path, package_name),
        JS_METHOD | JS_CONSTRUCTOR => prepare_method_object(dto_vec, js_object, rep_id, path, class_name, package_name),
        &_ => {}
    }
}

fn prepare_package_object(dto_vec: &mut Vec<RepositoryMethodDto>, js_object: Box<dyn JsObject>,
                          rep_id: i32, path: &String, class_name: &String) {

    let boxed_any = js_object.to_any();
    let boxed_package = boxed_any.downcast::<PackageObject>().unwrap();
    let package = *boxed_package;

    let (name,
        line_code,
        type_code,
        children) = package.take();


    let repository_method_dto = RepositoryMethodDto::new(
        rep_id,
        path.clone(),
        name.clone(),
        line_code,
        KeyWords::EMPTY_STRING.to_string(),
        KeyWords::EMPTY_STRING.to_string(),
        KeyWords::EMPTY_STRING.to_string(),
        serde_json::to_string(&Vec::<String>::new()).unwrap(),
        type_code.to_string(),
        0,
    );

    dto_vec.push(repository_method_dto);
    for child in children {
        prepare_output_data(dto_vec, child, rep_id, path, &name, &class_name);
    }
}

fn prepare_class_object(dto_vec: &mut Vec<RepositoryMethodDto>, js_object: Box<dyn JsObject>,
                        rep_id: i32, path: &String, package_name: &String) {

    let boxed_any = js_object.to_any();
    let boxed_class = boxed_any.downcast::<ClassObject>().unwrap();
    let class_object = *boxed_class;

    let (
        name,
        line_code,
        type_code,
        children
    ) = class_object.take();

    let repository_method_dto = RepositoryMethodDto::new(
        rep_id,
        path.clone(),
        package_name.clone(),
        line_code,
        name.clone(),
        KeyWords::EMPTY_STRING.to_string(),
        KeyWords::EMPTY_STRING.to_string(),
        serde_json::to_string(&Vec::<String>::new()).unwrap(),
        type_code.to_string(),
        0,
    );

    dto_vec.push(repository_method_dto);

    for child in children {
        prepare_output_data(dto_vec, child, rep_id, path, package_name, &name);
    }
}

fn prepare_method_object(dto_vec: &mut Vec<RepositoryMethodDto>, js_object: Box<dyn JsObject>,
                         rep_id: i32, path: &String, class_name: &String, package_name: &String) {

    let boxed_any = js_object.to_any();
    let boxed_method = boxed_any.downcast::<MethodObject>().unwrap();
    let method_object = *boxed_method;

    let (
        name,
        line_code,
        type_code,
        children,
        parameters
    ) = method_object.take();


    let repository_method_dto = RepositoryMethodDto::new(
        rep_id,
        path.clone(),
        package_name.clone(),
        line_code,
        class_name.clone(),
        name,
        KeyWords::EMPTY_STRING.to_string(),
        serde_json::to_string(&Vec::<String>::new()).unwrap(),
        type_code.to_string(),
        parameters.len(),
    );

    dto_vec.push(repository_method_dto);

    for child in children {
        prepare_output_data(dto_vec, child, rep_id, path, package_name, class_name);
    }
}


/* Logic */
fn walk_into(data: &mut Box<dyn JsObject>, node: Node, declaration_data: &mut DeclarationData) {
    match node.kind() {
        NodeKinds::CLASS_DECLARATION => add_class_declaration(data, node, declaration_data),
        NodeKinds::FUNCTION_DECLARATION => add_function_declaration(data, node, declaration_data),
        NodeKinds::ARROW_FUNCTION => add_arrow_function(data, node, declaration_data),
        NodeKinds::VARIABLE_DECLARATOR => add_function_from_variable(data, node, declaration_data),
        _ => {}
    }
}

fn add_class_declaration(parent: &mut Box<dyn JsObject>, node: Node, declaration_data: &mut DeclarationData) {

    let name_node = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let name = unwrap_or_return!(get_node_value(&name_node, declaration_data));
    let line_code_class = get_node_position(&name_node);

    let mut class_object = ClassObject::new_name(name.clone());
    class_object.set_line_code(line_code_class);
    let mut class_java_object: Box<dyn JsObject> = Box::new(class_object);

    let mut method_object = MethodObject::new(name.clone());
    method_object.set_line_code(line_code_class);
    method_object.set_type_code(JS_CONSTRUCTOR);
    let method_java_object: Box<dyn JsObject> = Box::new(method_object);

    class_java_object.add_child(method_java_object);

    if let Some(body_node) = node.child_by_field_name(NodeNames::BODY) {
        for child in body_node.named_children(&mut body_node.walk()) {
            match child.kind() {
                NodeKinds::METHOD_DEFINITION => add_method_definition(
                    &mut class_java_object,
                    child,
                    declaration_data
                ),
                _ => {}
            }
        }
    }


    parent.add_child(class_java_object);
}


fn add_function_declaration(parent: &mut Box<dyn JsObject>, node: Node, declaration_data: &mut DeclarationData) {

    let node_name = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let name = unwrap_or_return!(get_node_value(&node_name, declaration_data));
    let line = get_node_position(&node_name);

    let parameters = match node.child_by_field_name(NodeNames::PARAMETERS) {
        Some(params_node) => get_params_from_node(&params_node, declaration_data),
        None => vec![]
    };

    let mut function_object = MethodObject::new(name);
    function_object.set_line_code(line);
    function_object.set_parameters(parameters);

    let java_object: Box<dyn JsObject> = Box::new(function_object);
    parent.add_child(java_object);
}

fn add_arrow_function(parent: &mut Box<dyn JsObject>, node: Node, class_data: &mut DeclarationData) {

    let parent_node = unwrap_or_return!(node.parent());
    if parent_node.kind() != NodeKinds::VARIABLE_DECLARATOR { return; }

    let function_node = unwrap_or_return!(parent_node.child_by_field_name(NodeNames::NAME));
    let name = unwrap_or_return!(get_node_value(&function_node, class_data));
    let line = get_node_position(&function_node);

    let parameters = match node.child_by_field_name(NodeNames::PARAMETERS) {
        Some(param_node) => get_params_from_node(&param_node, class_data),
        None => match node.child_by_field_name(NodeNames::PARAMETER) {
            /* single param arrow function */
            Some(node) => vec![unwrap_or_empty_string!(get_node_value(&node, class_data))],
            None => vec![]
        }
    };

    let mut function_object = MethodObject::new(name);
    function_object.set_line_code(line);
    function_object.set_parameters(parameters);

    let java_object: Box<dyn JsObject> = Box::new(function_object);
    parent.add_child(java_object);
}

fn add_function_from_variable(parent: &mut Box<dyn JsObject>, node: Node, declaration_data: &mut DeclarationData) {

    let value_node = unwrap_or_return!(node.child_by_field_name(NodeNames::VALUE));
    if value_node.kind() != NodeKinds::FUNCTION { return; }

    /* Function name */
    let function_name_line = match value_node.child_by_field_name(NodeNames::NAME) {

        Some(node) => (
            unwrap_or_empty_string!(get_node_value(&node, declaration_data)),
            get_node_position(&node)
        ),

        None => (KeyWords::EMPTY_STRING.to_string(), 0)
    };

    /* Var name*/
    let var_name_line = match node.child_by_field_name(NodeNames::NAME) {

        Some(node) => (
            unwrap_or_empty_string!(get_node_value(&node, declaration_data)),
            get_node_position(&node)
        ),

        None => (KeyWords::EMPTY_STRING.to_string(), 0)
    };

    let (function_name, function_line_code) = function_name_line;
    let (var_name, var_line_code) = var_name_line;

    /* Parameters (common) */
    let parameters = match value_node.child_by_field_name(NodeNames::PARAMETERS) {

        Some(parameters_node) => get_params_from_node(
            &parameters_node,
            declaration_data
        ),

        None => vec![]
    };

    if var_name != KeyWords::EMPTY_STRING {
        let mut var_object = MethodObject::new(var_name);
        var_object.set_line_code(var_line_code);
        var_object.set_parameters(parameters.clone());
        let js_object: Box<dyn JsObject> = Box::new(var_object);
        parent.add_child(js_object);
    }

    if function_name != KeyWords::EMPTY_STRING {
        let mut function_object = MethodObject::new(function_name);
        function_object.set_line_code(function_line_code);
        function_object.set_parameters(parameters);
        let js_object: Box<dyn JsObject> = Box::new(function_object);
        parent.add_child(js_object);
    }
}

fn add_method_definition(parent: &mut Box<dyn JsObject>, node: Node, class_data: &mut DeclarationData) {

    let name_node = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let mut name = unwrap_or_return!(get_node_value(&name_node, class_data));
    let parameters_node = unwrap_or_return!(node.child_by_field_name(NodeNames::PARAMETERS));
    let parameters = get_params_from_node(&parameters_node, class_data);

    let current_class_name = parent.get_name();

    let mut type_code = JS_METHOD;
    if name == KeyWords::CONSTRUCTOR_IDENTIFIER {
        name = current_class_name;
        type_code = JS_CONSTRUCTOR;
    }

    let mut method_object = MethodObject::new_code(name, type_code);
    method_object.set_line_code(get_node_position(&node));
    method_object.set_parameters(parameters);
    let js_object: Box<dyn JsObject> = Box::new(method_object);
    parent.add_child(js_object);
}


/* Helpers */
fn get_node_value(node: &Node, parse_data: &mut DeclarationData) -> Option<String> {

    let source_code = parse_data.source_code();
    let bytes = source_code.as_bytes();
    let mut node_bytes = &bytes[node.start_byte()..node.end_byte()];
    let mut node_string = String::new();

    node_bytes
        .read_to_string(&mut node_string)
        .expect(&format!("RUST UNRECOVERABLE ERROR: Unable to read source code. Path file: {}", parse_data.path_file()));

    if node_string.len() < MAX_TOKEN_LENGTH {
        Some(node_string)
    } else {
        None
    }
}

fn get_params_from_node(node: &Node, parse_data: &mut DeclarationData) -> Vec<String> {

    let mut parameters = vec![];

    for child in node.named_children(&mut node.walk()) {
        let parameter_option = match child.kind() {
            NodeKinds::IDENTIFIER => get_node_value(&child, parse_data),
            NodeKinds::ASSIGNMENT_PATTERN => {
                let param_node = child.child_by_field_name(NodeNames::LEFT);
                match param_node {
                    Some(node) => get_node_value(&node, parse_data),
                    None => None
                }
            }
            // NodeKinds::REST_PATTERN => { Any pattern in DAO }
            _ => None
        };

        if let Some(parameter) = parameter_option {
            parameters.push(parameter);
        };
    }

    parameters
}

fn get_node_position(node: &Node) -> usize {
    node.start_position().row + 1
}

#[cfg(test)]
mod js_code_declaration_tests {

    use super::*;
    use tree_sitter::Parser;
    use std::fs;

    #[test]
    pub fn test_get_repository_method_dto() {
        let mut code = fs::read_to_string("../../resources/test_files/js/1.js.txt").unwrap();
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_javascript::language())
            .expect("ERROR: Unable to load JavaScript grammar");


        let tree = parser.parse(&mut code, None).unwrap();
        println!("{}", tree.root_node().to_sexp());
        let method_dto = get_repository_method_dto(code,
                                                   tree, "test".to_string(),
                                                   0);

        println!("{}", serde_json::to_string_pretty(&method_dto).unwrap());
        assert_eq!(method_dto.len(), 21);
    }
}
