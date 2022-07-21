use tree_sitter::{Tree, Node};
use crate::dto::invocation_structure::{InvocationStructure, RepositoryImportDeclaration};
use crate::dto::object_description::{MethodDescription, PackageDescription, VarDescription, Description};
use std::io::Read;
use crate::model::java_object::{CodeType};
use crate::unwrap_or_return;
use crate::unwrap_or_empty_string;

const MAX_TOKEN_LENGTH: usize = 250;

struct NodeKinds;

impl NodeKinds {
    const LOCAL_VARIABLE_DECLARATION: &'static str = "local_variable_declaration";
    const SCOPED_TYPE_IDENTIFIER: &'static str = "scoped_type_identifier";
    const METHOD_INVOCATION: &'static str = "method_invocation";
    const OBJECT_CREATION_EXPRESSION: &'static str = "object_creation_expression";
    const PACKAGE_DECLARATION: &'static str = "package_declaration";
    const IMPORT_DECLARATION: &'static str = "import_declaration";
    const INTERFACE_DECLARATION: &'static str = "interface_declaration";
    const CLASS_DECLARATION: &'static str = "class_declaration";
    const CONSTRUCTOR_DECLARATION: &'static str = "constructor_declaration";
    const ENUM_DECLARATION: &'static str = "enum_declaration";
    const METHOD_DECLARATION: &'static str = "method_declaration";
    const SCOPED_IDENTIFIER: &'static str = "scoped_identifier";
    const ENUM_BODY_DECLARATIONS: &'static str = "enum_body_declarations";
    const FORMAL_PARAMETER: &'static str = "formal_parameter";
    const VARIABLE_DECLARATOR: &'static str = "variable_declarator";
    const IDENTIFIER: &'static str = "identifier";
    const ASTERISK: &'static str = "asterisk";
    const FIELD_DECLARATION: &'static str = "field_declaration";
    const TYPE_IDENTIFIER: &'static str = "type_identifier";
    const GENERIC_TYPE: &'static str = "generic_type";
    const INTERFACE_TYPE_LIST: &'static str = "interface_type_list";
    const STATIC_INITIALIZER: &'static str = "static_initializer";
    const BLOCK:&'static str = "block";
}

struct KeyWords;

impl KeyWords {
    const ALL_CLASSES: &'static str = "*";
    const THIS: &'static str = "this";
    const SUPER_CLASS: &'static str = "super";
    const EMPTY_STRING: &'static str = "";
}

struct NodeNames;

impl NodeNames {
    const OBJECT: &'static str = "object";
    const NAME: &'static str = "name";
    const SCOPE: &'static str = "scope";
    const BODY: &'static str = "body";
    const TYPE: &'static str = "type";
    const ARGUMENTS: &'static str = "arguments";
    const PARAMETERS: &'static str = "parameters";
    const DECLARATOR: &'static str = "declarator";
    const SUPERCLASS: &'static str = "superclass";
    const INTERFACES: &'static str = "interfaces";
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
            current_package: "".to_string(),
            source_code,
            path,
            import_declarations: vec![],
            package_descriptions: vec![],
            var_descriptions: vec![],
            links: vec![],
        }
    }

    fn set_current_package(&mut self, current_package: String) {
        self.current_package = current_package;
    }

    fn get_current_package(&self) -> String {
        self.current_package.clone()
    }

    fn source_code(&self) -> &String {
        &self.source_code
    }

    fn path(&self) -> &String {
        &self.path
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

    fn mut_navigation_links(&mut self) -> &mut Vec<MethodDescription> {
        &mut self.links
    }

    fn mut_import_list(&mut self) -> &mut Vec<RepositoryImportDeclaration> {
        &mut self.import_declarations
    }

    fn import_index_of(&self, import_decl: &RepositoryImportDeclaration) -> Option<usize> {
        self.import_declarations.iter().position(|x| x == import_decl)
    }

    fn take(self) -> (Vec<RepositoryImportDeclaration>, Vec<MethodDescription>) {
        (self.import_declarations, self.links)
    }
}

