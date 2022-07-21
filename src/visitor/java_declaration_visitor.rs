use crate::model::java_object::{JavaObject, MethodObject, ClassObject, PackageObject};
use crate::dto::repository_method_dto::RepositoryMethodDto;
use crate::model::java_object::CodeType::{JAVA_METHOD, JAVA_ENUM, JAVA_CLASS, JAVA_INTERFACE, JAVA_PACKAGE, JAVA_CONSTRUCTOR};
use tree_sitter::{Node, Tree};
use std::io::Read;
use crate::unwrap_or_return;
use crate::unwrap_or_empty_string;

const MAX_TOKEN_LENGTH: usize = 250;

struct KeyWords;
impl KeyWords {
    const VOID: &'static str = "void";
    const EMPTY_STRING:&'static str = "";
}

struct NodeKinds;
impl NodeKinds {
    const PACKAGE_DECLARATION: &'static str = "package_declaration";
    const INTERFACE_DECLARATION: &'static str = "interface_declaration";
    const CLASS_DECLARATION: &'static str = "class_declaration";
    const CONSTRUCTOR_DECLARATION: &'static str = "constructor_declaration";
    const ENUM_DECLARATION: &'static str = "enum_declaration";
    const METHOD_DECLARATION: &'static str = "method_declaration";
    const SCOPED_IDENTIFIER: &'static str = "scoped_identifier";
    const IDENTIFIER: &'static str = "identifier";
    const ENUM_BODY_DECLARATIONS: &'static str = "enum_body_declarations";
    const FORMAL_PARAMETERS: &'static str = "formal_parameters";
    const VARIABLE_DECLARATOR: &'static str = "variable_declarator";
    const MODIFIERS: &'static str = "modifiers";
}

struct NodeNames;
impl NodeNames {
    const NAME: &'static str = "name";
    const BODY: &'static str = "body";
    const TYPE: &'static str = "type";
}

struct ClassData {
    source_code: String,
    path: String,
}

impl ClassData {

    fn new(source_code: String, path_file: String) -> ClassData {
        ClassData {
            source_code,
            path: path_file,
        }
    }

    fn source_code(&self) -> &String {
        &self.source_code
    }

    fn path_file(&self) -> &String {
        &self.path
    }
}


/* Main function */
pub fn get_repository_method_dto(source_code: String, tree: Tree, path: String, rep_id: i32) -> Vec<RepositoryMethodDto> {

    let node = tree.root_node();
    let mut class_data = ClassData::new(source_code, path.clone());
    let mut data = find_package_declaration(&mut class_data, &node);

    parse_node(&mut data, &node, &mut class_data);

    let mut method_dto_vec = vec![];

    prepare_output_data(
        &mut method_dto_vec,
        data,
        rep_id,
        &path,
        &KeyWords::EMPTY_STRING.to_string(),
        &KeyWords::EMPTY_STRING.to_string()
    );

    return method_dto_vec;
}

fn parse_node(parent: & mut Box<dyn JavaObject>, node: &Node, class_data: & mut ClassData){

    for child in node.children(& mut node.walk()) {
        match child.kind() {
            NodeKinds::CLASS_DECLARATION => add_class_declaration(&child, class_data, parent),
            NodeKinds::INTERFACE_DECLARATION => add_interface_declaration(&child, class_data, parent),
            NodeKinds::ENUM_DECLARATION => add_enum_declaration(&child, class_data, parent),
            NodeKinds::METHOD_DECLARATION => add_method_declaration(&child, class_data, parent),
            NodeKinds::CONSTRUCTOR_DECLARATION => add_constructor_declaration(&child, class_data, parent),
            &_ => {}
        }
    }

}

fn add_constructor_declaration(node: &Node, class_data: &mut ClassData, parent: &mut Box<dyn JavaObject>) {

    let name_node = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let name = unwrap_or_empty_string!(get_node_value(&name_node, class_data));

    let mut constructor_object = MethodObject::new(name, JAVA_CONSTRUCTOR);
    constructor_object.set_line_code(get_node_position(&name_node));
    constructor_object.set_parameters(get_parameters_from_node(&node, class_data));
    constructor_object.set_modifiers(get_modifiers_from_node(&node, class_data));
    let java_object: Box<dyn JavaObject> = Box::new(constructor_object);

    parent.add_child(java_object);
}

