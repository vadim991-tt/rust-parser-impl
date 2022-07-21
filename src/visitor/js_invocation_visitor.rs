use crate::dto::object_description::{MethodDescription, Description};
use crate::dto::invocation_structure::{RepositoryImportDeclaration, InvocationStructure};
use tree_sitter::{Node, Tree, TreeCursor};
use std::io::Read;
use crate::unwrap_or_return;
use crate::unwrap_or_empty_string;
use crate::model::js_object::{CodeType};

const MAX_TOKEN_LENGTH: usize = 250;

struct ClassData {
    source_code: String,
    current_package: String,
    current_parent_class: String,
}

impl ClassData {

    pub fn new(source_code: String, current_package: String) -> ClassData {
        ClassData {
            source_code,
            current_package,
            current_parent_class: String::new(),
        }
    }

    fn source_code(&self) -> &String {
        &self.source_code
    }

    fn get_parent_class(&self) -> String {
        self.current_parent_class.clone()
    }

    fn set_parent_class(&mut self, current_class: String) {
        self.current_parent_class = current_class
    }

    fn get_current_package(&self) -> String {
        self.current_package.clone()
    }

    fn current_package(&self) -> &String {
        &self.current_package
    }

}

struct NodeKinds;
impl NodeKinds {
    const CLASS_DECLARATION: &'static str = "class_declaration";
    const CLASS_HERITAGE: &'static str = "class_heritage";
    const CALL_EXPRESSION: &'static str = "call_expression";
    const MEMBER_EXPRESSION: &'static str = "member_expression";
    const IDENTIFIER: &'static str = "identifier";
}

struct NodeNames;
impl NodeNames {
    const FUNCTION: &'static str = "function";
    const PROPERTY: &'static str = "property";
    const OBJECT: &'static str = "object";
    const SUPER: &'static str = "super";
    const ARGUMENTS: &'static str = "arguments";
}

struct KeyWords;
impl KeyWords {
    const JQUERY_SIGN: &'static str = "$";
    const REQUIRE: &'static str = "require";
    const DEFINE: &'static str = "define";
    const EMPTY_STRING: &'static str = "";
}

/* Main function */
pub fn get_file_structure(source_code: String, tree: Tree, path: String) -> InvocationStructure {

    let mut class_data = ClassData::new(source_code, path);
    let mut navigation_links: Vec<MethodDescription> = vec![];
    let import_list: Vec<RepositoryImportDeclaration> = vec![];

    traverse_tree(tree.walk(), &mut navigation_links, &mut class_data);

    InvocationStructure::new(import_list, navigation_links, CodeType::type_codes())
}

