use crate::dto::repository_method_dto::{RepositoryMethodDto, RepositoryMethodDtoBuilder};
use tree_sitter::{Tree, Node};
use std::io::Read;
use crate::unwrap_or_return;
use crate::unwrap_or_empty_string;
use crate::model::cpp_object::{ClassObject, CppObject, PackageObject, MethodObject, MethodReturnType, CodeType, ObjectType};
use crate::model::cpp_object::CodeType::{CPP_METHOD, CPP_CONSTRUCTOR, CPP_ENUM, CPP_PACKAGE, CPP_CLASS};
use crate::model::cpp_object::MethodReturnType::{Value, Reference, Pointer};
use crate::model::cpp_object::ObjectType::{Definition, Declaration};

const MAX_TOKEN_LENGTH: usize = 250;

struct NodeKinds;
impl NodeKinds {
    const ERROR: &'static str = "ERROR";
    const CLASS_SPECIFIER: &'static str = "class_specifier";
    const STRUCT_SPECIFIER: &'static str = "struct_specifier";
    const ENUM_SPECIFIER: &'static str = "enum_specifier";
    const FUNCTION_DEFINITION: &'static str = "function_definition";
    const FRIEND_DECLARATION: &'static str = "friend_declaration";
    const NAMESPACE_DEFINITION: &'static str = "namespace_definition";
    const TEMPLATE_DECLARATION: &'static str = "template_declaration";
    const DECLARATION_LIST: &'static str = "declaration_list";
    const UNION_SPECIFIER: &'static str = "union_specifier";
    const LINKAGE_SPECIFICATION: &'static str = "linkage_specification";
    const DECLARATION: &'static str = "declaration";
    const PREPROC_IF: &'static str = "preproc_if";
    const PREPROC_IFDEF: &'static str = "preproc_ifdef";
    const PREPROC_FUNCTION_DEF: &'static str = "preproc_function_def";
    const FIELD_DECLARATION: &'static str = "field_declaration";
    const IDENTIFIER: &'static str = "identifier";
    const PARAMETER_DECLARATION: &'static str = "parameter_declaration";
    const OPTIONAL_PARAMETER_DECLARATION: &'static str = "optional_parameter_declaration";
    const FIELD_IDENTIFIER: &'static str = "field_identifier";
    const DESTRUCTOR_NAME: &'static str = "destructor_name";
    const SCOPED_IDENTIFIER: &'static str = "scoped_identifier";
    const NAMESPACE_IDENTIFIER: &'static str = "namespace_identifier";
    const TEMPLATE_TYPE: &'static str = "template_type";
    const FUNCTION_DECLARATOR: &'static str = "function_declarator";
    const TYPE_IDENTIFIER: &'static str = "type_identifier";
    const SCOPED_TYPE_IDENTIFIER: &'static str = "scoped_type_identifier";
    const SCOPED_NAMESPACE_IDENTIFIER: &'static str = "scoped_namespace_identifier";
    const PRIMITIVE_TYPE: &'static str = "primitive_type";
    const SIZED_TYPE_SPECIFIER: &'static str = "sized_type_specifier";
    const REFERENCE_DECLARATOR: &'static str = "reference_declarator";
    const POINTER_DECLARATOR: &'static str = "pointer_declarator";
}
struct NodeNames;
impl NodeNames {
    const BODY: &'static str = "body";
    const NAME: &'static str = "name";
    const DECLARATOR: &'static str = "declarator";
    const PARAMETERS: &'static str = "parameters";
    const NAMESPACE: &'static str = "namespace";
    const TYPE: &'static str = "type";
}