fn add_method_declaration(node: &Node, class_data: &mut ClassData, parent: &mut Box<dyn JavaObject>) {

    let name_node = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let method_name = unwrap_or_return!(get_node_value(&name_node, class_data));

    let mut method_object = MethodObject::new(method_name, JAVA_METHOD);
    method_object.set_line_code(get_node_position(&name_node));
    method_object.set_parameters(get_parameters_from_node(&node, class_data));
    method_object.set_modifiers(get_modifiers_from_node(&node, class_data));

    if let Some(output_param) = get_output_param_from_node(&node, class_data) {
        method_object.set_output_parameter(output_param);
    }
    let java_object: Box<dyn JavaObject> = Box::new(method_object);

    parent.add_child(java_object);
}

fn add_class_declaration(node: &Node, class_data: &mut ClassData, parent: &mut Box<dyn JavaObject>) {

    let node_name = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let class_name = unwrap_or_empty_string!(get_node_value(&node_name, class_data));
    let line_position = get_node_position(&node_name);
    let modifiers = get_modifiers_from_node(&node, class_data);

    let mut class_object = ClassObject::new(class_name.clone());
    class_object.set_line_code(line_position);
    class_object.set_type_code(JAVA_CLASS);
    class_object.set_modifiers(modifiers.clone());
    let mut java_class_object: Box<dyn JavaObject> = Box::new(class_object);

    /* Zero arg constructor */
    let mut constructor_object = MethodObject::default();
    constructor_object.set_name(class_name);
    constructor_object.set_type_code(JAVA_CONSTRUCTOR);
    constructor_object.set_line_code(line_position);
    constructor_object.set_modifiers(modifiers);
    let java_constr_object: Box<dyn JavaObject> = Box::new(constructor_object);

    java_class_object.add_child(java_constr_object);

    if let Some(class_body) = node.child_by_field_name(NodeNames::BODY) {
        parse_node(& mut java_class_object, &class_body, class_data);
    }

    parent.add_child(java_class_object);

}

fn add_interface_declaration(node: &Node, class_data: &mut ClassData, parent: &mut Box<dyn JavaObject>) {

    let node_name = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let name = unwrap_or_empty_string!(get_node_value(&node_name, class_data));

    let mut class_object = ClassObject::new(name);
    class_object.set_line_code(get_node_position(&node_name));
    class_object.set_type_code(JAVA_INTERFACE);

    let mut java_object: Box<dyn JavaObject> = Box::new(class_object);

    if let Some(class_body) = node.child_by_field_name(NodeNames::BODY) {
        parse_node(& mut java_object, &class_body, class_data);
    }

    parent.add_child(java_object);

}

fn add_enum_declaration(node: &Node, class_data: &mut ClassData, parent: &mut Box<dyn JavaObject>) {

    let node_name = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let name = unwrap_or_empty_string!(get_node_value(&node_name, class_data));

    let mut class_object = ClassObject::new(name);
    class_object.set_line_code(get_node_position(&node_name));
    class_object.set_type_code(JAVA_ENUM);
    let mut java_object: Box<dyn JavaObject> = Box::new(class_object);

    if let Some(enum_body) = node.child_by_field_name(NodeNames::BODY) {
        let enum_body_opt =  get_child_node_by_kind(
            &enum_body,
            NodeKinds::ENUM_BODY_DECLARATIONS
        );
        if let Some(enum_body_declarations) = enum_body_opt {
            parse_node(& mut java_object, &enum_body_declarations, class_data);
        }
    }

    parent.add_child(java_object);

}

