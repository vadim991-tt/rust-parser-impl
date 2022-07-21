use std::io::Read;
use tree_sitter::{Node, Tree};
use crate::dto::invocation_structure::{InvocationStructure, RepositoryImportDeclaration};
use crate::dto::object_description::{Description, MethodDescription, PackageDescription, VarDescription};
use crate::model::cpp_object::CodeType;
use crate::unwrap_or_return;
use crate::unwrap_or_empty_string;

struct NodeKinds;

impl NodeKinds {
    const CLASS_SPECIFIER: &'static str = "class_specifier";
    const STRUCT_SPECIFIER: &'static str = "struct_specifier";
    const NAMESPACE_DEFINITION: &'static str = "namespace_definition";
    const UNION_SPECIFIER: &'static str = "union_specifier";
    const DECLARATION: &'static str = "declaration";
    const FIELD_DECLARATION: &'static str = "field_declaration";
    const CALL_EXPRESSION: &'static str = "call_expression";
    const INIT_DECLARATOR: &'static str = "init_declarator";
    const BASE_CLASS_CLAUSE: &'static str = "base_class_clause";
    const PARAMETER_LIST: &'static str = "parameter_list";
    const PARAMETER_DECLARATION: &'static str = "parameter_declaration";
    const OPTIONAL_PARAMETER_DECLARATION: &'static str = "optional_parameter_declaration";
    const PRIMITIVE_TYPE: &'static str = "primitive_type";
    const FIELD_IDENTIFIER: &'static str = "field_identifier";
    const IDENTIFIER: &'static str = "identifier";
    const DESTRUCTOR_NAME: &'static str = "destructor_name";
    const SCOPED_IDENTIFIER: &'static str = "scoped_identifier";
    const TYPE_IDENTIFIER: &'static str = "type_identifier";
    const SIZED_TYPE_SPECIFIER: &'static str = "sized_type_specifier";
    const SCOPED_TYPE_IDENTIFIER: &'static str = "scoped_type_identifier";
    const TEMPLATE_TYPE: &'static str = "template_type";
    const INITIALIZER_LIST: &'static str = "initializer_list";
    const ARGUMENT_LIST: &'static str = "argument_list";
    const TEMPLATE_FUNCTION: &'static str = "template_function";
    const FIELD_EXPRESSION: &'static str = "field_expression";
}

struct NodeNames;

impl NodeNames {
    const DECLARATOR: &'static str = "declarator";
    const FUNCTION: &'static str = "function";
    const VALUE: &'static str = "value";
    const ARGUMENTS: &'static str = "arguments";
    const ARGUMENT: &'static str = "argument";
    const NAME: &'static str = "name";
    const TYPE: &'static str = "type";
    const FIELD: &'static str = "field";
}

struct KeyWords;

impl KeyWords {
    const THIS: &'static str = "this";
    const SUPER_CLASS: &'static str = "super";
    const EMPTY_STRING: &'static str = "";
}