fn traverse_tree(mut tree_cursor: TreeCursor, navigation_links: &mut Vec<MethodDescription>, class_data: &mut ClassData) {

    let mut reached_root = false;

    while reached_root == false {

        walk_into(tree_cursor.node(), navigation_links, class_data);

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

fn walk_into(node: Node, navigation_links: &mut Vec<MethodDescription>, class_data: &mut ClassData) {
    match node.kind() {
        NodeKinds::CLASS_DECLARATION => add_class_declaration(node, class_data),
        NodeKinds::CALL_EXPRESSION => add_call_expression(node, navigation_links, class_data),
        _ => {}
    }
}

fn add_class_declaration(node: Node, class_data: &mut ClassData) {
    /* Parent class name */
    let parent_node = unwrap_or_return!(get_child_node_by_kind(&node, NodeKinds::CLASS_HERITAGE));
    let parent_identifier_node = unwrap_or_return!(get_child_node_by_kind(&parent_node, NodeKinds::IDENTIFIER));
    let parent_class_name = unwrap_or_return!(get_node_value(&parent_identifier_node, class_data));
    class_data.set_parent_class(parent_class_name);
}

fn add_call_expression(call_node: Node, links: &mut Vec<MethodDescription>, class_data: &ClassData) {

    let function_node = unwrap_or_return!(call_node.child_by_field_name(NodeNames::FUNCTION));

    /* Method description from identifier node */
    if let Some(name_node) = get_child_node_by_kind(&call_node, NodeKinds::IDENTIFIER) {
        let name = unwrap_or_return!(get_node_value(&name_node, class_data));
        let number = get_line_number(&name_node);
        let position = get_position_in_line(&name_node);
        let count_params = match call_node.child_by_field_name(NodeNames::ARGUMENTS) {
            Some(value) => value.named_child_count(),
            None => 0
        };
        add_new_method_description(name, number, position, count_params, links, class_data);
        return;
    }


    /* Method description from member expression nodes */
    if let Some(member_node) = function_node.child_by_field_name(NodeNames::OBJECT) {
        if member_node.kind() == NodeKinds::MEMBER_EXPRESSION || member_node.kind() == NodeKinds::IDENTIFIER {
            add_members_from_function_call(member_node, links, class_data);
        }
    }

    /* Method description from property node */
    if let Some(name_node) = function_node.child_by_field_name(NodeNames::PROPERTY) {
        let name = unwrap_or_return!(get_node_value(&name_node, class_data));
        let number = get_line_number(&name_node);
        let position = get_position_in_line(&name_node);
        let count_params = match call_node.child_by_field_name(NodeNames::ARGUMENTS) {
            Some(value) => value.named_child_count() ,
            None => 0
        };
        add_new_method_description(name, number, position, count_params, links, class_data);
        return;
    }


    /* Method description from super node */
    if let Some(name_node) = function_node.child_by_field_name(NodeNames::SUPER) {
        let name = class_data.get_parent_class();
        let line = get_line_number(&name_node);
        let position = get_position_in_line(&name_node);
        let count_params = match call_node.child_by_field_name(NodeNames::ARGUMENTS) {
            Some(value) => value.named_child_count() ,
            None => 0
        };
        add_new_method_description(name, line, position, count_params, links, class_data);
    }
}

fn add_members_from_function_call(member: Node, links: &mut Vec<MethodDescription>, class_data: &ClassData) {

    /*  This function is responsible for creating links to function call members.
        For example, if in source code exists function call like:
            `someProperty.someVariableStoredFunction.call()`
        this function will add someProperty and SomeVariableStoredFunctions as method descriptions. */

    if member.kind() == NodeKinds::IDENTIFIER {
        let name = unwrap_or_empty_string!(get_node_value(&member, class_data));
        let line = get_line_number(&member);
        let position = get_position_in_line(&member);
        let count_params = 0;
        add_new_method_description(name, line, position, count_params, links, class_data);
    };

    if let Some(property_node) = member.child_by_field_name(NodeNames::PROPERTY) {
        let name = unwrap_or_empty_string!(get_node_value(&property_node, class_data));
        let line = get_line_number(&property_node);
        let position = get_position_in_line(&property_node);
        let count_params = 0;
        add_new_method_description(name, line, position, count_params, links, class_data);
    }


    if let Some(member_node) = member.child_by_field_name(NodeNames::OBJECT) {
        let member_kind = member_node.kind();
        if member_kind == NodeKinds::MEMBER_EXPRESSION || member_kind == NodeKinds::IDENTIFIER {
            add_members_from_function_call(member_node, links, class_data);
        }
    }
}

fn add_new_method_description(method_name: String, line: usize, position: usize, count_params: usize,
                              links: &mut Vec<MethodDescription>, class_data: &ClassData) {

    if !is_name_valid(&method_name) {
        return;
    }

    let mut method_description = MethodDescription::default();
    method_description.set_line(line);
    method_description.set_position(position);
    method_description.set_method_name(method_name);
    method_description.set_count_param_input(count_params);
    method_description.set_package_name(class_data.get_current_package());
    links.push(method_description);
}

fn get_node_value(node: &Node, class_data: &ClassData) -> Option<String> {

    let source_code = class_data.source_code();
    let bytes = source_code.as_bytes();
    let mut node_bytes = &bytes[node.start_byte()..node.end_byte()];
    let mut node_string = String::new();

    node_bytes
        .read_to_string(&mut node_string)
        .expect(&format!(
            "RUST UNRECOVERABLE ERROR: Unable to read source code. Path file: {}",
            class_data.current_package())
        );

    if node_string.len() < MAX_TOKEN_LENGTH {
        Some(node_string)
    } else {
        None
    }
}

/* Helpers */
fn is_name_valid(function_name: &str) -> bool {
    return match function_name {
        KeyWords::JQUERY_SIGN => false,
        KeyWords::REQUIRE => false,
        KeyWords::DEFINE => false,
        KeyWords::EMPTY_STRING => false,
        &_ => true
    };
}

fn get_child_node_by_kind<'time_spec>(node: &'time_spec Node, kind: &'time_spec str) -> Option<Node<'time_spec>> {
    node
        .children(&mut node.walk())
        .filter(|x| { x.kind() == kind })
        .next()
}

fn get_line_number(node: &Node) -> usize {
    node.start_position().row + 1
}

fn get_position_in_line(node: &Node) -> usize {
    node.start_position().column
}

#[cfg(test)]
mod js_method_invocation_tests {
    use super::*;
    use std::fs;
    use tree_sitter::Parser;

    #[test]
    pub fn test_get_file_structure() {
        let mut code = fs::read_to_string("../../resources/test_files/js/1.js.txt").unwrap();
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_javascript::language()).expect("ERROR: Unable to load JavaScript grammar");
        let tree = parser.parse(&mut code, None).unwrap();
        let invoc_structure = get_file_structure(code, tree, "test".to_string());
        println!("{}", serde_json::to_string_pretty(&invoc_structure).unwrap_or("".to_string()));

    }
}
