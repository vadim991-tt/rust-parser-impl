use std::io::Read;
use tree_sitter::{Node, Tree, TreeCursor};
use crate::dto::invocation_structure::{InvocationStructure, RepositoryImportDeclaration};
use crate::dto::object_description::{Description, MethodDescription, PackageDescription, VarDescription};
use crate::model::python_object::CodeType;
use crate::unwrap_or_empty_string;
use crate::unwrap_or_return;

const MAX_TOKEN_LENGTH: usize = 250;

struct NodeKinds;
impl NodeKinds {
    const CLASS_DEFINITION:&'static str = "class_definition";
    const IDENTIFIER:&'static str = "identifier";
    const TYPED_PARAMETER:&'static str = "typed_parameter";
    const TYPED_DEFAULT_PARAMETER:&'static str = "typed_default_parameter";
    const CALL:&'static str = "call";
    const PARAMETERS:&'static str = "parameters";
    const IMPORT_STATEMENT:&'static str = "import_statement";
    const IMPORT_FROM_STATEMENT:&'static str = "import_from_statement";
    const DOTTED_NAME:&'static str = "dotted_name";
    const ALIASED_IMPORT: &'static str = "aliased_import";
    const ATTRIBUTE:&'static str = "attribute";
}

struct NodeNames;
impl NodeNames {
    const ALIAS:&'static str = "alias";
    const NAME:&'static str = "name";
    const TYPE:&'static str = "type";
    const FUNCTION:&'static str = "function";
    const ARGUMENTS:&'static str = "arguments";
}

struct KeyWords;
impl KeyWords {
    const SELF_SPECIFIER:&'static str = "self";
    const EMPTY_STRING:&'static str = "";
}

struct InvocationData {
    import_declarations: Vec<RepositoryImportDeclaration>,
    package_descriptions: Vec<PackageDescription>,
    var_descriptions: Vec<VarDescription>,
    links: Vec<MethodDescription>,
    current_package: String,
    source_code: String,
    path: String,
}

impl InvocationData {
    fn new(source_code: String, path: String) -> Self {
        Self {
            import_declarations: vec![],
            current_package: String::new(),
            package_descriptions: vec![],
            var_descriptions: vec![],
            links: vec![],
            source_code,
            path,
        }
    }

    fn source_code(&self) -> &String {
        &self.source_code
    }

    fn path(&self) -> &String {
        &self.path
    }

    fn get_current_package(&self) -> String {
        self.current_package.clone()
    }

    fn mut_import_vec(&mut self) -> &mut Vec<RepositoryImportDeclaration> {
        &mut self.import_declarations
    }

    fn package_descriptions(&self) -> &Vec<PackageDescription> {
        &self.package_descriptions
    }

    fn var_descriptions(&self) -> &Vec<VarDescription> {
        &self.var_descriptions
    }

    fn mut_package_descriptions(&mut self) -> &mut Vec<PackageDescription> {
        &mut self.package_descriptions
    }

    fn mut_var_descriptions(&mut self) -> &mut Vec<VarDescription> {
        &mut self.var_descriptions
    }

    fn import_index_of(&self, import_decl: &RepositoryImportDeclaration) -> Option<usize> {
        self.import_declarations.iter().position(|x| x == import_decl)
    }

    fn take(self) -> (Vec<RepositoryImportDeclaration>, Vec<MethodDescription>) {
        (self.import_declarations, self.links)
    }

    fn mut_navigation_links(&mut self) -> &mut Vec<MethodDescription> {
        &mut self.links
    }


}

pub fn get_file_structure(source_code: String, tree: Tree, path: String) -> InvocationStructure {

    let mut invocation_data = InvocationData::new(source_code, path.clone());

    traverse_tree(tree.walk(), & mut invocation_data);

    let (
        import_declarations,
        method_descriptions
    ) = invocation_data.take();

    return InvocationStructure::new(
        import_declarations,
        method_descriptions,
        CodeType::type_codes(),
    );
}

