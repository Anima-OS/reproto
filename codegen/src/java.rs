/// A code generator inspired by JavaPoet (https://github.com/square/javapoet)

use std::collections::BTreeSet;

use errors::*;

fn java_quote_string(input: &str) -> String {
    let mut out = String::new();
    let mut it = input.chars();

    out.push('"');

    while let Some(c) = it.next() {
        match c {
            '\t' => out.push_str("\\t"),
            '\u{0007}' => out.push_str("\\b"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\u{0014}' => out.push_str("\\f"),
            '\'' => out.push_str("\\'"),
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c => out.push(c),
        }
    }

    out.push('"');
    out
}

fn format_statement_part_format(format: &String, variables: &Vec<Variable>) -> Result<String> {
    let mut out = String::new();

    let mut it = format.chars();
    let mut var_it = variables.iter();

    while let Some(c) = it.next() {
        match c {
            '$' => {
                let kind: char = it.next().ok_or(ErrorKind::InvalidEscape)?;
                let var = var_it.next().ok_or(ErrorKind::VariableUnderflow)?;

                match kind {
                    'L' => {
                        if let Variable::Literal(ref literal) = *var {
                            out.push_str(literal);
                        } else {
                            return Err(ErrorKind::InvalidVariable.into());
                        }
                    }
                    'T' => {
                        if let Variable::TypeSpec(ref type_) = *var {
                            out.push_str(&type_.format()?);
                        } else {
                            return Err(ErrorKind::InvalidVariable.into());
                        }
                    }
                    'S' => {
                        if let Variable::String(ref string) = *var {
                            out.push_str(&java_quote_string(string));
                        } else {
                            return Err(ErrorKind::InvalidVariable.into());
                        }
                    }
                    'N' => {
                        if let Variable::Name(ref name) = *var {
                            out.push_str(name);
                        } else {
                            return Err(ErrorKind::InvalidVariable.into());
                        }
                    }
                    '$' => {
                        if let Variable::Statement(ref stmt) = *var {
                            let lines = stmt.format()?;
                            out.push_str(&lines.join(" "));
                        } else {
                            return Err(ErrorKind::InvalidVariable.into());
                        }
                    }
                    _ => return Err(ErrorKind::InvalidEscape.into()),
                }
            }
            c => out.push(c),
        }
    }

    Ok(out)
}

fn add_annotations(annotations: &Vec<AnnotationSpec>, target: &mut Statement) -> Result<()> {
    if annotations.is_empty() {
        return Ok(());
    }

    for a in annotations {
        target.push_statement(a.as_statement()?);
        target.push_spacing();
    }

    Ok(())
}

fn add_arguments<S>(arguments: &Vec<S>, target: &mut Statement) -> Result<()>
    where S: AsStatement
{
    if arguments.is_empty() {
        return Ok(());
    }

    let mut out: Statement = Statement::new();

    for a in arguments {
        out.push_statement(a.as_statement()?);
    }

    target.push_statement(out.join(", "));

    Ok(())
}

pub trait Imports {
    fn imports<I>(&self, &mut I) where I: ImportReceiver;
}

pub trait ImportReceiver {
    fn receive(&mut self, type_: &Type);

    fn import_all<T>(&mut self, sources: &Vec<T>)
        where T: Imports,
              Self: Sized
    {
        for source in sources {
            source.imports(self);
        }
    }
}

/// Trait allowing a type to be converted to a statement.
pub trait AsStatement {
    fn as_statement(&self) -> Result<Statement>;
}

impl AsStatement for Statement {
    fn as_statement(&self) -> Result<Statement> {
        Ok(self.clone())
    }
}

impl ImportReceiver for BTreeSet<Type> {
    fn receive(&mut self, type_: &Type) {
        self.insert(type_.clone());
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum Modifier {
    Public,
    Protected,
    Private,
    Static,
    Final,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Modifiers {
    pub modifiers: BTreeSet<Modifier>,
}

impl Modifiers {
    pub fn new() -> Modifiers {
        Modifiers { modifiers: BTreeSet::new() }
    }

    pub fn insert(&mut self, modifier: Modifier) {
        self.modifiers.insert(modifier);
    }

    pub fn format(&self) -> Result<String> {
        let mut out: Vec<String> = Vec::new();

        for m in &self.modifiers {
            out.push(match *m {
                Modifier::Public => "public".to_owned(),
                Modifier::Protected => "protected".to_owned(),
                Modifier::Private => "private".to_owned(),
                Modifier::Static => "static".to_owned(),
                Modifier::Final => "final".to_owned(),
            });
        }

        Ok(out.join(" "))
    }

    pub fn is_empty(&self) -> bool {
        self.modifiers.is_empty()
    }
}

#[derive(Debug, Clone)]
pub enum Section {
    Block(Block),
    Statement(Statement),
    Spacing,
}

impl Imports for Section {
    fn imports<I>(&self, receiver: &mut I)
        where I: ImportReceiver
    {
        match *self {
            Section::Block(ref block) => block.imports(receiver),
            Section::Statement(ref statement) => statement.imports(receiver),
            _ => {}
        };
    }
}

#[derive(Debug, Clone)]
pub enum Variable {
    Literal(String),
    TypeSpec(TypeSpec),
    String(String),
    Name(String),
    Statement(Statement),
}

#[derive(Debug, Clone)]
pub enum StatementPart {
    // literal part
    Literal(String),
    // formatted part
    Format(String, Vec<Variable>),
    // nested statement
    Statement(Statement),
    // spacing
    Spacing,
}

#[derive(Debug, Clone)]
pub struct Statement {
    parts: Vec<StatementPart>,
}

impl Statement {
    pub fn new() -> Statement {
        Statement { parts: Vec::new() }
    }

    pub fn push_spacing(&mut self) {
        self.parts.push(StatementPart::Spacing);
    }

    pub fn push_literal(&mut self, literal: &str) {
        self.parts.push(StatementPart::Literal(literal.to_owned()));
    }

    pub fn push_statement(&mut self, statement: Statement) {
        self.parts.push(StatementPart::Statement(statement));
    }

    pub fn push(&mut self, format: &str, variables: Vec<Variable>) {
        self.parts.push(StatementPart::Format(format.to_owned(), variables));
    }

    pub fn join(self, literal: &str) -> Statement {
        let mut it = self.parts.into_iter();

        let part = match it.next() {
            Some(part) => part,
            None => return Statement::new(),
        };

        let mut parts: Vec<StatementPart> = Vec::new();
        parts.push(part);

        while let Some(part) = it.next() {
            parts.push(StatementPart::Literal(literal.to_owned()));
            parts.push(part);
        }

        Statement { parts: parts }
    }

    pub fn imports<I>(&self, receiver: &mut I)
        where I: ImportReceiver
    {
        for part in &self.parts {
            match *part {
                StatementPart::Format(_, ref variables) => {
                    for var in variables {
                        if let Variable::TypeSpec(ref type_) = *var {
                            type_.imports(receiver);
                        }
                    }
                }
                StatementPart::Statement(ref stmt) => {
                    stmt.imports(receiver);
                }
                _ => {}
            }
        }
    }

    pub fn format(&self) -> Result<Vec<String>> {
        let mut out: Vec<String> = Vec::new();
        let mut current: Vec<String> = Vec::new();

        for part in &self.parts {
            match *part {
                StatementPart::Format(ref format, ref variables) => {
                    current.push(format_statement_part_format(format, variables)?);
                }
                StatementPart::Statement(ref stmt) => {
                    current.push(stmt.format()?.join(" "));
                }
                StatementPart::Literal(ref string) => {
                    current.push(string.clone());
                }
                StatementPart::Spacing => {
                    out.push(current.join(""));
                    current.clear();
                }
            }
        }

        if !current.is_empty() {
            out.push(current.join(""));
            current.clear();
        }

        Ok(out)
    }
}

#[macro_export]
macro_rules! stmt {
    ($($fmt:expr, $vars:expr),*) => {{
        let mut s = Statement::new();
        $(s.push($fmt, $vars);)*
        s
    }};

    ($fmt:expr, $($tail:tt)*) => {{
        let mut s = Statement::new();
        let mut vars = Vec::new();
        vars.extend(stmt!($($tail)*));
        s.push($fmt, vars);
        s
    }};

    (type_spec $var:expr) => {{
        vec![Variable::TypeSpec($var.as_type_spec())]
    }};

    (type_spec $var:expr, $($tail:tt)*) => {{
        let mut vars = vec![Variable::TypeSpec($var.as_type_spec())];
        vars.extend(stmt!($($tail)*));
        vars
    }};

    (name $var:expr) => {{
        vec![Variable::Name($var.as_name())]
    }};

    (name $var:expr, $($tail:tt)*) => {{
        let mut vars = vec![Variable::Name($var.as_name())];
        vars.extend(stmt!($($tail)*));
        vars
    }};

    (literal $var:expr) => {{
        vec![Variable::Literal($var)]
    }};

    (literal $var:expr, $($tail:tt)*) => {{
        let mut vars = vec![Variable::Literal($var)];
        vars.extend(stmt!($($tail)*));
        vars
    }};

    (string $var:expr) => {{
        vec![Variable::String($var.to_owned())]
    }};

    (string $var:expr, $($tail:tt)*) => {{
        let mut vars = vec![Variable::String($var.to_owned())];
        vars.extend(stmt!($($tail)*));
        vars
    }};

    (stmt $var:expr) => {{
        vec![Variable::Statement($var.clone())]
    }};

    (stmt $var:expr, $($tail:tt)*) => {{
        let mut vars = vec![Variable::Statement($var.clone())];
        vars.extend(stmt!($($tail)*));
        vars
    }};
}

#[macro_export]
macro_rules! mods {
    ($($modifier:expr),*) => {
        {
            let mut tmp_modifiers = Modifiers::new();

            $(
                tmp_modifiers.insert($modifier);
            )*

            tmp_modifiers
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sections {
    sections: Vec<Section>,
}

impl Sections {
    pub fn new() -> Sections {
        Sections { sections: Vec::new() }
    }

    pub fn push_statement(&mut self, statement: &Statement) {
        self.sections.push(Section::Statement(statement.clone()));
    }

    pub fn push_spacing(&mut self) {
        self.sections.push(Section::Spacing);
    }

    pub fn push_block(&mut self, block: &Block) {
        self.sections.push(Section::Block(block.clone()));
    }

    pub fn extend(&mut self, sections: &Sections) {
        self.sections.extend(sections.sections.iter().map(Clone::clone));
    }

    pub fn format(&self, current: &str, indent: &str) -> Result<Vec<String>> {
        let mut out = Vec::new();

        for section in &self.sections {
            match *section {
                Section::Statement(ref statement) => {
                    for line in statement.format()? {
                        out.push(format!("{}{};", current, line));
                    }
                }
                Section::Block(ref block) => {
                    out.extend(block.format(current, indent)?);
                }
                Section::Spacing => {
                    out.push("".to_owned());
                }
            }
        }

        Ok(out)
    }
}

impl Imports for Sections {
    fn imports<I>(&self, receiver: &mut I)
        where I: ImportReceiver
    {
        receiver.import_all(&self.sections);
   }
}

#[derive(Debug, Clone)]
pub struct Block {
    open: Option<Statement>,
    close: Option<Statement>,
    sections: Sections,
}

impl Block {
    pub fn new() -> Block {
        Block {
            open: None,
            close: None,
            sections: Sections::new(),
        }
    }

    pub fn open(&mut self, open: Statement) {
        self.open = Some(open)
    }

    pub fn close(&mut self, close: Statement) {
        self.close = Some(close)
    }

    pub fn push_statement(&mut self, statement: &Statement) {
        self.sections.push_statement(statement);
    }

    pub fn push_spacing(&mut self) {
        self.sections.push_spacing();
    }

    pub fn push_block(&mut self, block: &Block) {
        self.sections.push_block(block);
    }

    pub fn extend(&mut self, sections: &Sections) {
        self.sections.extend(sections);
    }

    pub fn imports<I>(&self, receiver: &mut I)
        where I: ImportReceiver
    {
        if let Some(ref open) = self.open {
            open.imports(receiver);
        }

        if let Some(ref close) = self.close {
            close.imports(receiver);
        }

        self.sections.imports(receiver);
    }

    pub fn format(&self, current: &str, indent: &str) -> Result<Vec<String>> {
        let mut out = Vec::new();

        if let Some(ref open) = self.open {
            let mut it = open.format()?.into_iter().peekable();

            while let Some(line) = it.next() {
                if it.peek().is_none() {
                    out.push(format!("{}{} {{", current, line).to_owned());
                } else {
                    out.push(format!("{}{}", current, line).to_owned());
                }
            }
        } else {
            out.push(format!("{}{{", current).to_owned());
        }

        out.extend(self.sections.format(&format!("{}{}", current, indent), indent)?);

        if let Some(ref close) = self.close {
            let close = close.format()?.join(" ");
            out.push(format!("{}}} {};", current, close).to_owned());
        } else {
            out.push(format!("{}}}", current).to_owned());
        }

        Ok(out)
    }
}


/// Raw (importable) types.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Type {
    pub package: String,
    pub name: String,
}

impl Type {
    pub fn new(package: &str, name: &str) -> Type {
        Type {
            package: package.to_owned(),
            name: name.to_owned(),
        }
    }

    pub fn with_arguments<I>(&self, arguments: Vec<I>) -> TypeSpec
        where I: AsTypeSpec
    {
        let arguments: Vec<TypeSpec> = arguments.iter().map(AsTypeSpec::as_type_spec).collect();
        TypeSpec::new(self.clone(), arguments)
    }
}

/// Implementation for TypeSpec reference (&TypeSpec) to TypeSpec conversion.
impl<'a> AsTypeSpec for &'a TypeSpec {
    fn as_type_spec(&self) -> TypeSpec {
        (*self).clone()
    }
}

/// Implementation for Type reference (&Type) to TypeSpec conversion.
impl<'a> AsTypeSpec for &'a Type {
    fn as_type_spec(&self) -> TypeSpec {
        TypeSpec::new((*self).clone(), vec![])
    }
}

/// Implementation for Type to TypeSpec conversion.
impl AsTypeSpec for Type {
    fn as_type_spec(&self) -> TypeSpec {
        TypeSpec::new(self.clone(), vec![])
    }
}

/// Trait for types that can be converted into TypeSpec's
pub trait AsTypeSpec {
    fn as_type_spec(&self) -> TypeSpec;
}

pub trait AsName {
    fn as_name(&self) -> String;
}

/// Complete types, including generic arguments.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct TypeSpec {
    pub raw: Type,
    pub arguments: Vec<TypeSpec>,
}

impl TypeSpec {
    pub fn new(raw: Type, arguments: Vec<TypeSpec>) -> TypeSpec {
        TypeSpec {
            raw: raw,
            arguments: arguments,
        }
    }

    pub fn imports<I>(&self, receiver: &mut I)
        where I: ImportReceiver
    {
        receiver.receive(&self.raw);

        for t in &self.arguments {
            t.imports(receiver);
        }
    }

    pub fn format(&self) -> Result<String> {
        let mut out = String::new();

        out.push_str(&self.raw.name);

        if !self.arguments.is_empty() {
            let mut arguments = Vec::new();

            for g in &self.arguments {
                arguments.push(g.format()?);
            }

            let joined = arguments.join(", ");

            out.push('<');
            out.push_str(&joined);
            out.push('>');
        }

        Ok(out)
    }
}

impl AsTypeSpec for TypeSpec {
    fn as_type_spec(&self) -> TypeSpec {
        self.clone()
    }
}

#[derive(Debug, Clone)]
pub struct MethodArgument {
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone)]
pub struct FieldSpec {
    pub modifiers: Modifiers,
    pub type_: TypeSpec,
    pub name: String,
}

impl FieldSpec {
    pub fn new<I>(modifiers: Modifiers, type_: I, name: &str) -> FieldSpec
        where I: AsTypeSpec
    {
        FieldSpec {
            modifiers: modifiers,
            type_: type_.as_type_spec(),
            name: name.to_owned(),
        }
    }

    pub fn imports<I>(&self, receiver: &mut I)
        where I: ImportReceiver
    {
        self.type_.imports(receiver);
    }
}

impl AsStatement for FieldSpec {
    fn as_statement(&self) -> Result<Statement> {
        let mut s = Statement::new();

        if !self.modifiers.is_empty() {
            s.push("$L ", stmt!(literal self.modifiers.format()?));
        }

        s.push("$T ", stmt![type_spec self.type_]);
        s.push("$L", stmt!(literal self.name.clone()));

        Ok(s)
    }
}

impl AsName for FieldSpec {
    fn as_name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, Clone)]
pub struct ConstructorSpec {
    pub modifiers: Modifiers,
    pub annotations: Vec<AnnotationSpec>,
    pub arguments: Vec<ArgumentSpec>,
    pub sections: Sections,
}

impl ConstructorSpec {
    pub fn new(modifiers: Modifiers) -> ConstructorSpec {
        ConstructorSpec {
            modifiers: modifiers,
            annotations: Vec::new(),
            arguments: Vec::new(),
            sections: Sections::new(),
        }
    }

    pub fn push_annotation(&mut self, annotation: &AnnotationSpec) {
        self.annotations.push(annotation.clone());
    }

    pub fn push_argument(&mut self, argument: &ArgumentSpec) {
        self.arguments.push(argument.clone());
    }

    pub fn push_statement(&mut self, statement: &Statement) {
        self.sections.push_statement(statement);
    }

    pub fn as_block(&self, enclosing: &str) -> Result<Block> {
        let mut open = Statement::new();

        add_annotations(&self.annotations, &mut open)?;

        if !self.modifiers.is_empty() {
            open.push("$L ", stmt!(literal self.modifiers.format()?));
        }

        open.push("$L(", stmt![literal enclosing.to_owned()]);
        add_arguments(&self.arguments, &mut open)?;
        open.push(")", vec![]);

        let mut block = Block::new();
        block.open(open);
        block.extend(&self.sections);

        Ok(block)
    }
}

impl Imports for ConstructorSpec {
    fn imports<I>(&self, receiver: &mut I)
        where I: ImportReceiver
    {
        self.sections.imports(receiver);
        receiver.import_all(&self.annotations);
        receiver.import_all(&self.arguments);
    }
}

#[derive(Debug, Clone)]
pub struct AnnotationSpec {
    pub type_: TypeSpec,
    pub arguments: Vec<Statement>,
}

impl AnnotationSpec {
    pub fn new<I>(type_: I) -> AnnotationSpec
        where I: AsTypeSpec
    {
        AnnotationSpec {
            type_: type_.as_type_spec(),
            arguments: Vec::new(),
        }
    }

    pub fn push_argument(&mut self, statement: &Statement) {
        self.arguments.push(statement.clone());
    }
}

impl AsStatement for AnnotationSpec {
    fn as_statement(&self) -> Result<Statement> {
        let mut s = Statement::new();
        s.push("@$T", stmt![type_spec self.type_]);

        if !self.arguments.is_empty() {
            s.push_literal("(");
            add_arguments(&self.arguments, &mut s)?;
            s.push_literal(")");
        }

        Ok(s)
    }
}

impl Imports for AnnotationSpec {
    fn imports<I>(&self, receiver: &mut I)
        where I: ImportReceiver
    {
        self.type_.imports(receiver);

        for a in &self.arguments {
            a.imports(receiver);
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArgumentSpec {
    pub modifiers: Modifiers,
    pub type_: TypeSpec,
    pub name: String,
    pub annotations: Vec<AnnotationSpec>,
}

impl ArgumentSpec {
    pub fn new<I>(modifiers: Modifiers, type_: I, name: &str) -> ArgumentSpec
        where I: AsTypeSpec
    {
        ArgumentSpec {
            modifiers: modifiers,
            type_: type_.as_type_spec(),
            name: name.to_owned(),
            annotations: Vec::new(),
        }
    }

    pub fn push_annotation(&mut self, annotation: &AnnotationSpec) {
        self.annotations.push(annotation.clone());
    }
}

impl AsStatement for ArgumentSpec {
    fn as_statement(&self) -> Result<Statement> {
        let mut s = Statement::new();

        add_annotations(&self.annotations, &mut s)?;

        if !self.modifiers.is_empty() {
            s.push("$L ", stmt!(literal self.modifiers.format()?));
        }

        s.push("$T ", stmt![type_spec self.type_.clone()]);
        s.push("$L", stmt!(literal self.name.clone()));

        Ok(s)
    }
}

impl Imports for ArgumentSpec {
    fn imports<I>(&self, receiver: &mut I)
        where I: ImportReceiver
    {
        self.type_.imports(receiver);

        for a in &self.annotations {
            a.imports(receiver);
        }
    }
}

impl AsName for ArgumentSpec {
    fn as_name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, Clone)]
pub struct MethodSpec {
    pub modifiers: Modifiers,
    pub name: String,
    pub annotations: Vec<AnnotationSpec>,
    pub arguments: Vec<ArgumentSpec>,
    pub returns: Option<TypeSpec>,
    pub sections: Sections,
}

impl MethodSpec {
    pub fn new(modifiers: Modifiers, name: &str) -> MethodSpec {
        MethodSpec {
            modifiers: modifiers,
            name: name.to_owned(),
            annotations: Vec::new(),
            arguments: Vec::new(),
            returns: None,
            sections: Sections::new(),
        }
    }

    pub fn push_annotation(&mut self, annotation: &AnnotationSpec) {
        self.annotations.push(annotation.clone());
    }

    pub fn push_argument(&mut self, argument: &ArgumentSpec) {
        self.arguments.push(argument.clone());
    }

    pub fn returns<I>(&mut self, returns: I)
        where I: AsTypeSpec
    {
        self.returns = Some(returns.as_type_spec())
    }

    pub fn push_statement(&mut self, statement: &Statement) {
        self.sections.push_statement(statement);
    }

    pub fn as_block(&self) -> Result<Block> {
        let mut open = Statement::new();

        add_annotations(&self.annotations, &mut open)?;

        if !self.modifiers.is_empty() {
            open.push("$L ", stmt!(literal self.modifiers.format()?));
        }

        match self.returns {
            None => open.push("void ", vec![]),
            Some(ref returns) => open.push("$T ", stmt![type_spec returns]),
        }

        open.push("$L(", stmt!(literal self.name.clone()));

        if !self.arguments.is_empty() {
            let mut arguments: Statement = Statement::new();

            for a in &self.arguments {
                arguments.push_statement(a.as_statement()?);
            }

            let arguments: Statement = arguments.join(", ");
            open.push_statement(arguments);
        }

        open.push(")", vec![]);

        let mut block = Block::new();
        block.open(open);
        block.extend(&self.sections);

        Ok(block)
    }
}

impl Imports for MethodSpec {
    fn imports<I>(&self, receiver: &mut I)
        where I: ImportReceiver
    {
        if let Some(ref type_) = self.returns {
            type_.imports(receiver);
        }

        receiver.import_all(&self.arguments);
        self.sections.imports(receiver);
    }
}

#[derive(Debug, Clone)]
pub struct InterfaceSpec {
    pub modifiers: Modifiers,
    pub name: String,
    pub annotations: Vec<AnnotationSpec>,
    pub elements: Vec<ElementSpec>,
}

impl InterfaceSpec {
    pub fn new(modifiers: Modifiers, name: &str) -> InterfaceSpec {
        InterfaceSpec {
            modifiers: modifiers,
            name: name.to_owned(),
            annotations: Vec::new(),
            elements: Vec::new(),
        }
    }

    pub fn push_annotation(&mut self, annotation: &AnnotationSpec) {
        self.annotations.push(annotation.clone());
    }

    pub fn push_class(&mut self, class: &ClassSpec) {
        self.elements.push(ElementSpec::Class(class.clone()))
    }

    pub fn push_interface(&mut self, interface: &InterfaceSpec) {
        self.elements.push(ElementSpec::Interface(interface.clone()))
    }

    pub fn imports<I>(&self, receiver: &mut I) where I: ImportReceiver {
        receiver.import_all(&self.annotations);
        receiver.import_all(&self.elements);
    }

    pub fn as_block(&self) -> Result<Block> {
        let mut open = Statement::new();

        add_annotations(&self.annotations, &mut open)?;

        if !self.modifiers.is_empty() {
            open.push("$L ", stmt!(literal self.modifiers.format()?));
        }

        open.push("interface $L", stmt!(literal self.name.clone()));

        let mut block = Block::new();
        block.open(open);

        let mut first: bool = true;

        for element in &self.elements {
            if first {
                first = false;
            } else {
                block.push_spacing();
            }

            block.push_block(&element.as_block()?);
        }

        Ok(block)
    }
}

#[derive(Debug, Clone)]
pub struct ClassSpec {
    pub modifiers: Modifiers,
    pub name: String,
    pub annotations: Vec<AnnotationSpec>,
    pub fields: Vec<FieldSpec>,
    pub constructors: Vec<ConstructorSpec>,
    pub methods: Vec<MethodSpec>,
    pub elements: Vec<ElementSpec>,
}

impl ClassSpec {
    pub fn new(modifiers: Modifiers, name: &str) -> ClassSpec {
        ClassSpec {
            modifiers: modifiers,
            name: name.to_owned(),
            annotations: Vec::new(),
            fields: Vec::new(),
            constructors: Vec::new(),
            methods: Vec::new(),
            elements: Vec::new(),
        }
    }

    pub fn push_annotation(&mut self, annotation: &AnnotationSpec) {
        self.annotations.push(annotation.clone());
    }

    pub fn push_field(&mut self, field: &FieldSpec) {
        self.fields.push(field.clone());
    }

    pub fn push_constructor(&mut self, constructor: &ConstructorSpec) {
        self.constructors.push(constructor.clone());
    }

    pub fn push_method(&mut self, method: &MethodSpec) {
        self.methods.push(method.clone());
    }

    pub fn push_class(&mut self, class: &ClassSpec) {
        self.elements.push(ElementSpec::Class(class.clone()))
    }

    pub fn push_interface(&mut self, interface: &InterfaceSpec) {
        self.elements.push(ElementSpec::Interface(interface.clone()))
    }

    pub fn imports<I>(&self, receiver: &mut I)
        where I: ImportReceiver
    {
        receiver.import_all(&self.annotations);
        receiver.import_all(&self.constructors);
        receiver.import_all(&self.methods);
    }

    pub fn as_block(&self) -> Result<Block> {
        let mut open = Statement::new();

        add_annotations(&self.annotations, &mut open)?;

        if !self.modifiers.is_empty() {
            open.push("$L ", stmt!(literal self.modifiers.format()?));
        }

        open.push("class $L", stmt!(literal self.name.clone()));

        let mut block = Block::new();
        block.open(open);

        for field in &self.fields {
            block.push_statement(&field.as_statement()?);
        }

        /// TODO: figure out a better way...
        let mut first = self.fields.is_empty();

        for constructor in &self.constructors {
            if first {
                first = false;
            } else {
                block.push_spacing();
            }

            block.push_block(&constructor.as_block(&self.name)?);
        }

        for method in &self.methods {
            if first {
                first = false;
            } else {
                block.push_spacing();
            }

            block.push_block(&method.as_block()?);
        }

        for element in &self.elements {
            if first {
                first = false;
            } else {
                block.push_spacing();
            }

            block.push_block(&element.as_block()?);
        }

        Ok(block)
    }
}

#[derive(Debug, Clone)]
pub enum ElementSpec {
    Class(ClassSpec),
    Interface(InterfaceSpec),
}

impl ElementSpec {
    pub fn as_block(&self) -> Result<Block> {
        match *self {
            ElementSpec::Class(ref class) => class.as_block(),
            ElementSpec::Interface(ref interface) => interface.as_block(),
        }
    }
}

impl Imports for ElementSpec {
    fn imports<I>(&self, receiver: &mut I)
        where I: ImportReceiver
    {
        match *self {
            ElementSpec::Class(ref class) => class.imports(receiver),
            ElementSpec::Interface(ref interface) => interface.imports(receiver),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileSpec {
    pub package: String,
    pub elements: Vec<ElementSpec>,
}

impl FileSpec {
    pub fn new(package: &str) -> FileSpec {
        FileSpec {
            package: package.to_owned(),
            elements: Vec::new(),
        }
    }

    pub fn push_class(&mut self, class: &ClassSpec) {
        self.elements.push(ElementSpec::Class(class.clone()))
    }

    pub fn push_interface(&mut self, interface: &InterfaceSpec) {
        self.elements.push(ElementSpec::Interface(interface.clone()))
    }

    pub fn format(&self) -> Result<String> {
        let mut sections = Sections::new();

        sections.push_statement(&stmt!("package $L", literal self.package.clone()));
        sections.push_spacing();

        let mut receiver: BTreeSet<Type> = BTreeSet::new();

        for element in &self.elements {
            match *element {
                ElementSpec::Class(ref class_spec) => {
                    class_spec.imports(&mut receiver);
                }
                ElementSpec::Interface(ref interface_spec) => {
                    interface_spec.imports(&mut receiver);
                }
            }
        }

        let imports: Vec<Type> = receiver.into_iter()
            .filter(|t| t.package != "java.lang")
            .filter(|t| t.package != self.package)
            .collect();

        if !imports.is_empty() {
            for t in imports {
                sections.push_statement(&stmt!("import $L.$L", literal t.package.clone(), literal t.name.clone()));
            }

            sections.push_spacing();
        }

        for element in &self.elements {
            sections.push_block(&element.as_block()?);
        }

        let mut out = String::new();

        for line in sections.format("", "  ")? {
            out.push_str(&line);
            out.push('\n');
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_Test_java() {
        let string_type = Type::new("java.lang", "String");
        let list_type = Type::new("java.util", "List");
        let json_creator_type = Type::new("com.fasterxml.jackson.annotation", "JsonCreator");
        let list_of_strings = list_type.with_arguments(vec![&string_type]);

        let values_field = FieldSpec::new(mods![Modifier::Private, Modifier::Final],
                                          &list_of_strings,
                                          "values");

        let mut values_argument =
            ArgumentSpec::new(mods![Modifier::Final], &list_of_strings, "values");

        let mut constructor = ConstructorSpec::new(mods![Modifier::Public]);
        constructor.push_annotation(&AnnotationSpec::new(json_creator_type));
        constructor.push_argument(&values_argument);
        constructor.push_statement(&stmt!("this.values = $N", name values_argument));

        let mut values_getter = MethodSpec::new(mods![Modifier::Public], "getValues");
        values_getter.returns(&list_of_strings);
        values_getter.push_statement(&stmt!("return this.$N", name values_field));

        let mut class = ClassSpec::new(mods![Modifier::Public], "Test");
        class.push_field(&values_field);
        class.push_constructor(&constructor);
        class.push_method(&values_getter);

        let mut file = FileSpec::new("se.tedro");
        file.push_class(&class);

        let result = file.format().unwrap();

        let reference = ::std::str::from_utf8(include_bytes!("tests/Test.java")).unwrap();
        assert_eq!(reference, result);
    }
}