/* Main */
pub fn get_file_structure(source_code: String, tree: Tree, path: String) -> InvocationStructure {

    let node = tree.root_node();
    let mut invocation_data = InvocationData::new(source_code, path.clone());
    add_package_declaration(&mut invocation_data, &node);

    parse_root_node(&node, &mut invocation_data);

    let (
        repository_import_declarations,
        method_descriptions
    ) = invocation_data.take();

    return InvocationStructure::new(
        repository_import_declarations,
        method_descriptions,
        CodeType::type_codes(),
    );
}

fn parse_root_node(node: &Node, invocation_data: &mut InvocationData) {

    for child in node.named_children(&mut node.walk()) {
        match child.kind() {
            NodeKinds::IMPORT_DECLARATION => add_import_declaration(&child, invocation_data),
            NodeKinds::ENUM_DECLARATION => add_enum_declaration(&child, invocation_data),
            NodeKinds::CLASS_DECLARATION | NodeKinds::INTERFACE_DECLARATION
            => add_class_or_interface_declaration(&child, invocation_data),
            &_ => {}
        }
    }
}

fn parse_class_body(node: &Node, invocation_data: &mut InvocationData, class_name: String) {

    for declaration in node.named_children(&mut node.walk()) {
        match declaration.kind() {

            NodeKinds::INTERFACE_DECLARATION | NodeKinds::CLASS_DECLARATION
            => add_class_or_interface_declaration(&declaration, invocation_data),

            NodeKinds::ENUM_DECLARATION => add_enum_declaration(&declaration, invocation_data),

            NodeKinds::FIELD_DECLARATION => add_field_declaration(&declaration, invocation_data, class_name.clone()),

            NodeKinds::METHOD_DECLARATION | NodeKinds::CONSTRUCTOR_DECLARATION
            => add_method_or_constr_declaration(&declaration, invocation_data, class_name.clone()),

            NodeKinds::STATIC_INITIALIZER => add_static_initializer(&declaration, invocation_data, class_name.clone()),
            &_ => {}
        }
    }
}

fn parse_node(node: &Node, invocation_data: &mut InvocationData, class_name: String) {

    for statement in node.named_children(&mut node.walk()) {
        match statement.kind() {
            NodeKinds::LOCAL_VARIABLE_DECLARATION => add_variable_declaration(&statement, invocation_data),
            NodeKinds::METHOD_INVOCATION => add_method_invocation(&statement, invocation_data, class_name.clone()),
            NodeKinds::OBJECT_CREATION_EXPRESSION => add_object_creation_expression(&statement, invocation_data),
            _ => {}
        }
        parse_node(&statement, invocation_data, class_name.clone());
    }
}

/* Visitors */
fn add_static_initializer(node: &Node, invocation_data: &mut InvocationData, class_name: String) {
    let block_node = unwrap_or_return!(get_child_node_by_kind(&node, NodeKinds::BLOCK));
    parse_node(&block_node, invocation_data, class_name);
}

fn add_enum_declaration(node: &Node, invocation_data: &mut InvocationData) {

    let name_node = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let class_name = unwrap_or_empty_string!(get_node_value(&name_node, invocation_data));
    let line = get_line_number(&name_node);
    let position = get_position_in_line(&name_node);

    let mut package_description = PackageDescription::new(
        invocation_data.get_current_package(),
        class_name.clone(),
        line,
        position,
        vec![],
    );

    let var_description = VarDescription::new(
        invocation_data.get_current_package(),
        class_name.clone(),
        line,
        position,
        KeyWords::THIS.to_string(),
    );


    if let Some(super_node) = node.child_by_field_name(NodeNames::SUPERCLASS) {
        add_var_description_from_super_class(&super_node, invocation_data, &mut package_description);
    }

    if let Some(interfaces_node) = node.child_by_field_name(NodeNames::INTERFACES) {
        add_var_descriptions_from_interfaces(&interfaces_node, invocation_data, &mut &mut package_description);
    }

    invocation_data.mut_package_descriptions().push(package_description);
    invocation_data.mut_var_descriptions().push(var_description);

    let enum_body = unwrap_or_return!(node.child_by_field_name(NodeNames::BODY));
    let enum_declarations = unwrap_or_return!(get_child_node_by_kind(&enum_body, NodeKinds::ENUM_BODY_DECLARATIONS));
    parse_class_body(&enum_declarations, invocation_data, class_name);
}