fn traverse_tree(mut tree_cursor: TreeCursor, invocation_data: &mut InvocationData) {

    let mut reached_root = false;

    while reached_root == false {

        walk_into(tree_cursor.node(), invocation_data);

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

fn walk_into(node: Node, invocation_data: &mut InvocationData){
    match node.kind() {
        NodeKinds::IMPORT_STATEMENT => add_import_statement(node, invocation_data),
        NodeKinds::IMPORT_FROM_STATEMENT => add_import_from_statement(node, invocation_data),
        NodeKinds::CLASS_DEFINITION => add_class_definition(node, invocation_data),
        NodeKinds::PARAMETERS => add_parameters(node, invocation_data),
        NodeKinds::CALL => add_function_call(node, invocation_data),
        &_ => {}
    }
}

fn add_import_statement(node: Node, invocation_data: &mut InvocationData) {
    for child in node.named_children(&mut  node.walk()) {
        add_import_from_import_statement(child, invocation_data);
    }
}

fn add_import_from_statement(node: Node, invocation_data: &mut InvocationData) {

    let mut package_node_opt:Option<Node> = None;
    let mut class_nodes = vec![];

    let mut first_iteration = true;
    for child in node.named_children(& mut node.walk()) {
        if first_iteration {
            package_node_opt = Some(child);
            first_iteration = false;
            continue;
        }
        class_nodes.push(child);
    }

    let module_node = unwrap_or_return!(package_node_opt);
    let package_name = unwrap_or_empty_string!(get_node_value(&module_node, invocation_data));

    let mut class_names = vec![];
    for class_node in class_nodes {

        let class_name = get_class_name_from_import_node(&class_node, invocation_data);
        let position = get_position_in_line(&class_node);
        let line = get_line_number(&class_node);
        class_names.push(class_name.clone());
        create_package_description(package_name.clone(), class_name, line, position, invocation_data);
    }

    let import_declaration = get_or_create_import_decl(package_name.clone(), invocation_data);
    for class_name in class_names {
        import_declaration.add_class(class_name);
    }

}

fn add_import_from_import_statement(node: Node, invocation_data: & mut InvocationData) {

    let mut class_name= KeyWords::EMPTY_STRING.to_string();
    let mut package_name = KeyWords::EMPTY_STRING.to_string();
    let line = get_line_number(&node);
    let position = get_position_in_line(&node);
    let import_kind = node.kind();

    if import_kind == NodeKinds::DOTTED_NAME {

        class_name = unwrap_or_empty_string!(get_node_value(&node, invocation_data));
        package_name = class_name.clone();

    } else if import_kind == NodeKinds::ALIASED_IMPORT {

        let alias_node = unwrap_or_return!(node.child_by_field_name(NodeNames::ALIAS));
        class_name = unwrap_or_empty_string!(get_node_value(&alias_node, invocation_data));

        let name_node = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
        package_name = unwrap_or_empty_string!(get_node_value(&name_node, invocation_data));
    }

    let import_declaration = get_or_create_import_decl(package_name.clone(),  invocation_data);
    import_declaration.add_class(class_name.clone());

    create_package_description(package_name, class_name, line, position, invocation_data);
}

fn add_class_definition(node: Node, invocation_data: &mut InvocationData) {

    let name_node = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let class_name = unwrap_or_empty_string!(get_node_value(&name_node, invocation_data));
    let line = get_line_number(&name_node);
    let position = get_position_in_line(&name_node);
    let current_package = invocation_data.get_current_package();

    let mut package_description = PackageDescription::default();
    package_description.set_class_name(class_name.clone());
    package_description.set_package_name(current_package.clone());
    package_description.set_position(position);
    package_description.set_line(line);

    invocation_data.mut_package_descriptions().push(package_description);

    let mut var_description = VarDescription::default();
    var_description.set_package_name(current_package);
    var_description.set_class_name(class_name.clone());
    var_description.set_position(position);
    var_description.set_line(line);
    var_description.set_var_name(class_name); /* Static methods */

}

fn add_parameters(node: Node, invocation_data: &mut InvocationData) {
    for child in node.named_children(& mut node.walk()) {
        add_parameter(child, invocation_data);
    }
}

fn add_parameter(node: Node, invocation_data: &mut InvocationData) {

    let mut type_name = KeyWords::EMPTY_STRING.to_string();
    let mut param_name = KeyWords::EMPTY_STRING.to_string();

    match node.kind() {
        NodeKinds::TYPED_DEFAULT_PARAMETER => {
            param_name = get_name_from_typed_default_param(&node, invocation_data);
            type_name = get_type_from_typed_param(&node, invocation_data);
        }
        NodeKinds::TYPED_PARAMETER => {
            param_name = get_name_from_typed_param(&node, invocation_data);
            type_name = get_type_from_typed_param(&node, invocation_data);
        }
        &_ => {}
    }

    if type_name != KeyWords::EMPTY_STRING && param_name != KeyWords::EMPTY_STRING {
        add_var_description(&node, type_name, param_name, invocation_data);
    }

}

fn add_var_description(node: &Node, class_name: String, param_name: String, invocation_data: &mut InvocationData) {

    let mut var_description = VarDescription::default();
    if let Some(package_desc) = find_package_by_class_name(&class_name, invocation_data) {
        var_description.set_package_name(package_desc.get_package_name());
        var_description.set_class_name(package_desc.get_class_name());
    } else {
        var_description.set_package_name(invocation_data.get_current_package());
        var_description.set_class_name(class_name);
    }
    var_description.set_line(get_line_number(&node));
    var_description.set_position(get_position_in_line(&node));
    var_description.set_var_name(param_name);
    invocation_data.mut_var_descriptions().push(var_description);
}

fn add_function_call(node: Node, invocation_data: &mut InvocationData) {

    let function_node = unwrap_or_return!(node.child_by_field_name(NodeNames::FUNCTION));
    let arguments = unwrap_or_return!(node.child_by_field_name(NodeNames::ARGUMENTS));
    let (method_name, var_name) = function_name_var_name_from_function(&function_node, invocation_data);
    let count_of_params = arguments.named_child_count();

    add_navigation_link(&function_node, &var_name, &method_name, count_of_params, invocation_data);
}

fn add_navigation_link(node: &Node, var_name: &String, method_name: &String,
                       count_of_params: usize, invocation_data: &mut InvocationData) {

    if add_link_from_var(&node, &var_name, &method_name, count_of_params, invocation_data) {
        return;
    }

    if add_link(&node, &var_name, &method_name, count_of_params, invocation_data) {
        return;
    }
}

fn add_link_from_var(node: &Node, var_name: &String, method_name: &String, count_of_params: usize,
                     invocation_data: &mut InvocationData) -> bool {

    if var_name == KeyWords::SELF_SPECIFIER { return false; }
    let var_description_opt = find_var_desc_by_name(&var_name, invocation_data);
    if var_description_opt.is_none() { return false; }
    let var_description = var_description_opt.unwrap();

    let navigation_link = MethodDescription::new(
        var_description.get_package_name(),
        var_description.get_class_name(),
        get_line_number(&node),
        get_position_in_line(&node),
        var_description.get_var_name(),
        method_name.clone(),
        count_of_params,
    );

    invocation_data.mut_navigation_links().push(navigation_link);
    return true;
}

fn add_link(node: &Node, var_name: &String, method_name: &String,
            count_of_params: usize, invocation_data: &mut InvocationData) -> bool {

    let mut navigation_link = MethodDescription::default();
    if let Some(parent_description) = find_package_by_class_name(&var_name, invocation_data) {
        navigation_link.set_package_name(parent_description.get_package_name());
        navigation_link.set_class_name(parent_description.get_class_name());
        navigation_link.set_var_name(var_name.clone());
        navigation_link.set_method_name(method_name.clone());
    } else {
        navigation_link.set_package_name(invocation_data.get_current_package());
        navigation_link.set_class_name(KeyWords::EMPTY_STRING.to_string());
        navigation_link.set_var_name(var_name.clone());
        navigation_link.set_method_name(method_name.clone());
    }

    navigation_link.set_line(get_line_number(&node));
    navigation_link.set_position(get_position_in_line(&node));
    navigation_link.set_count_param_input(count_of_params);
    invocation_data.mut_navigation_links().push(navigation_link);

    return true;
}

/* Helpers */
fn create_package_description(package_name:String, class_name: String, line: usize,
                              position: usize, invocation_data: &mut InvocationData){

    let mut package_description = PackageDescription::default();
    package_description.set_package_name(package_name);
    package_description.set_class_name(class_name);
    package_description.set_line(line);
    package_description.set_position(position);
    invocation_data.mut_package_descriptions().push(package_description);
}

fn get_name_from_typed_default_param(node: &Node, declaration_data: &InvocationData) -> String {

    let node_opt = node.child_by_field_name(NodeNames::NAME);
    if node_opt.is_none() {
        return KeyWords::EMPTY_STRING.to_string();
    }
    return unwrap_or_empty_string!(get_node_value(&node_opt.unwrap(), declaration_data));
}

fn get_name_from_typed_param(node: &Node, declaration_data: &InvocationData) -> String {

    let identifier_opt = get_child_node_by_kind(&node, NodeKinds::IDENTIFIER);
    if identifier_opt.is_none() {
        return KeyWords::EMPTY_STRING.to_string();
    }
    return unwrap_or_empty_string!(get_node_value(&identifier_opt.unwrap(), declaration_data))
}

fn get_type_from_typed_param(node: &Node, declaration_data: &InvocationData) -> String {

    let node_opt = node.child_by_field_name(NodeNames::TYPE);
    if node_opt.is_none() {
        return KeyWords::EMPTY_STRING.to_string();
    }
    return unwrap_or_empty_string!(get_node_value(&node_opt.unwrap(), declaration_data));
}

fn function_name_var_name_from_function(node: &Node, invocation_data: &mut InvocationData) -> (String, String) {

    /* Self specifier is used to emphasize that function call belongs to this module */
    let mut function_name = KeyWords::EMPTY_STRING.to_string();
    let mut var_name = KeyWords::SELF_SPECIFIER.to_string();

    match node.kind() {

        NodeKinds::ATTRIBUTE => {

            if let Some(attribute_node) = node.child_by_field_name("attribute") {
                function_name = unwrap_or_empty_string!(get_node_value(&attribute_node, invocation_data));
            }

            if let Some(object_node) = node.child_by_field_name("object") {
                match object_node.kind() {
                    NodeKinds::IDENTIFIER => {
                        var_name = unwrap_or_empty_string!(get_node_value(&object_node, invocation_data))
                    }
                    &_ => {
                        var_name = KeyWords::EMPTY_STRING.to_string();
                    }
                }
            }
        }

        NodeKinds::IDENTIFIER => {
            function_name = unwrap_or_empty_string!(get_node_value(&node, invocation_data));
        }

        &_ => {}
    }

    return (function_name, var_name)

}

fn find_package_by_class_name<'time>(class_name: &'time String, invocation_data: &'time InvocationData) -> Option<&'time PackageDescription> {

    let package_descriptions = invocation_data.package_descriptions();
    let named_package_opt = package_descriptions
        .iter()
        .find(|&x| x.class_name() == class_name);

    if let Some(named_package) = named_package_opt {
        return Some(named_package);
    }

    /* Wildcard_import */
    let unnamed_package_opt = package_descriptions
        .iter()
        .find(|&x| x.class_name() == KeyWords::EMPTY_STRING);

    if let Some(unnamed_package) = unnamed_package_opt {
        return Some(unnamed_package);
    }

    return None;
}

fn get_node_value(node: &Node, declaration_data: & InvocationData) -> Option<String> {

    let source = declaration_data.source_code();
    let bytes = source.as_bytes();
    let mut node_bytes = &bytes[node.start_byte()..node.end_byte()];
    let mut node_string = String::new();

    node_bytes
        .read_to_string(&mut node_string)
        .expect(&format!(
            "RUST UNRECOVERABLE ERROR: Unable to read source code. Path file: {}",
            declaration_data.path())
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

fn get_position_in_line(node: &Node) -> usize {
    node.start_position().column
}

fn get_or_create_import_decl(package_name: String, invocation_data: & mut InvocationData) -> &mut RepositoryImportDeclaration {

    let import_declaration = RepositoryImportDeclaration::new(package_name);

    return if let Some(index) = invocation_data.import_index_of(&import_declaration) {

        invocation_data
            .mut_import_vec()
            .get_mut(index)
            .unwrap()

    } else {

        invocation_data.mut_import_vec().push(import_declaration);

        invocation_data
            .mut_import_vec()
            .last_mut()
            .unwrap()
    }
}

fn get_class_name_from_import_node(node: &Node, invocation_data: &InvocationData) -> String {

    let mut class_name = KeyWords::EMPTY_STRING.to_string();

    let import_kind = node.kind();
    if import_kind == NodeKinds::DOTTED_NAME {
        class_name = unwrap_or_empty_string!(get_node_value(&node, invocation_data));
    } else if import_kind == NodeKinds::ALIASED_IMPORT {
        if let Some(alias_node) = node.child_by_field_name(NodeNames::ALIAS) {
            class_name = unwrap_or_empty_string!(get_node_value(&alias_node, invocation_data))
        }
    }

    return class_name;
}

fn get_child_node_by_kind<'time>(node: &'time Node, kind: &'time str) -> Option<Node<'time>> {
    node
        .children(&mut node.walk())
        .filter(|x| { x.kind() == kind })
        .next()
}

fn find_var_desc_by_name<'spec>(name: &'spec String, invocation_data: &'spec InvocationData) -> Option<&'spec VarDescription> {

    invocation_data
        .var_descriptions()
        .iter()
        .rev()
        .find(|&x| x.class_name() == name)
}


#[cfg(test)]
mod python_method_invocation_tests {

    use super::*;
    use std::fs;
    use tree_sitter::Parser;

    #[test]
    pub fn test_get_file_structure() {

        let mut code = fs::read_to_string("../../resources/test_files/python/3.py.txt").unwrap();
        let mut parser = Parser::new();

        parser
            .set_language(tree_sitter_python::language())
            .expect("ERROR: Unable to load Python grammar");

        let tree = parser.parse(&mut code, None).unwrap();


        let fs = get_file_structure(code, tree, "test".to_string());
        println!("{}", serde_json::to_string_pretty(&fs).unwrap())

    }
}