fn find_package_declaration(class_data: &mut ClassData, root_node: &Node) -> Box<dyn JavaObject> {


    if let Some(node) = get_child_node_by_kind(root_node, NodeKinds::PACKAGE_DECLARATION) {

        if let Some(name_node) = get_child_node_by_kind(&node,  NodeKinds::SCOPED_IDENTIFIER) {
            if let Some(scoped_identifier_value) = get_node_value(&name_node, class_data) {
                let mut package_object = PackageObject::new();
                package_object.set_name(scoped_identifier_value);
                package_object.set_line_code(get_node_position(&node));
                return Box::new(package_object);
            }
        }

        if let Some(name_node) = get_child_node_by_kind(&node, NodeKinds::IDENTIFIER) {
            if let Some(identifier_value) = get_node_value(&name_node, class_data) {
                let mut package_object = PackageObject::new();
                package_object.set_name(identifier_value);
                package_object.set_line_code(get_node_position(&node));
                return Box::new(package_object);
            }
        }

    }

    return Box::new(PackageObject::new());

}

fn prepare_output_data(dto_vec: &mut Vec<RepositoryMethodDto>, java_object: Box<dyn JavaObject>,
                       rep_id: i32, path: &String, package_name: &String, class_name: &String) {
    
    match java_object.type_code() {
        JAVA_PACKAGE => prepare_package_object(dto_vec, java_object, rep_id, path, class_name),
        JAVA_CLASS | JAVA_ENUM | JAVA_INTERFACE => prepare_class_object(dto_vec, java_object, rep_id, path, package_name),
        JAVA_METHOD | JAVA_CONSTRUCTOR => prepare_method_object(dto_vec, java_object, rep_id, path, package_name, class_name),
        &_ => {}
    }
}

fn prepare_package_object(dto_vec: &mut Vec<RepositoryMethodDto>, java_object: Box<dyn JavaObject>,
                          rep_id: i32, path: &String, class_name: &String) {

    let boxed_any = java_object.to_any();
    let boxed_package = boxed_any.downcast::<PackageObject>().unwrap();
    let package_object = *boxed_package;

    let (
        name,
        line_code,
        type_code,
        children,
        modifiers
    ) = package_object.take();

    let repository_method_dto = RepositoryMethodDto::new(
        rep_id,
        path.clone(),
        name.clone(),
        line_code,
        KeyWords::EMPTY_STRING.to_string(),
        KeyWords::EMPTY_STRING.to_string(),
        KeyWords::EMPTY_STRING.to_string(),
        serde_json::to_string(&modifiers).unwrap_or(KeyWords::EMPTY_STRING.to_string()),
        type_code.to_string(),
        0,
    );

    dto_vec.push(repository_method_dto);

    for child in children {
        prepare_output_data(dto_vec, child, rep_id, path, &name, &class_name);
    }
}

fn prepare_class_object(dto_vec: &mut Vec<RepositoryMethodDto>, java_object: Box<dyn JavaObject>,
                        rep_id: i32, path: &String, package_name: &String) {

    let boxed_any = java_object.to_any();
    let boxed_class = boxed_any.downcast::<ClassObject>().unwrap();
    let class_object = *boxed_class;

    let (
        name,
        line_code,
        type_code,
        children,
        modifiers,
    ) = class_object.take();

    let repository_method_dto = RepositoryMethodDto::new(
        rep_id,
        path.clone(),
        package_name.clone(),
        line_code,
        name.clone(),
        KeyWords::EMPTY_STRING.to_string(),
        KeyWords::EMPTY_STRING.to_string(),
        serde_json::to_string(&modifiers).unwrap_or(KeyWords::EMPTY_STRING.to_string()),
        type_code.to_string(),
        0,
    );

    dto_vec.push(repository_method_dto);

    for child in children {
        prepare_output_data(dto_vec, child, rep_id, path, package_name, &name);
    }
    
}

fn prepare_method_object(dto_vec: &mut Vec<RepositoryMethodDto>, java_object: Box<dyn JavaObject>,
                         rep_id: i32, path: &String, package_name: &String, class_name: &String) {

    let boxed_any = java_object.to_any();
    let boxed_method = boxed_any.downcast::<MethodObject>().unwrap();
    let method_object = *boxed_method;

    let (
        name,
        line_code,
        type_code,
        children,
        modifiers,
        parameters,
        _output_param,
    ) = method_object.take();

    let repository_method_dto = RepositoryMethodDto::new(
        rep_id,
        path.clone(),
        package_name.clone(),
        line_code,
        class_name.clone(),
        name,
        KeyWords::EMPTY_STRING.to_string(),
        serde_json::to_string(&modifiers).unwrap_or(KeyWords::EMPTY_STRING.to_string()),
        type_code.to_string(),
        parameters.len(),
    );

    dto_vec.push(repository_method_dto);

    for child in children {
        prepare_output_data(dto_vec, child, rep_id, path, package_name, class_name);
    }
}