fn add_field_declaration(node: &Node, invocation_data: &mut InvocationData, parent_class_name: String) {

    let type_node = unwrap_or_return!(node.child_by_field_name(NodeNames::TYPE));
    let declarator_node = unwrap_or_return!(node.child_by_field_name(NodeNames::DECLARATOR));
    let name_node = unwrap_or_return!(declarator_node.child_by_field_name(NodeNames::NAME));
    let class_name = unwrap_or_empty_string!(get_node_value(&type_node, invocation_data));
    let param_name = unwrap_or_empty_string!(get_node_value(&name_node, invocation_data));

    add_var_description(node, class_name, param_name, invocation_data);

    parse_node(node, invocation_data, parent_class_name);
}

fn add_method_or_constr_declaration(node: &Node, invocation_data: &mut InvocationData, class_name: String) {

    let parameters = unwrap_or_return!(node.child_by_field_name(NodeNames::PARAMETERS));
    add_parameters(&parameters, invocation_data);
    let body = unwrap_or_return!(node.child_by_field_name(NodeNames::BODY));
    parse_node(&body, invocation_data, class_name);
}

fn add_method_invocation(node: &Node, invocation_data: &mut InvocationData, class_name: String) {

    let name_node = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let method_name = unwrap_or_empty_string!(get_node_value(&name_node, invocation_data));
    let var_name = var_name_from_invocation_or_this(node, invocation_data);
    let count_of_params: usize = count_params_from_node(node);

    add_navigation_link(
        node,
        &var_name,
        &method_name,
        &class_name,
        count_of_params,
        invocation_data,
    );
}

fn add_object_creation_expression(node: &Node, invocation_data: &mut InvocationData) {

    let node_name = unwrap_or_return!(node.child_by_field_name(NodeNames::TYPE));
    let method_name = get_name_from_node(&node_name, invocation_data);
    let var_name = var_name_from_obj_parent_or_this(node, invocation_data);
    let count_of_params: usize = count_params_from_node(node);

    add_navigation_link(
        node,
        &var_name,
        &method_name,
        &method_name,
        count_of_params,
        invocation_data,
    );
}

fn add_variable_declaration(node: &Node, invocation_data: &mut InvocationData) {

    let type_node = unwrap_or_return!(node.child_by_field_name(NodeNames::TYPE));
    let declarator_node = unwrap_or_return!(node.child_by_field_name(NodeNames::DECLARATOR));
    let name_node = unwrap_or_return!(declarator_node.child_by_field_name(NodeNames::NAME));
    let class_name = unwrap_or_empty_string!(get_node_value(&type_node, invocation_data));
    let param_name = unwrap_or_empty_string!(get_node_value(&name_node, invocation_data));
    add_var_description(node, class_name, param_name, invocation_data);
}

fn add_parameters(node: &Node, invocation_data: &mut InvocationData) {

    for child in node.named_children(&mut node.walk()) {
        match child.kind() {
            NodeKinds::FORMAL_PARAMETER => add_formal_parameter(&child, invocation_data),
            &_ => {} /* Spread parameter, Receiver parameter*/
        }
    }
}

fn add_formal_parameter(node: &Node, invocation_data: &mut InvocationData) {

    let type_node = unwrap_or_return!(node.child_by_field_name(NodeNames::TYPE));
    let name_node = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let param_type = unwrap_or_empty_string!(get_node_value(&type_node, invocation_data));
    let param_name = unwrap_or_empty_string!(get_node_value(&name_node, invocation_data));
    add_var_description(node, param_type, param_name, invocation_data);
}

