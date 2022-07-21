use std::io::Read;
use tree_sitter::{Node, Tree};
use crate::dto::repository_method_dto::RepositoryMethodDto;
use crate::model::python_object::CodeType::{PYTHON_ENUM, PYTHON_METHOD, PYTHON_CLASS, PYTHON_CONSTRUCTOR, PYTHON_PACKAGE};
use crate::model::python_object::{ClassObject, CodeType, MethodObject, PackageObject, PythonObject};
use crate::unwrap_or_return;
use crate::unwrap_or_empty_string;

const MAX_TOKEN_LENGTH: usize = 250;

struct NodeKinds;
impl NodeKinds {
    const CLASS_DEFINITION:&'static str = "class_definition";
    const FUNCTION_DEFINITION:&'static str = "function_definition";
    const IDENTIFIER:&'static str = "identifier";
    const TYPED_PARAMETER:&'static str = "typed_parameter";
    const DEFAULT_PARAMETER:&'static str = "default_parameter";
    const TYPED_DEFAULT_PARAMETER:&'static str = "typed_default_parameter";

}

struct NodeNames;
impl NodeNames {
    const NAME:&'static str = "name";
    const BODY:&'static str = "body";
    const PARAMETERS:&'static str = "parameters";
    const RETURN_TYPE:&'static str = "return_type";
    const SUPERCLASSES:&'static str = "superclasses";
}

struct KeyWords;
impl KeyWords {
    const CONSTRUCTOR_NAME: &'static str = "__init__";
    const SELF_SPECIFIER:&'static str = "self";
    const CLS_SPECIFIER:&'static str = "cls";
    const EMPTY_STRING:&'static str = "";
    const ENUM:&'static str = "Enum";
    const INT_ENUM:&'static str = "IntEnum";
    const FLAG:&'static str = "Flag";
    const INT_FLAG:&'static str = "IntFlag";
}

struct DeclarationData {
    source_code: String,
    path: String,
}

impl DeclarationData {
    fn new(source_code: String, path: String) -> Self {
        Self { source_code, path }
    }

    fn source_code(&self) -> &String {
        &self.source_code
    }

    fn path_file(&self) -> &String {
        &self.path
    }
}

/* Main function */
pub fn get_repository_method_dto(source: String, tree: Tree, path: String, rep_id: i32) -> Vec<RepositoryMethodDto> {

    let package_name = convert_path_to_package(&path);
    let mut declaration_data = DeclarationData::new(source, path.clone());
    let mut python_object:Box<dyn PythonObject> = Box::new(PackageObject::new(package_name));

    parse_root_node(tree.root_node(),  &mut declaration_data, & mut python_object);

    let mut method_dto_vec = vec![];

    prepare_output_data(
        &mut method_dto_vec,
        python_object,
        rep_id,
        &path,
        &String::new(),
        &String::new()
    );

    return method_dto_vec;
}

/* Logic */
fn parse_root_node(node: Node, declaration_data: &mut DeclarationData, parent: & mut Box<dyn PythonObject>) {
    for child in node.named_children(&mut node.walk()) {
        add_statement(child, declaration_data, parent);
    }
}

fn add_statement(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn PythonObject>) {
    match node.kind() {
        NodeKinds::CLASS_DEFINITION => add_class_definition(node, declaration_data, parent),
        NodeKinds::FUNCTION_DEFINITION => add_function_definition(node, declaration_data, parent),
        &_ => {}
    }

}

fn add_class_definition(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn PythonObject>) {

    let name_node = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let name = unwrap_or_empty_string!(get_node_value(&name_node, declaration_data));
    let body_node = unwrap_or_return!(node.child_by_field_name(NodeNames::BODY));
    let line_number = get_line_number(&name_node);
    let type_code = get_type_code_from_class_node(&node, declaration_data);

    let mut class_object = ClassObject::default();
    class_object.set_name(name.clone());
    class_object.set_line_number(line_number);
    class_object.set_type_code(type_code);
    let mut python_class:Box<dyn PythonObject> = Box::new(class_object);

    let mut constructor_object = MethodObject::default();
    constructor_object.set_name(name);
    constructor_object.set_type_code(CodeType::PYTHON_CONSTRUCTOR);
    constructor_object.set_line_number(line_number);
    let python_constructor:Box<dyn PythonObject> = Box::new(constructor_object);

    python_class.add_child(python_constructor);

    for child in body_node.named_children(&mut node.walk()) {
        add_statement(child, declaration_data, & mut python_class);
    }

    parent.add_child(python_class);
}