fn get_node_value(node: &Node, class_data: &mut ClassData) -> Option<String> {

    let source = class_data.source_code();
    let bytes = source.as_bytes();
    let mut node_bytes = &bytes[node.start_byte()..node.end_byte()];
    let mut node_string = String::new();

    node_bytes
        .read_to_string(&mut node_string)
        .expect(&format!("RUST UNRECOVERABLE ERROR: Unable to read source code. Path file: {}", class_data.path_file()));

    if node_string.len() < MAX_TOKEN_LENGTH {
        Some(node_string)
    } else {
        None
    }
}

/* Helpers */
fn get_child_node_by_kind<'time_spec>(node: &'time_spec Node, kind: &'time_spec str) -> Option<Node<'time_spec>> {
    node
        .children(&mut node.walk())
        .filter(|x| { x.kind() == kind })
        .next()
}

fn get_node_position(node: &Node) -> usize {
    node.start_position().row + 1
}

fn get_parameters_from_node(node: &Node, class_data: &mut ClassData) -> Vec<String> {

    let mut parameters = vec![];

    let params_opt = get_child_node_by_kind(&node, NodeKinds::FORMAL_PARAMETERS);
    if params_opt.is_none() {
        return parameters;
    }

    let params_node = params_opt.unwrap();
    for child in params_node.named_children(&mut params_node.walk()) {

        if let Some(param_name) = child.child_by_field_name(NodeNames::NAME) {
            if let Some(parameter) = get_node_value(&param_name, class_data) {
                parameters.push(parameter);
            }
        }

        if let Some(var) = get_child_node_by_kind(&child, NodeKinds::VARIABLE_DECLARATOR) {
            if let Some(spread_param) = var.child_by_field_name(NodeNames::NAME) {
                if let Some(spread_param_name) = get_node_value(&spread_param, class_data) {
                    parameters.push(spread_param_name);
                }
            }
        }
    }

    return parameters;
}

fn get_output_param_from_node(node: &Node, class_data: &mut ClassData) -> Option<String> {

    if let Some(output_param_node) = node.child_by_field_name(NodeNames::TYPE) {
        if let Some(value) = get_node_value(&output_param_node, class_data) {
            if value != KeyWords::VOID {
                return Some(value);
            }
        }
    }
    return None;
}

fn get_modifiers_from_node(node: &Node, class_data: &mut ClassData) -> Vec<String> {

    let mut modifiers = vec![];

    if let Some(modifiers_node) = get_child_node_by_kind(&node,  NodeKinds::MODIFIERS) {
        let mut tree_cursor = modifiers_node.walk();
        for child in modifiers_node.children(&mut tree_cursor) {
            if let Some(modifier) = get_node_value(&child, class_data) {
                modifiers.push(modifier);
            }
        }
    }
    return modifiers;
}

#[cfg(test)]
mod java_code_declaration_tests {

    use super::*;
    use tree_sitter::Parser;
    use std::fs;
    use crate::dto::invocation_structure::InvocationStructure;


    #[test]
    pub fn test_get_repository_method_dto() {
        let mut code = fs::read_to_string("resources/test_files/java/1.java.txt").unwrap();
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_java::language()).expect("ERROR: Unable to load Java grammar");
        let tree = parser.parse(&mut code, None).unwrap();
        println!("{},  ", tree.root_node().to_sexp());
        let mut repository_method_dto = get_repository_method_dto(code,
                                                                  tree, "test".to_string(),
                                                                  0);
        for dto in repository_method_dto {
            println!("{}", serde_json::to_string_pretty(&dto).unwrap())
        }

    }

    #[derive(Default)]
    pub struct MyClass {
        name: String
    }


}