fn add_class_or_interface_declaration(node: &Node, invocation_data: &mut InvocationData) {

    let name_node = unwrap_or_return!(node.child_by_field_name(NodeNames::NAME));
    let class_name = unwrap_or_empty_string!(get_node_value(&name_node, invocation_data));
    let line = get_line_number(&name_node);
    let position = get_position_in_line(&name_node);
    let package_name = invocation_data.get_current_package();

    let mut package_description = PackageDescription::new(
        package_name.clone(),
        class_name.clone(),
        line,
        position,
        vec![],
    );


    let var_description = VarDescription::new(
        package_name,
        class_name.clone(),
        line,
        position,
        KeyWords::THIS.to_string(),
    );


    if let Some(super_node) = node.child_by_field_name(NodeNames::SUPERCLASS) {
        add_var_description_from_super_class(&super_node, invocation_data, &mut package_description);
    }

    if let Some(interfaces_node) = node.child_by_field_name(NodeNames::INTERFACES) {
        add_var_descriptions_from_interfaces(&interfaces_node, invocation_data, &mut package_description);
    }

    invocation_data.mut_var_descriptions().push(var_description);
    invocation_data.mut_package_descriptions().push(package_description);

    let class_body = unwrap_or_return!(node.child_by_field_name(NodeNames::BODY));
    parse_class_body(&class_body, invocation_data, class_name);
}

fn add_package_declaration(invocation_data: &mut InvocationData, root_node: &Node) {

    let mut current_package = KeyWords::EMPTY_STRING.to_string();
    if let Some(node) = get_child_node_by_kind(root_node, NodeKinds::PACKAGE_DECLARATION) {
        if let Some(name_node) = get_child_node_by_kind(&node, NodeKinds::SCOPED_IDENTIFIER) {
            if let Some(scoped_identifier_value) = get_node_value(&name_node, invocation_data) {
                current_package = scoped_identifier_value;
            }
        } else if let Some(name_node) = get_child_node_by_kind(&node, NodeKinds::IDENTIFIER) {
            if let Some(identifier_value) = get_node_value(&name_node, invocation_data) {
                current_package = identifier_value;
            }
        }
    }

    invocation_data.mut_import_list().push(RepositoryImportDeclaration::new(current_package.clone()));
    invocation_data.set_current_package(current_package);
}

fn add_import_declaration(node: &Node, invocation_data: &mut InvocationData) {

    let package_name;
    let mut class_name;

    if let Some(_asterisk) = get_child_node_by_kind(&node, NodeKinds::ASTERISK) {
        class_name = KeyWords::ALL_CLASSES.to_string();
    } else {
        class_name = KeyWords::EMPTY_STRING.to_string();
    }

    let scoped_node = unwrap_or_return!(get_child_node_by_kind(&node, NodeKinds::SCOPED_IDENTIFIER));
    if class_name != KeyWords::ALL_CLASSES {

        let class_node = unwrap_or_return!(scoped_node.child_by_field_name(NodeNames::NAME));
        class_name = unwrap_or_empty_string!(get_node_value(& class_node, invocation_data));

        let package_node = unwrap_or_return!(scoped_node.child_by_field_name(NodeNames::SCOPE));
        package_name = unwrap_or_empty_string!(get_node_value(& package_node, invocation_data));

    } else {
        package_name = unwrap_or_return!(get_node_value(&scoped_node, invocation_data));
    }

    let mut package_description = PackageDescription::default();
    if class_name != KeyWords::ALL_CLASSES {
        package_description.set_class_name(class_name.clone());
    }
    package_description.set_package_name(package_name.clone());
    package_description.set_line(get_line_number(&node));
    package_description.set_position(get_position_in_line(&node));
    invocation_data.mut_package_descriptions().push(package_description);

    let mut import_declaration = RepositoryImportDeclaration::new(package_name);

    if let Some(index) = invocation_data.import_index_of(&import_declaration) {

        let import_from_data = invocation_data
            .mut_import_list()
            .get_mut(index)
            .unwrap();

        if class_name != KeyWords::ALL_CLASSES {
            import_from_data.add_class(class_name.clone());
        }

    } else {

        if class_name != KeyWords::ALL_CLASSES {
            import_declaration.add_class(class_name.clone());
        }

        invocation_data.mut_import_list().push(import_declaration);
    }
}