fn add_function_definition(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn PythonObject>) {

    let name_node = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let params_node = unwrap_or_return!(node.child_by_field_name(NodeNames::PARAMETERS));
    let body_node = unwrap_or_return!(node.child_by_field_name(NodeNames::BODY));

    let mut name = unwrap_or_empty_string!(get_node_value(&name_node, declaration_data));
    let parameters = get_params_from_param_node(params_node, declaration_data);
    let line_number = get_line_number(&node);

    let mut type_code = PYTHON_METHOD;
    if name == KeyWords::CONSTRUCTOR_NAME {
        name = parent.get_name();
        type_code = PYTHON_CONSTRUCTOR;
    }

    let output_param = match node.child_by_field_name(NodeNames::RETURN_TYPE) {
        Some(return_node) => unwrap_or_empty_string!(get_node_value(&return_node, declaration_data)),
        None => KeyWords::EMPTY_STRING.to_string()
    };
    
    let method_object = MethodObject::new(
        name,
        type_code,
        parameters,
        line_number,
        output_param
    );
    
    let mut python_method:Box<dyn PythonObject> = Box::new(method_object);
    
    for child in body_node.named_children(&mut node.walk()) {
        add_statement(child, declaration_data, & mut python_method);
    }
    
    parent.add_child(python_method);
}

/* Data conversion */
fn prepare_output_data(dto_vec: &mut Vec<RepositoryMethodDto>, python_object: Box<dyn PythonObject>,
                       rep_id: i32, path: &String, package_name: &String, class_name: &String) {
    
    match python_object.type_code() {
        PYTHON_PACKAGE => prepare_package_object(dto_vec, python_object, rep_id, path, class_name),
        PYTHON_CLASS | PYTHON_ENUM => prepare_class_object(dto_vec, python_object, rep_id, path, package_name),
        PYTHON_METHOD | PYTHON_CONSTRUCTOR
        => prepare_method_object(dto_vec, python_object, rep_id, path, package_name, class_name),
        &_ => {}
    }
    
}