struct KeyWords;
impl KeyWords {
    const EMPTY_STRING: &'static str = "";
    const VOID: &'static str = "void";
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


pub fn get_repository_method_dto(source_code: String, tree: Tree,
                                 path: String, rep_id: i32) -> Vec<RepositoryMethodDto> {

    let mut declaration_data = DeclarationData::new(source_code, path.clone());
    let package = PackageObject::new(path.clone());
    let mut cpp_object: Box<dyn CppObject> = Box::new(package);

    parse_top_level_node(tree.root_node(), &mut declaration_data, &mut cpp_object);

    let mut method_dto_vec = vec![];

    prepare_output_data(&mut method_dto_vec, cpp_object, rep_id,
        &path, &String::new(), &String::new());

    return method_dto_vec;
}

fn parse_top_level_node(root: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn CppObject>) {
    for node in root.named_children(&mut root.walk()) {
        add_top_level_item(node, declaration_data, parent);
    }
}

fn add_top_level_item(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn CppObject>) {

    match node.kind() {

        NodeKinds::NAMESPACE_DEFINITION => add_name_space_definition(node, declaration_data, parent),
        NodeKinds::ENUM_SPECIFIER => add_enum_specifier(node, declaration_data, parent),
        NodeKinds::CLASS_SPECIFIER | NodeKinds::STRUCT_SPECIFIER | NodeKinds::UNION_SPECIFIER
        => add_struct_spec(node, declaration_data, parent),
        NodeKinds::FUNCTION_DEFINITION => add_function_definition(node, declaration_data, parent),
        NodeKinds::DECLARATION => add_declaration(node, declaration_data, parent),
        NodeKinds::LINKAGE_SPECIFICATION => add_linkage_specification(node, declaration_data, parent),
        NodeKinds::PREPROC_IF => add_preproc_if(node, declaration_data, parent),
        NodeKinds::PREPROC_IFDEF => add_preproc_ifdef(node, declaration_data, parent),
        NodeKinds::PREPROC_FUNCTION_DEF => add_preproc_func_def(node, declaration_data, parent),
        NodeKinds::TEMPLATE_DECLARATION => add_template_declaration(node, declaration_data, parent),

        /* Found error node - trying to parse it's children */
        NodeKinds::ERROR => parse_top_level_node(node, declaration_data, parent),
        _ => {}
    }
}

/* Structures */
fn add_name_space_definition(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn CppObject>) {

    let decl_list_node = unwrap_or_return!( node.child_by_field_name(NodeNames::BODY));
    let namespace_node = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let namespace_name = unwrap_or_empty_string!(get_node_value(&namespace_node, declaration_data));
    let line_number = get_line_number(&namespace_node);
    let class_object = ClassObject::new_class(namespace_name, line_number);
    let mut cpp_object: Box<dyn CppObject> = Box::new(class_object);
    add_declaration_list(decl_list_node, declaration_data, &mut cpp_object);
    parent.add_child(cpp_object);
}

fn add_struct_spec(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn CppObject>) {

    let name_opt = node.child_by_field_name(NodeNames::NAME);
    if name_opt.is_none() {
        /* Anonymous structure */
        let field_declaration_list = unwrap_or_return!(node.child_by_field_name(NodeNames::BODY));
        add_field_declaration_list(field_declaration_list, declaration_data, parent);
        return;
    }

    /* Class declaration */
    let name_node = name_opt.unwrap();
    let name = unwrap_or_empty_string!(get_name_from_class_name_node(&name_node, declaration_data));
    let line_number = get_line_number(&name_node);
    let mut class_object = ClassObject::new_class(name.clone(), line_number);
    let mut cpp_class_object: Box<dyn CppObject>;

    if let Some(field_declaration_list) = node.child_by_field_name(NodeNames::BODY) {
        class_object.set_object_type(Definition);
        cpp_class_object = Box::new(class_object);
        add_field_declaration_list(field_declaration_list, declaration_data, &mut cpp_class_object);
    } else {
        class_object.set_object_type(Declaration);
        cpp_class_object = Box::new(class_object);
    }

    /* Zero args constructor declaration */
    let mut constructor_object = MethodObject::default();
    constructor_object.set_line_code(line_number );
    constructor_object.set_type_code(CodeType::CPP_CONSTRUCTOR);
    constructor_object.set_name(name);
    let cpp_constructor_object:Box<dyn CppObject> = Box::new(constructor_object);
    cpp_class_object.add_child(cpp_constructor_object);

    parent.add_child(cpp_class_object);
}

fn add_enum_specifier(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn CppObject>) {

    let name_node = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let name = unwrap_or_empty_string!(get_name_from_class_name_node(&name_node, declaration_data));
    let class_object = ClassObject::new_enum(name, get_line_number(&node));
    let cpp_object: Box<dyn CppObject> = Box::new(class_object);
    parent.add_child(cpp_object);
}

fn add_field_declaration_list(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn CppObject>) {

    for child in node.named_children(&mut node.walk()) {
        match child.kind() {
            NodeKinds::FIELD_DECLARATION | NodeKinds::DECLARATION
            => add_declaration(child, declaration_data, parent),
            NodeKinds::FUNCTION_DEFINITION => add_function_definition(child, declaration_data, parent),
            NodeKinds::FRIEND_DECLARATION => add_friend_declaration(child, declaration_data, parent),
            NodeKinds::PREPROC_IFDEF => add_preproc_ifdef(child, declaration_data, parent),
            NodeKinds::PREPROC_IF => add_preproc_if(child, declaration_data, parent),
            NodeKinds::PREPROC_FUNCTION_DEF => add_preproc_func_def(child, declaration_data, parent),
            NodeKinds::TEMPLATE_DECLARATION => add_template_declaration(child, declaration_data, parent),
            &_ => {}
        }
    }
}

fn add_friend_declaration(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn CppObject>) {

    for child in node.named_children(&mut node.walk()) {
        match child.kind() {
            NodeKinds::FUNCTION_DEFINITION => add_function_definition(child, declaration_data, parent),
            NodeKinds::DECLARATION => add_declaration(child, declaration_data, parent),
            &_ => {}
        }
    }
}

fn add_declaration_list(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn CppObject>) {
    for child in node.named_children(&mut node.walk()) {
        add_top_level_item(child, declaration_data, parent);
    }
}

/* Functions */
fn add_function(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn CppObject>, method_type: ObjectType) {

    let decl = unwrap_or_return!(node.child_by_field_name(NodeNames::DECLARATOR));
    let function_decl = unwrap_or_return!(get_function_declarator(decl));
    let identifier_decl = unwrap_or_return!(function_decl.child_by_field_name(NodeNames::DECLARATOR));
    let name = get_name_from_declarator(identifier_decl, declaration_data);
    let namespace = get_namespace_from_declarator(identifier_decl, declaration_data);
    if name == KeyWords::EMPTY_STRING { return; }

    let (output_parameter, type_code) = get_type_and_param_from_node(&node, declaration_data);
    let return_type = get_return_type_from_declarator(&decl);
    let params_node = unwrap_or_return!(function_decl.child_by_field_name(NodeNames::PARAMETERS));
    let parameters = get_parameters_from_list_node(params_node, declaration_data);
    let line_code = get_line_number(&node);

    let method_object = MethodObject::new(
        name,
        type_code,
        output_parameter,
        parameters,
        namespace,
        return_type,
        line_code,
        method_type,
    );

    let cpp_object: Box<dyn CppObject> = Box::new(method_object);
    parent.add_child(cpp_object);
}

fn add_function_definition(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn CppObject>) {
    add_function(node, declaration_data, parent, Definition);
}

fn add_declaration(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn CppObject>) {
    add_function(node, declaration_data, parent, Declaration);
}

/* Templates */
fn add_template_declaration(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn CppObject>) {

    for child in node.named_children(&mut node.walk()) {
        match child.kind() {
            /* From _empty_declaration node */
            NodeKinds::ENUM_SPECIFIER => add_enum_specifier(child, declaration_data, parent),
            NodeKinds::CLASS_SPECIFIER | NodeKinds::STRUCT_SPECIFIER | NodeKinds::UNION_SPECIFIER
            => add_struct_spec(child, declaration_data, parent),

            /* From other nodes */
            NodeKinds::TEMPLATE_DECLARATION => add_template_declaration(child, declaration_data, parent),
            NodeKinds::FUNCTION_DEFINITION => add_function_definition(child, declaration_data, parent),
            NodeKinds::DECLARATION => add_declaration(child, declaration_data, parent),
            &_ => {}
        }
    }
}

/* Linkage (ffi functions) */
fn add_linkage_specification(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn CppObject>) {

    let body = unwrap_or_return!(node.child_by_field_name(NodeNames::BODY));
    match body.kind() {
        NodeKinds::FUNCTION_DEFINITION => add_function_definition(body, declaration_data, parent),
        NodeKinds::DECLARATION => add_declaration(body, declaration_data, parent),
        NodeKinds::DECLARATION_LIST => add_declaration_list(body, declaration_data, parent),
        &_ => {}
    }
}

/* C/C++ Preprocessor nodes */
fn add_preproc_func_def(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn CppObject>) {

    let name_node = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let preproc_parameters = unwrap_or_return!(node.child_by_field_name(NodeNames::PARAMETERS));

    let method_object = MethodObject::new(
        unwrap_or_empty_string!(get_node_value(&name_node, declaration_data)),
        CPP_METHOD,
        KeyWords::EMPTY_STRING.to_string(),
        get_parameters_from_preproc_node(&preproc_parameters, declaration_data),
        KeyWords::EMPTY_STRING.to_string(),
        Value,
        get_line_number(&node) ,
        Definition,
    );

    let cpp_object: Box<dyn CppObject> = Box::new(method_object);
    parent.add_child(cpp_object);
}

fn add_preproc_if(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn CppObject>) {
    for child in node.named_children(&mut node.walk()) {
        add_top_level_item(child, declaration_data, parent);
    }
}

fn add_preproc_ifdef(node: Node, declaration_data: &mut DeclarationData, parent: &mut Box<dyn CppObject>) {
    for child in node.named_children(&mut node.walk()) {
        add_top_level_item(child, declaration_data, parent);
    }
}

/* Data conversation */
fn prepare_output_data(dto_vec: &mut Vec<RepositoryMethodDto>, cpp_object: Box<dyn CppObject>,
                       rep_id: i32, path: &String, package_name: &String, class_name: &String) {

    match cpp_object.type_code() {
        CPP_PACKAGE => prepare_package_object(dto_vec, cpp_object, rep_id, path, class_name),
        CPP_CLASS | CPP_ENUM => prepare_class_object(dto_vec, cpp_object, rep_id, path, package_name),
        CPP_METHOD | CPP_CONSTRUCTOR
        => prepare_method_object(dto_vec, cpp_object, rep_id, path, package_name, class_name),
        &_ => {}
    }
}

fn prepare_package_object(dto_vec: &mut Vec<RepositoryMethodDto>, cpp_object: Box<dyn CppObject>,
                          rep_id: i32, path: &String, class_name: &String) {

    let boxed_any = cpp_object.to_any();
    let boxed_package = boxed_any.downcast::<PackageObject>().unwrap();
    let package_object = *boxed_package;

    let (
        name,
        line_code,
        type_code,
        children,
        modifiers
    ) = package_object.take();


    let method_dto = RepositoryMethodDtoBuilder::default()
        .repository_id(rep_id)
        .path_file(path.clone())
        .package_name(name.clone())
        .line_code(line_code)
        .modifiers(serde_json::to_string(&modifiers).unwrap_or(KeyWords::EMPTY_STRING.to_string()))
        .method_type(type_code.to_string())
        .build();


    dto_vec.push(method_dto);

    for child in children {
        prepare_output_data(dto_vec, child, rep_id, path, &name, &class_name);
    }
}

fn prepare_class_object(dto_vec: &mut Vec<RepositoryMethodDto>, cpp_object: Box<dyn CppObject>,
                        rep_id: i32, path: &String, package_name: &String) {

    let boxed_any = cpp_object.to_any();
    let boxed_class = boxed_any.downcast::<ClassObject>().unwrap();
    let class_object = *boxed_class;

    let (
        name,
        line_code,
        type_code,
        children,
        modifiers,
        _class_type
    ) = class_object.take();

    let method_dto = RepositoryMethodDtoBuilder::default()
        .repository_id(rep_id)
        .path_file(path.clone())
        .package_name(package_name.clone())
        .line_code(line_code)
        .class_name(name.clone())
        .modifiers(convert_modifiers_vec_to_string(&modifiers))
        .method_type(type_code.to_string())
        .build();

    dto_vec.push(method_dto);

    for child in children {
        prepare_output_data(dto_vec, child, rep_id, path, package_name, &name);
    }
}

fn prepare_method_object(dto_vec: &mut Vec<RepositoryMethodDto>, cpp_object: Box<dyn CppObject>,
                         rep_id: i32, path: &String, package_name: &String, class_name: &String) {

    let boxed_any = cpp_object.to_any();
    let boxed_method = boxed_any.downcast::<MethodObject>().unwrap();
    let method_object = *boxed_method;

    let (
        name,
        line_code,
        type_code,
        children,
        modifiers,
        parameters,
        namespace,
        _output_parameter,
        _return_type,
        _method_type
    ) = method_object.take();

    let method_dto = RepositoryMethodDtoBuilder::default()
        .repository_id(rep_id)
        .path_file(path.clone())
        .package_name(package_name.clone())
        .line_code(line_code)
        .class_name(if class_name == KeyWords::EMPTY_STRING { namespace } else { class_name.clone()})
        .method_name(name)
        .modifiers(convert_modifiers_vec_to_string(&modifiers))
        .method_type(type_code.to_string())
        .count_of_parameters(parameters.len())
        .build();

    dto_vec.push(method_dto);

    for child in children {
        prepare_output_data(dto_vec, child, rep_id, path, package_name, class_name);
    }
}


/* Helpers */
fn convert_modifiers_vec_to_string(modifiers: &Vec<String>) -> String {
    serde_json::to_string(modifiers).unwrap_or(KeyWords::EMPTY_STRING.to_string())
}

fn get_parameters_from_preproc_node(node: &Node, declaration_data: &mut DeclarationData) -> Vec<String> {

    let mut parameters = vec![];
    for child in node.children(&mut node.walk()) {
        if child.kind() == NodeKinds::IDENTIFIER {
            parameters.push(unwrap_or_empty_string!(get_node_value(&child, declaration_data)))
        } //  else if child.kind() == "..." { /* Rest pattern */ }
    }
    parameters
}

fn get_parameters_from_list_node(node: Node, declaration_data: &mut DeclarationData) -> Vec<String> {

    let mut parameters = vec![];
    for child in node.named_children(&mut node.walk()) {

        let node_kind = child.kind();
        if node_kind != NodeKinds::PARAMETER_DECLARATION
            && node_kind != NodeKinds::OPTIONAL_PARAMETER_DECLARATION {
            continue;
        } /* else if node_kind == "variadic_parameter_declaration" { Rest pattern } */

        let declarator_opt = child.child_by_field_name(NodeNames::DECLARATOR);
        if declarator_opt.is_none() { continue; }
        let identifier_opt = get_identifier_from_declarator(declarator_opt.unwrap());
        if identifier_opt.is_none() { continue; }

        let identifier = identifier_opt.unwrap();
        parameters.push(unwrap_or_empty_string!(get_node_value(&identifier, declaration_data)));
    }

    return parameters;
}

fn get_identifier_from_declarator(node: Node) -> Option<Node> {

    if node.kind() == NodeKinds::IDENTIFIER {
        return Some(node);
    }
    for child in node.named_children(&mut node.walk()) {
        return get_identifier_from_declarator(child);
    }
    return None;
}

fn get_name_from_declarator(node: Node, declaration_data: &mut DeclarationData) -> String {

    match node.kind() {
        NodeKinds::FIELD_IDENTIFIER | NodeKinds::IDENTIFIER | NodeKinds::DESTRUCTOR_NAME => {
            unwrap_or_empty_string!(get_node_value(&node, declaration_data))
        }

        NodeKinds::SCOPED_IDENTIFIER => {
            if let Some(name_node) = node.child_by_field_name(NodeNames::NAME) {
                get_name_from_declarator(name_node, declaration_data)
            } else {
                KeyWords::EMPTY_STRING.to_string()
            }
        }

        _ => KeyWords::EMPTY_STRING.to_string()
    }
}

fn get_namespace_from_declarator(node: Node, declaration_data: &mut DeclarationData) -> String {

    let mut namespace_string = KeyWords::EMPTY_STRING.to_string();
    let namespace_opt = node.child_by_field_name(NodeNames::NAMESPACE);
    if namespace_opt.is_none() { return namespace_string; }
    let namespace_node = namespace_opt.unwrap();

    match namespace_node.kind() {

        NodeKinds::NAMESPACE_IDENTIFIER => {
            namespace_string = unwrap_or_empty_string!(get_node_value(&namespace_node, declaration_data))
        }

        NodeKinds::TEMPLATE_TYPE => {
            if let Some(name_node) = namespace_node.child_by_field_name(NodeNames::NAME) {
                match name_node.kind() {

                    NodeKinds::TYPE_IDENTIFIER => {
                        namespace_string = unwrap_or_empty_string!(
                            get_node_value(&name_node, declaration_data)
                        )
                    }

                    NodeKinds::SCOPED_TYPE_IDENTIFIER => {
                        namespace_string = unwrap_or_empty_string!(
                            get_name_from_scoped_type_identifier(&name_node, declaration_data)
                        )
                    }
                    _ => {}
                }
            }
        }
        NodeKinds::SCOPED_NAMESPACE_IDENTIFIER => {
            if let Some(name_node) = namespace_node.child_by_field_name(NodeNames::NAME) {
                namespace_string = unwrap_or_empty_string!(get_node_value(&name_node, declaration_data))
            }
        }
        _ => {}
    }
    return namespace_string;
}

fn get_output_param_from_type_specifier(node: Node, declaration_data: &mut DeclarationData) -> String {

    let mut parameter = String::new();
    match node.kind() {

        NodeKinds::TYPE_IDENTIFIER | NodeKinds::PRIMITIVE_TYPE | NodeKinds::SIZED_TYPE_SPECIFIER
        => parameter = unwrap_or_empty_string!(get_node_value(&node, declaration_data)),

        NodeKinds::SCOPED_TYPE_IDENTIFIER
        => parameter = unwrap_or_empty_string!(get_name_from_scoped_type_identifier(&node, declaration_data)),

        NodeKinds::TEMPLATE_TYPE =>
            {
                if let Some(name_node) = node.child_by_field_name(NodeNames::NAME) {
                    if name_node.kind() == NodeKinds::SCOPED_TYPE_IDENTIFIER {
                        parameter = unwrap_or_empty_string!(
                            get_name_from_scoped_type_identifier(&name_node, declaration_data)
                        )
                    } else {
                        parameter = unwrap_or_empty_string!(
                            get_node_value(&name_node, declaration_data)
                        )
                    }
                }
            }

        _ => {}
    }

    return if parameter == KeyWords::VOID { KeyWords::EMPTY_STRING.to_string() } else { parameter };
}

fn get_function_declarator(node: Node) -> Option<Node> {

    if node.kind() == NodeKinds::FUNCTION_DECLARATOR {
        return Some(node);
    }
    for child in node.named_children(&mut node.walk()) {
        return get_function_declarator(child);
    }
    return None;
}

fn get_name_from_class_name_node(node: &Node, declaration_data: &mut DeclarationData) -> Option<String> {

    return match node.kind() {
        NodeKinds::TYPE_IDENTIFIER => get_node_value(&node, declaration_data),
        NodeKinds::SCOPED_TYPE_IDENTIFIER => get_name_from_scoped_type_identifier(&node, declaration_data),
        NodeKinds::TEMPLATE_TYPE => {
            let name_node = node.child_by_field_name(NodeNames::NAME)?;
            match name_node.kind() {
                NodeKinds::TYPE_IDENTIFIER => get_node_value(&name_node, declaration_data),
                NodeKinds::SCOPED_TYPE_IDENTIFIER => get_name_from_scoped_type_identifier(
                    &name_node, declaration_data
                ),
                &_ => None
            }
        }
        &_ => None
    };
}

fn get_name_from_scoped_type_identifier(node: &Node, declaration_data: &mut DeclarationData) -> Option<String> {
    let name_node = node.child_by_field_name(NodeNames::NAME)?;
    get_node_value(&name_node, declaration_data)
}

fn get_return_type_from_declarator(node: &Node) -> MethodReturnType {
    return match node.kind() {
        NodeKinds::REFERENCE_DECLARATOR => Reference,
        NodeKinds::POINTER_DECLARATOR => Pointer,
        &_ => Value,
    };
}

fn get_type_and_param_from_node(node: &Node, declaration_data: &mut DeclarationData) -> (String, CodeType) {
    /* If class node contains child with field name "type" it's a method */
    return if let Some(type_node) = node.child_by_field_name(NodeNames::TYPE) 
    {
        let output_param = get_output_param_from_type_specifier(type_node, declaration_data);
        (output_param, CPP_METHOD)
    } 
    else 
    {
        (KeyWords::EMPTY_STRING.to_string(), CPP_CONSTRUCTOR)
    };
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


#[cfg(test)]
mod cpp_code_declaration_tests {

    use super::*;
    use tree_sitter::Parser;
    use std::fs;
    use crate::dto::invocation_structure::InvocationStructure;

    #[test]
    pub fn test_get_repository_method_dto() {
        let mut code = fs::read_to_string("resources/test_files/cpp/5.cc.txt").unwrap();
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_cpp::language()).expect("ERROR: Unable to load CPP grammar");
        let tree = parser.parse(&mut code, None).unwrap();
        println!("{},  ", tree.root_node().to_sexp());
        let repository_method_dto = get_repository_method_dto(code,
                                                              tree, "test".to_string(),
                                                              0);
        for dto in repository_method_dto {
            let json = serde_json::to_string_pretty(&dto).unwrap();
            print!("{}", json);
        }

    }
}