const MAX_TOKEN_LENGTH: usize = 250;

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

    fn set_current_package(&mut self, current_package: String) {
        self.current_package = current_package;
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

    fn mut_import_list(&mut self) -> &mut Vec<RepositoryImportDeclaration> {
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

    fn mut_navigation_links(&mut self) -> &mut Vec<MethodDescription> {
        &mut self.links
    }

    fn take(self) -> (Vec<RepositoryImportDeclaration>, Vec<MethodDescription>) {
        (self.import_declarations, self.links)
    }

}

pub fn get_file_structure(source_code: String, tree: Tree, path: String) -> InvocationStructure {

    let mut invocation_data = InvocationData::new(source_code, path.clone());
    add_package_declaration(&mut invocation_data, path.clone());

    visit_node(tree.root_node(), &mut invocation_data, &mut KeyWords::EMPTY_STRING.to_string());

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

fn visit_node(node: Node, invocation_data: &mut InvocationData, class_name: &mut String) {

    for child in node.named_children(&mut node.walk()) {

        match child.kind() {

            /* Var descriptions */
            NodeKinds::NAMESPACE_DEFINITION => {
                *class_name = visit_namespace_definition_get_name(&child, invocation_data)
            }

            NodeKinds::CLASS_SPECIFIER | NodeKinds::STRUCT_SPECIFIER | NodeKinds::UNION_SPECIFIER => {
                *class_name = visit_struct_spec_get_name(&child, invocation_data);
            }

            NodeKinds::PARAMETER_LIST => visit_parameter_list(&child, invocation_data),
            NodeKinds::FIELD_DECLARATION => visit_field_declaration(&child, invocation_data),
            NodeKinds::BASE_CLASS_CLAUSE => visit_super_class(&child, invocation_data, class_name),

            /* Navigation links */
            NodeKinds::DECLARATION => visit_declaration(&child, invocation_data, class_name),
            NodeKinds::INIT_DECLARATOR => visit_init_declaration(&child, invocation_data, class_name),
            NodeKinds::CALL_EXPRESSION => visit_call_expression(&child, invocation_data, class_name),
            _ => {}
        }

        visit_node(child, invocation_data, class_name);
    }
}

fn visit_namespace_definition_get_name(node: &Node, invocation_data: &mut InvocationData) -> String {

    let namespace_opt = node.child_by_field_name(NodeNames::NAME);
    if namespace_opt.is_none() {
        return KeyWords::EMPTY_STRING.to_string();
    }

    let name_node = namespace_opt.unwrap();
    let namespace_name = unwrap_or_empty_string!(get_node_value(&name_node, invocation_data));
    let line = get_line_number(&name_node);
    let position = get_position_in_line(&name_node);

    let package_description = PackageDescription::new(
        invocation_data.get_current_package(),
        namespace_name.clone(),
        line,
        position,
        vec![],
    );

    invocation_data.mut_package_descriptions().push(package_description);

    let var_description = VarDescription::new(
        invocation_data.get_current_package(),
        namespace_name.clone(),
        line,
        position,
        KeyWords::THIS.to_string(),
    );

    invocation_data.mut_var_descriptions().push(var_description);

    return namespace_name;
}

fn visit_field_declaration(node: &Node, invocation_data: &mut InvocationData) {

    let type_node = unwrap_or_return!(node.child_by_field_name(NodeNames::TYPE));
    let declarator = unwrap_or_return!(node.child_by_field_name(NodeNames::DECLARATOR));
    let identifier = unwrap_or_return!(get_field_identifier_from_declarator(declarator));
    let field_name = unwrap_or_empty_string!(get_node_value(&identifier, invocation_data));
    let field_type = unwrap_or_empty_string!(get_node_value(&type_node, invocation_data));

    if field_name.is_empty() || field_type.is_empty() {
        return;
    }

    add_var_description(node, field_type, field_name, invocation_data);
}

fn visit_parameter_list(node: &Node, invocation_data: &mut InvocationData) {
    for child in node.named_children(&mut node.walk()) {
        match child.kind() {
            NodeKinds::PARAMETER_DECLARATION => visit_field_declaration(&child, invocation_data),
            NodeKinds::OPTIONAL_PARAMETER_DECLARATION => visit_field_declaration(&child, invocation_data),
            _ => { /* variadic parameter declaration */ }
        }
    }
}

fn visit_declaration(node: &Node, invocation_data: &mut InvocationData, class_name: &String) {

    let declarator = unwrap_or_return!(node.child_by_field_name(NodeNames::DECLARATOR));
    let type_node = unwrap_or_return!(node.child_by_field_name(NodeNames::TYPE));
    let identifier = unwrap_or_return!(get_identifier_from_declarator(declarator));
    let method_name = get_var_from_type_specifier_or_this(&type_node, invocation_data);
    let var_name = unwrap_or_empty_string!(get_node_value(&identifier, invocation_data));

    add_var_description(node, method_name.clone(), var_name.clone(), invocation_data);

    if declarator.kind() == NodeKinds::INIT_DECLARATOR && type_node.kind() == NodeKinds::PRIMITIVE_TYPE {
        return;
    }

    /* Zero arg initializer */
    add_navigation_link(node, &var_name, &method_name, &class_name, 0, invocation_data);
}

fn visit_init_declaration(node: &Node, invocation_data: &mut InvocationData, class_name: &String) {
    let declarator = unwrap_or_return!(node.child_by_field_name(NodeNames::DECLARATOR));
    let value = unwrap_or_return!(node.child_by_field_name(NodeNames::VALUE));
    let value_kind = value.kind();

    if value_kind != NodeKinds::INITIALIZER_LIST && value_kind != NodeKinds::ARGUMENT_LIST {
        return;
    }

    let var_name = get_var_from_declarator(&declarator, invocation_data);
    let method_name = get_constr_name_from_parent_or_this(&node, invocation_data);
    let count_of_params: usize = value.named_child_count();

    add_navigation_link(node, &var_name, &method_name, &class_name, count_of_params, invocation_data);
}

fn visit_call_expression(node: &Node, invocation_data: &mut InvocationData, class_name: &String) {
    let function_node = unwrap_or_return!(node.child_by_field_name(NodeNames::FUNCTION));
    let arguments_node = unwrap_or_return!(node.child_by_field_name(NodeNames::ARGUMENTS));
    let (method_name, var_name) = get_method_and_var_from_expression(&function_node, invocation_data);
    let count_of_params: usize = arguments_node.named_child_count();

    add_navigation_link(node, &var_name, &method_name, &class_name, count_of_params, invocation_data);
}

fn visit_struct_spec_get_name(node: &Node, invocation_data: &mut InvocationData) -> String {
    let name_node_opt = node.child_by_field_name(NodeNames::NAME);
    if name_node_opt.is_none() { return KeyWords::EMPTY_STRING.to_string(); }
    let name_node = name_node_opt.unwrap();
    let class_name = get_name_from_class_name_node(&name_node, invocation_data);
    let line = get_line_number(&name_node);
    let position = get_position_in_line(&name_node);

    let package_description = PackageDescription::new(
        invocation_data.get_current_package(),
        class_name.clone(),
        line,
        position,
        vec![],
    );

    invocation_data.mut_package_descriptions().push(package_description);

    let var_description = VarDescription::new(
        invocation_data.get_current_package(),
        class_name.clone(),
        line,
        position,
        KeyWords::THIS.to_string(),
    );

    invocation_data.mut_var_descriptions().push(var_description);

    return class_name;
}

fn visit_super_class(node: &Node, invocation_data: &mut InvocationData, child_class_name: &String) {

    let mut inheritance_vector = vec![];

    for child in node.named_children(&mut node.walk()) {

        let class_name = get_name_from_class_name_node(&child, invocation_data);
        if class_name.is_empty() {
            continue;
        }

        let class_line = get_line_number(&child);
        let position = get_position_in_line(&child);

        let package_description = PackageDescription::new(
            invocation_data.get_current_package(),
            class_name.clone(),
            class_line,
            position,
            vec![],
        );

        invocation_data.mut_package_descriptions().push(package_description);

        let var_description = VarDescription::new(
            invocation_data.get_current_package(),
            class_name.clone(),
            class_line,
            position,
            KeyWords::SUPER_CLASS.to_string(),
        );

        invocation_data.mut_var_descriptions().push(var_description);
        inheritance_vector.push(class_name);
    }

    let package_desc = unwrap_or_return!(mut_package_by_class_name(child_class_name, invocation_data));
    package_desc.mut_parents().append(&mut inheritance_vector);
}

fn add_package_declaration(invocation_data: &mut InvocationData, path: String) {

    invocation_data
        .mut_import_list()
        .push(RepositoryImportDeclaration::new(path.clone()));

    invocation_data.set_current_package(path);
}

fn add_navigation_link(node: &Node, var_name: &String, method_name: &String, class_name: &String,
                       count_of_params: usize, invocation_data: &mut InvocationData) {

    if add_link_from_var(&node, &var_name, &method_name, count_of_params, invocation_data) {
        return;
    }

    if add_links_from_package(&node, &var_name, &method_name, &class_name,
                              count_of_params, invocation_data)
    {
        return;
    }

    if add_link(&node, &var_name, &method_name, count_of_params, invocation_data) {
        return;
    }
}

fn add_var_description(node: &Node, class_name: String, param_name: String, invocation_data: &mut InvocationData) {

    let mut var_description = VarDescription::default();
    if let Some(package_desc) = package_by_class_name(&class_name, invocation_data) {
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

fn add_link_from_var(node: &Node, var_name: &String, method_name: &String,
                     count_of_params: usize, invocation_data: &mut InvocationData) -> bool {

    if var_name == KeyWords::THIS { return false; }
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

    true
}

fn add_link(node: &Node, var_name: &String, method_name: &String, count_of_params: usize, invocation_data: &mut InvocationData) -> bool {

    let mut navigation_link = MethodDescription::default();

    if let Some(parent_description) = package_by_class_name(&var_name, invocation_data) {
        navigation_link.set_package_name(parent_description.get_package_name());
        navigation_link.set_class_name(parent_description.get_class_name());
        navigation_link.set_var_name(var_name.clone());
        navigation_link.set_method_name(method_name.clone());
    } else if var_name.contains(':') || var_name.contains('(') {
        navigation_link.set_method_name(method_name.clone()); /* Only method name */
    } else {
        navigation_link.set_package_name(invocation_data.get_current_package());
        navigation_link.set_class_name(var_name.clone());
        navigation_link.set_var_name(var_name.clone());
        navigation_link.set_method_name(method_name.clone());
    }

    navigation_link.set_line(get_line_number(&node));
    navigation_link.set_position(get_position_in_line(&node));
    navigation_link.set_count_param_input(count_of_params);

    invocation_data.mut_navigation_links().push(navigation_link);

    return true;
}

fn add_links_from_package(node: &Node, var_name: &String, method_name: &String,
                          class_name: &String, count_of_params: usize, invocation_data: &mut InvocationData) -> bool {

    if var_name != KeyWords::THIS && var_name != class_name {
        return false;
    }

    let package_opt = package_by_class_name(&class_name, invocation_data);
    if package_opt.is_none() {
        return false;
    }

    let mut navigation_links = vec![];
    let package_description = package_opt.unwrap();
    let line_number = get_line_number(&node);
    let position_in_line = get_position_in_line(&node);

    let navigation_link = MethodDescription::new(
        package_description.get_package_name(),
        package_description.get_class_name(),
        line_number,
        position_in_line,
        var_name.clone(),
        method_name.clone(),
        count_of_params,
    );

    navigation_links.push(navigation_link);

    for parent_class in package_description.parents() {
        let package_opt = package_by_class_name(parent_class, invocation_data);
        let package_name = if let Some(inner_package) = package_opt {
            inner_package.get_package_name()
        } else {
            package_description.get_package_name()
        };

        let navigation_link = MethodDescription::new(
            package_name,
            package_description.get_class_name(),
            line_number,
            position_in_line,
            var_name.clone(),
            method_name.clone(),
            count_of_params,
        );

        navigation_links.push(navigation_link);
    }

    invocation_data.mut_navigation_links().append(&mut navigation_links);

    true
}

/* Helpers */
fn find_var_desc_by_name<'time>(name: &'time String, invocation_data: &'time InvocationData) -> Option<&'time VarDescription> {
    /* To find latest added variable rev() function is used */
    invocation_data
        .var_descriptions()
        .iter()
        .rev()
        .find(|&x| x.var_name() == name)
}

fn package_by_class_name<'time>(class_name: &'time String, invocation_data: &'time InvocationData) -> Option<&'time PackageDescription> {

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

fn mut_package_by_class_name<'time>(class_name: &'time String,
                                    invocation_data: &'time mut InvocationData) -> Option<&'time mut PackageDescription> {
    return invocation_data
        .mut_package_descriptions()
        .into_iter()
        .find(|x| x.class_name() == class_name);
}

fn get_constr_name_from_parent_or_this(node: &Node, invocation_data: &mut InvocationData) -> String {

    /* This specifier is used to emphasize that function call belongs to this module */
    let parent_opt = node.parent();
    if parent_opt.is_none() {
        return KeyWords::THIS.to_string();
    }

    let parent = parent_opt.unwrap();
    if parent.kind() != NodeKinds::DECLARATION {
        return KeyWords::THIS.to_string();
    }

    let type_node_opt = parent.child_by_field_name(NodeNames::TYPE);
    if type_node_opt.is_none() {
        return KeyWords::THIS.to_string();
    }

    return get_var_from_type_specifier_or_this(&type_node_opt.unwrap(), invocation_data);
}

fn get_var_from_type_specifier_or_this(node: &Node, invocation_data: &InvocationData) -> String {

    /* This specifier is used to emphasize that function call belongs to this module */
    let output_param = match node.kind() {

        NodeKinds::TYPE_IDENTIFIER | NodeKinds::PRIMITIVE_TYPE | NodeKinds::SIZED_TYPE_SPECIFIER
        => unwrap_or_empty_string!(get_node_value(&node, invocation_data)),

        NodeKinds::SCOPED_TYPE_IDENTIFIER => get_name_from_scoped_type_identifier(&node, invocation_data),

        NodeKinds::TEMPLATE_TYPE => {
            if let Some(name_node) = node.child_by_field_name(NodeNames::NAME) {
                get_var_from_type_specifier_or_this(&name_node, invocation_data)
            } else {
                KeyWords::EMPTY_STRING.to_string()
            }
        }

        _ => KeyWords::EMPTY_STRING.to_string()
    };

    return if !(output_param.is_empty()) { output_param } else { KeyWords::THIS.to_string() };
}

fn get_var_from_declarator(node: &Node, invocation_data: &mut InvocationData) -> String {

    match node.kind() {

        NodeKinds::FIELD_IDENTIFIER | NodeKinds::IDENTIFIER | NodeKinds::DESTRUCTOR_NAME
        => unwrap_or_empty_string!(get_node_value(&node, invocation_data)),

        NodeKinds::SCOPED_IDENTIFIER => {
            if let Some(name_node) = node.child_by_field_name(NodeNames::NAME) {
                get_var_from_declarator(&name_node, invocation_data)
            } else {
                KeyWords::EMPTY_STRING.to_string()
            }
        }

        _ => KeyWords::EMPTY_STRING.to_string()
    }
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

fn get_field_identifier_from_declarator(node: Node) -> Option<Node> {
    if node.kind() == NodeKinds::FIELD_IDENTIFIER {
        return Some(node);
    }
    for child in node.named_children(&mut node.walk()) {
        return get_field_identifier_from_declarator(child);
    }
    return None;
}

fn get_name_from_scoped_type_identifier(node: &Node, invocation_data: &InvocationData) -> String {
    let name_opt = node.child_by_field_name(NodeNames::NAME);
    if name_opt.is_none() {
        return KeyWords::EMPTY_STRING.to_string();
    }
    return unwrap_or_empty_string!(get_node_value(&name_opt.unwrap(), invocation_data));
}

fn get_node_value(node: &Node, invocation_data: &InvocationData) -> Option<String> {

    let source = invocation_data.source_code();
    let source_bytes = source.as_bytes();
    let mut node_bytes = &source_bytes[node.start_byte()..node.end_byte()];
    let mut node_string = String::new();

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

fn get_name_from_class_name_node(node: &Node, invocation_data: &mut InvocationData) -> String {

    return match node.kind() {

        NodeKinds::TYPE_IDENTIFIER => unwrap_or_empty_string!(get_node_value(&node, invocation_data)),

        NodeKinds::SCOPED_TYPE_IDENTIFIER => get_name_from_scoped_type_identifier(&node, invocation_data),

        NodeKinds::TEMPLATE_TYPE => {

            let name_opt = node.child_by_field_name(NodeNames::NAME);
            if name_opt.is_none() {
                return KeyWords::EMPTY_STRING.to_string();
            }

            get_name_from_class_name_node(&name_opt.unwrap(), invocation_data)
        }

        &_ => KeyWords::EMPTY_STRING.to_string()
    };
}

fn get_method_and_var_from_expression(node: &Node, invocation_data: &mut InvocationData) -> (String, String) {

    let mut method_name = KeyWords::EMPTY_STRING.to_string();
    let mut var_name = KeyWords::EMPTY_STRING.to_string();

    match node.kind() {

        NodeKinds::IDENTIFIER => method_name = unwrap_or_empty_string!(get_node_value(&node, invocation_data)),

        NodeKinds::TEMPLATE_FUNCTION | NodeKinds::SCOPED_IDENTIFIER
        => method_name = get_name_from_template_or_scoped_node(&node, invocation_data),

        NodeKinds::FIELD_EXPRESSION => {

            let (
                field_method_name,
                field_var_name
            ) = get_method_and_var_from_field_expression(&node, invocation_data);

            method_name = field_method_name;
            var_name = field_var_name;
        }

        _ => {}
    }

    return (method_name, var_name);
}

fn get_method_and_var_from_field_expression(node: &Node, invocation_data: &mut InvocationData) -> (String, String) {

    let mut method_name = KeyWords::EMPTY_STRING.to_string();
    let mut var_name = KeyWords::EMPTY_STRING.to_string();

    if let Some(field_node) = node.child_by_field_name(NodeNames::FIELD) {
        method_name = unwrap_or_empty_string!(get_node_value(&field_node, invocation_data));
    }

    if let Some(argument_node) = node.child_by_field_name(NodeNames::ARGUMENT) {
        var_name = match argument_node.kind() {
            NodeKinds::IDENTIFIER => unwrap_or_empty_string!(get_node_value(&argument_node, invocation_data)),
            NodeKinds::TEMPLATE_FUNCTION | NodeKinds::SCOPED_IDENTIFIER
            => get_name_from_template_or_scoped_node(&argument_node, invocation_data),
            _ => KeyWords::EMPTY_STRING.to_string()
        }
    }

    return (method_name, var_name);
}

fn get_name_from_template_or_scoped_node(node: &Node, invocation_data: &mut InvocationData) -> String {

    if let Some(name_node) = node.child_by_field_name(NodeNames::NAME) {
        match name_node.kind() {
            NodeKinds::IDENTIFIER => unwrap_or_empty_string!(get_node_value(&name_node, invocation_data)),
            NodeKinds::SCOPED_IDENTIFIER => get_name_from_template_or_scoped_node(&name_node, invocation_data),
            _ => KeyWords::EMPTY_STRING.to_string()
        }
    } else {
        KeyWords::EMPTY_STRING.to_string()
    }
}

#[cfg(test)]
mod cpp_code_invocation_tests {

    use super::*;
    use tree_sitter::Parser;
    use std::fs;


    #[test]
    pub fn test_get_invocation_structure() {
        let mut code = fs::read_to_string("resources/test_files/cpp/5.cc.txt").unwrap();
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_cpp::language()).expect("ERROR: Unable to load C++ grammar");
        let tree = parser.parse(&mut code, None).unwrap();
        println!("{},  ", tree.root_node().to_sexp());
        let mut structure = get_file_structure(code, tree, "test".to_string());
        println!("{}", serde_json::to_string_pretty(&structure).unwrap());
    }
}