fn prepare_package_object(dto_vec: &mut Vec<RepositoryMethodDto>, python_object: Box<dyn PythonObject>,
                          rep_id: i32, path: &String, class_name: &String) {

    let boxed_any = python_object.to_any();
    let boxed_package = boxed_any.downcast::<PackageObject>().unwrap();
    let package_object = *boxed_package;

    let (
        name,
        line_code,
        type_code,
        children
    ) = package_object.take();

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

fn prepare_class_object(dto_vec: &mut Vec<RepositoryMethodDto>, python_object: Box<dyn PythonObject>,
                        rep_id: i32, path: &String, package_name: &String) {

    let boxed_any = python_object.to_any();
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

fn prepare_method_object(dto_vec: &mut Vec<RepositoryMethodDto>, python_object: Box<dyn PythonObject>,
                         rep_id: i32, path: &String, package_name: &String, class_name: &String) {

    let boxed_any = python_object.to_any();
    let boxed_method = boxed_any.downcast::<MethodObject>().unwrap();
    let method_object = *boxed_method;

    let (
        name,
        line_code,
        type_code,
        children,
        parameters,
        _output_parameter,
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

/* Helpers */
fn get_type_code_from_class_node(node: &Node, declaration_data: & mut DeclarationData) -> CodeType {

    /* According to https://docs.python.org/3/library/enum.html :
    Enumerations are created using the class syntax, which makes them easy to read and write.
    To define an enumeration, subclass Enum/IntEnum/IntFlag/Flag. */
    let superclasses_opt = node.child_by_field_name(NodeNames::SUPERCLASSES);
    if superclasses_opt.is_none() {
        return PYTHON_CLASS;
    }

    let superclasses = superclasses_opt.unwrap();

    for child in superclasses.named_children(& mut superclasses.walk()) {
        match unwrap_or_empty_string!(get_node_value(& child, declaration_data)).as_str() {
            KeyWords::ENUM | KeyWords::INT_ENUM |
            KeyWords::FLAG | KeyWords::INT_FLAG => return PYTHON_ENUM,
            _ => {}
        }
    }

    return PYTHON_CLASS;

}

fn get_params_from_param_node(node: Node, declaration_data: &DeclarationData) -> Vec<String> {

    let mut parameters = vec![];

    for child in node.named_children(&mut node.walk()) {

        let mut parameter = KeyWords::EMPTY_STRING.to_string();

        match child.kind() {

            NodeKinds::IDENTIFIER =>
                parameter = unwrap_or_empty_string!(get_node_value(&child, declaration_data)),

            NodeKinds::DEFAULT_PARAMETER | NodeKinds::TYPED_DEFAULT_PARAMETER =>
                parameter = get_name_from_default_param(&child, declaration_data),

            NodeKinds::TYPED_PARAMETER =>
                parameter = get_name_from_typed_param(&child, declaration_data),

            &_ => ( /* "list_splat_pattern" | "dictionary_splat_pattern" => { Rest Pattern } */)
        }

        if parameter != KeyWords::EMPTY_STRING && parameter != KeyWords::CLS_SPECIFIER
            && parameter != KeyWords::SELF_SPECIFIER
        {
            parameters.push(parameter)
        }
    }

    parameters
}

fn get_name_from_default_param(node: &Node, declaration_data: &DeclarationData) -> String {

    let node_opt = node.child_by_field_name(NodeNames::NAME);
    if node_opt.is_none() {
       return KeyWords::EMPTY_STRING.to_string();
    }
    return unwrap_or_empty_string!(get_node_value(&node_opt.unwrap(), declaration_data));
}

fn get_name_from_typed_param(node: &Node, declaration_data: &DeclarationData) -> String {

    let identifier_opt = get_child_node_by_kind(&node, NodeKinds::IDENTIFIER);
    if identifier_opt.is_none() {
        return KeyWords::EMPTY_STRING.to_string();
    }
    return unwrap_or_empty_string!(get_node_value(&identifier_opt.unwrap(), declaration_data))
}

fn get_child_node_by_kind<'time>(node: &'time Node, kind: &'time str) -> Option<Node<'time>> {
    node
        .children(&mut node.walk())
        .filter(|x| { x.kind() == kind })
        .next()
}

fn get_node_value(node: &Node, declaration_data: &DeclarationData) -> Option<String> {

    let source = declaration_data.source_code();
    let bytes = source.as_bytes();
    let mut node_bytes = &bytes[node.start_byte()..node.end_byte()];
    let mut node_string = String::new();

    node_bytes
        .read_to_string(&mut node_string)
        .expect(&format!(
            "RUST UNRECOVERABLE ERROR: Unable to read source code. Path file: {}",
            declaration_data.path_file())
        );

    if node_string.len() < MAX_TOKEN_LENGTH {
        Some(node_string)
    } else {
        None
    }
}

fn get_line_number(node: &Node) -> usize {
    node.start_position().row + 1
}

fn convert_path_to_package(path: &String) -> String {

    const PY_EXTENSION: &str = ".py";
    const EMPTY_STRING: &str = "";
    const SLASH: &str = "/";
    const DOT: &str = ".";

    return path.replace(PY_EXTENSION, EMPTY_STRING).replace(SLASH, DOT);
}


#[cfg(test)]
mod python_code_declaration_tests {

    use super::*;
    use tree_sitter::Parser;
    use std::fs;

    #[test]
    pub fn test_get_repository_method_dto() {
        let mut code = fs::read_to_string("../../resources/test_files/python/2.py.txt").unwrap();
        let mut parser = Parser::new();

        parser
            .set_language(tree_sitter_python::language())
            .expect("ERROR: Unable to load PYTHON grammar");

        let tree = parser.parse(&mut code, None).unwrap();
        println!("{},  ", tree.root_node().to_sexp());

        let repository_method_dto =
            get_repository_method_dto(code, tree, "test".to_string(), 0);

        for dto in repository_method_dto {
            let json = serde_json::to_string_pretty(&dto).unwrap();
            print!("{}", json);
        }
    }
}