/* Var descriptions */
fn add_var_description_from_super_class(node: &Node, invocation_data: &mut InvocationData,
                                        child: &mut PackageDescription) {
    let class_name = get_name_from_node(node, invocation_data);
    let line = get_line_number(&node);
    let position = get_position_in_line(&node);
    let package_name = invocation_data.get_current_package();

    let package_description = PackageDescription::new(
        package_name.clone(),
        class_name.clone(),
        line,
        position,
        vec![],
    );


    let var_description = VarDescription::new(
        package_name,
        class_name.clone(),
        line,
        position,
        KeyWords::SUPER_CLASS.to_string(),
    );

    invocation_data.mut_package_descriptions().push(package_description);
    invocation_data.mut_var_descriptions().push(var_description);
    child.add_parent(class_name)
}

fn add_var_descriptions_from_interfaces(node: &Node, invocation_data: &mut InvocationData,
                                        child: &mut PackageDescription) {

    let interface_list = unwrap_or_return!(get_child_node_by_kind(&node, NodeKinds::INTERFACE_TYPE_LIST));

    for child_node in interface_list.named_children(&mut interface_list.walk()) {
        add_var_description_from_super_class(&child_node, invocation_data, child);
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

/* Navigation links */
fn add_navigation_link(node: &Node, var_name: &String, method_name: &String, class_name: &String,
                       count_of_params: usize, invocation_data: &mut InvocationData) {

    if add_link_from_var(&node, &var_name, &method_name, count_of_params, invocation_data) {
        return;
    }

    if add_links_from_package(&node, &var_name, &method_name, &class_name, count_of_params, invocation_data) {
        return;
    }

    if add_link(&node, &var_name, &method_name, count_of_params, invocation_data) {
        return;
    }
}

fn add_link_from_var(node: &Node, var_name: &String, method_name: &String, count_of_params: usize,
                     invocation_data: &mut InvocationData) -> bool {

    if var_name == KeyWords::THIS {
        return false;
    }

    let var_description_opt = find_var_desc_by_name(&var_name, invocation_data);
    if var_description_opt.is_none() {
        return false;
    }

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

fn add_links_from_package(node: &Node, var_name: &String, method_name: &String,
                          class_name: &String, count_of_params: usize, invocation_data: &mut InvocationData) -> bool {

    if var_name != KeyWords::THIS && var_name != class_name { return false; }
    let package_opt = find_package_by_class_name(&class_name, invocation_data);
    if package_opt.is_none() { return false; }

    let package_description = package_opt.unwrap();
    let line_number = get_line_number(&node);
    let position = get_position_in_line(&node);
    let package_name = package_description.package_name();
    let class_name = package_description.class_name();

    let mut navigation_links = vec![];
    let navigation_link = MethodDescription::new(
        package_name.clone(),
        class_name.clone(),
        line_number,
        position,
        var_name.clone(),
        method_name.clone(),
        count_of_params,
    );

    navigation_links.push(navigation_link);

    for parent_class in package_description.parents() {
        let package_opt = find_package_by_class_name(parent_class, invocation_data);
        let package_name = if let Some(inner_package) = package_opt {
            inner_package.get_package_name()
        } else {
            package_name.clone()
        };

        let navigation_link = MethodDescription::new(
            package_name,
            class_name.clone(),
            line_number,
            position,
            var_name.clone(),
            method_name.clone(),
            count_of_params,
        );

        navigation_links.push(navigation_link);
    }

    invocation_data.mut_navigation_links().append(&mut navigation_links);

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
    } else if var_name.contains('.') || var_name.contains('(') {
        navigation_link.set_method_name(method_name.clone());
    } else {
        navigation_link.set_package_name(invocation_data.get_current_package());
        navigation_link.set_class_name(var_name.clone());
        navigation_link.set_var_name(var_name.clone());
        navigation_link.set_method_name(method_name.clone());
    }

    navigation_link.set_line(get_line_number(&node));
    navigation_link.set_position(get_position_in_line(&node));
    navigation_link.set_count_param_input(count_of_params );
    invocation_data.mut_navigation_links().push(navigation_link);

    return true;
}


/* Helpers */
fn count_params_from_node(node: &Node) -> usize {

    if let Some(arguments) = node.child_by_field_name(NodeNames::ARGUMENTS) {
        return arguments.named_child_count();
    }
    return 0;
}

fn var_name_from_invocation_or_this(node: &Node, invocation_data: &mut InvocationData) -> String {

    if let Some(name_node) = node.child_by_field_name(NodeNames::OBJECT) {
        return get_node_value(&name_node, invocation_data).unwrap_or(KeyWords::THIS.to_string());
    }
    return KeyWords::THIS.to_string();
}

fn var_name_from_obj_parent_or_this(node: &Node, invocation_data: &mut InvocationData) -> String {

    let parent_opt = node.parent();
    if parent_opt.is_none() {
        return KeyWords::THIS.to_string();
    }

    let parent = parent_opt.unwrap();
    if parent.kind() != NodeKinds::VARIABLE_DECLARATOR {
        return KeyWords::THIS.to_string();
    }

    let parent_name_opt = parent.child_by_field_name(NodeNames::NAME);
    if parent_name_opt.is_none() {
        return KeyWords::THIS.to_string();
    }

    return unwrap_or_empty_string!(get_node_value(&parent_name_opt.unwrap(), invocation_data));
}

fn find_var_desc_by_name<'spec>(name: &'spec String, invocation_data: &'spec InvocationData) -> Option<&'spec VarDescription> {

    invocation_data
        .var_descriptions()
        .iter()
        .rev()
        .find(|&x| x.class_name() == name)
}

fn get_child_node_by_kind<'time_spec>(node: &'time_spec Node, kind: &'time_spec str) -> Option<Node<'time_spec>> {
     node
        .children(&mut  node.walk())
        .filter(|x| { x.kind() == kind })
        .next()
}

fn get_node_value(node: &Node, invocation_data: &InvocationData) -> Option<String> {

    let source = invocation_data.source_code();
    let bytes = source.as_bytes();
    let mut node_bytes = &bytes[node.start_byte()..node.end_byte()];
    let mut node_string = KeyWords::EMPTY_STRING.to_string();

    node_bytes
        .read_to_string(&mut node_string)
        .expect(&format!(
            "RUST UNRECOVERABLE ERROR: Unable to read source code. Path file: {}",
            invocation_data.path())
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

fn get_name_from_node(node: &Node, invocation_data: &mut InvocationData) -> String {

    if node.kind() == NodeKinds::TYPE_IDENTIFIER {
        return unwrap_or_empty_string!(get_node_value(&node, invocation_data));
    }

    if let Some(type_node) = get_child_node_by_kind(node, NodeKinds::TYPE_IDENTIFIER)
    {
        return unwrap_or_empty_string!(get_node_value(&type_node, invocation_data));
    }
    else if let Some(generic_type) = get_child_node_by_kind(node, NodeKinds::GENERIC_TYPE)
    {
        return get_name_from_node(&generic_type, invocation_data);
    }
    else if let Some(scoped_type_identifier) = get_child_node_by_kind(node, NodeKinds::SCOPED_TYPE_IDENTIFIER)
    {
        return get_name_from_node(&scoped_type_identifier, invocation_data);
    }

    KeyWords::EMPTY_STRING.to_string()
}

fn find_package_by_class_name<'time>(class_name: &'time String, invocation_data: &'time InvocationData) -> Option<&'time PackageDescription> {

    let package_descriptions = invocation_data.package_descriptions();
    let named_package_opt = package_descriptions
        .iter()
        .find(|&x| x.class_name() == class_name);

    if let Some(named_package) = named_package_opt {
        return Some(named_package);
    }

    let unnamed_package_opt = package_descriptions
        .iter()
        .find(|&x| x.class_name() == KeyWords::EMPTY_STRING);

    if let Some(unnamed_package) = unnamed_package_opt {
        return Some(unnamed_package);
    }

    return None;
}

#[cfg(test)]
mod java_code_declaration_tests {

    use super::*;
    use tree_sitter::Parser;
    use std::fs;
    use serde;

    #[test]
    pub fn test_get_repository_method_dto() {
        let mut code = fs::read_to_string("resources/test_files/java/2.java.txt").unwrap();
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_java::language()).expect("ERROR: Unable to load Java grammar");
        let tree = parser.parse(&mut code, None).unwrap();
        println!("{},  ", tree.root_node().to_sexp());
        let mut repository_method_dto = get_file_structure(code,
                                                           tree, "test".to_string());


        println!("{}", serde_json::to_string_pretty(&repository_method_dto).unwrap());
    }
}