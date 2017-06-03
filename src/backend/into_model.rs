//! Implementations for converting asts into models.
use parser::ast;
use std::collections::btree_map;
use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;
use super::errors::*;
use super::merge::Merge;
use super::models::*;
use super::options::Options;

fn code(pos: &Pos, ast_pos: ast::Pos, context: String, lines: Vec<String>) -> RpToken<Code> {
    let pos = (pos.0.clone(), ast_pos.0, ast_pos.1);

    let code = Code {
        context: context,
        lines: lines,
    };

    RpToken::new(code, pos)
}

type Fields = Vec<RpToken<Field>>;
type Codes = Vec<RpToken<Code>>;
type OptionVec = Vec<RpToken<OptionDecl>>;

fn members_into_model(pos: &Pos,
                      members: Vec<ast::AstToken<ast::Member>>)
                      -> Result<(Fields, Codes, OptionVec, MatchDecl)> {
    let mut fields: Vec<RpToken<Field>> = Vec::new();
    let mut codes = Vec::new();
    let mut options: Vec<RpToken<OptionDecl>> = Vec::new();
    let mut match_decl = MatchDecl::new();

    for member in members {
        let pos = (pos.0.to_owned(), member.pos.0, member.pos.1);

        match member.inner {
            ast::Member::Field(field) => {
                let field = field.into_model(&pos)?;

                if let Some(other) = fields.iter().find(|f| f.name == field.name) {
                    return Err(Error::field_conflict(field.name.clone(), pos, other.pos.clone()));
                }

                fields.push(RpToken::new(field, pos));
            }
            ast::Member::Code(context, lines) => {
                codes.push(code(&pos, member.pos, context, lines));
            }
            ast::Member::Option(option) => {
                options.push(option.into_model(&pos)?);
            }
            ast::Member::Match(m) => {
                for member in m.members {
                    match_decl.push(member.into_model(&pos)?)?;
                }
            }
        }
    }

    Ok((fields, codes, options, match_decl))
}

struct OrdinalGenerator {
    next_ordinal: u32,
    ordinals: HashSet<u32>,
}

impl OrdinalGenerator {
    pub fn new() -> OrdinalGenerator {
        OrdinalGenerator {
            next_ordinal: 0,
            ordinals: HashSet::new(),
        }
    }

    pub fn next(&mut self, ordinal: &Option<ast::AstToken<ast::Value>>, pos: &Pos) -> Result<u32> {
        if let Some(ref ordinal) = *ordinal {
            let pos = (pos.0.to_owned(), ordinal.pos.0, ordinal.pos.1);

            if let ast::Value::Number(ref number) = ordinal.inner {
                let n: u32 = number.floor() as u32;

                if self.ordinals.contains(&n) {
                    return Err(Error::pos("duplicate ordinal".to_owned(), pos));
                }

                self.ordinals.insert(n);

                self.next_ordinal = n + 1;

                return Ok(n);
            }

            return Err(Error::pos("must be a number".to_owned(), pos));
        }

        let o = self.next_ordinal;

        self.next_ordinal += 1;

        if self.ordinals.contains(&o) {
            return Err(Error::pos(format!("generated ordinal {} conflicts with existing", o),
                                  pos.clone()));
        }

        Ok(o)
    }
}

/// Adds the into_model() method for all types that supports conversion into models.
pub trait IntoModel {
    type Output;

    /// Convert the current type to a model.
    fn into_model(self, pos: &Pos) -> Result<Self::Output>;
}

impl<T> IntoModel for ast::AstToken<T>
    where T: IntoModel
{
    type Output = RpToken<T::Output>;

    fn into_model(self, pos: &Pos) -> Result<Self::Output> {
        let pos = (pos.0.clone(), self.pos.0, self.pos.1);
        let out = self.inner.into_model(&pos)?;
        Ok(RpToken::new(out, pos))
    }
}

impl<T> IntoModel for Vec<T>
    where T: IntoModel
{
    type Output = Vec<T::Output>;

    fn into_model(self, pos: &Pos) -> Result<Self::Output> {
        let mut out = Vec::new();

        for v in self {
            out.push(v.into_model(pos)?);
        }

        Ok(out)
    }
}

impl<T> IntoModel for Option<T>
    where T: IntoModel
{
    type Output = Option<T::Output>;

    fn into_model(self, pos: &Pos) -> Result<Self::Output> {
        if let Some(value) = self {
            return Ok(Some(value.into_model(pos)?));
        }

        Ok(None)
    }
}

impl IntoModel for ast::InterfaceBody {
    type Output = Rc<InterfaceBody>;

    fn into_model(self, pos: &Pos) -> Result<Rc<InterfaceBody>> {
        let (fields, codes, options, match_decl) = members_into_model(&pos, self.members)?;

        let mut sub_types: BTreeMap<String, RpToken<Rc<SubType>>> = BTreeMap::new();

        for sub_type in self.sub_types.into_model(pos)? {
            // key has to be owned by entry
            let key = sub_type.name.clone();

            match sub_types.entry(key) {
                btree_map::Entry::Occupied(entry) => {
                    entry.into_mut().merge(sub_type)?;
                }
                btree_map::Entry::Vacant(entry) => {
                    entry.insert(sub_type);
                }
            }
        }

        let _options = Options::new(&pos, options);

        let interface_body = InterfaceBody {
            name: self.name,
            fields: fields,
            codes: codes,
            match_decl: match_decl,
            sub_types: sub_types,
        };

        Ok(Rc::new(interface_body))
    }
}

impl IntoModel for ast::EnumBody {
    type Output = Rc<EnumBody>;

    fn into_model(self, pos: &Pos) -> Result<Rc<EnumBody>> {
        let mut values: Vec<RpToken<Rc<EnumValue>>> = Vec::new();

        let mut ordinals = OrdinalGenerator::new();

        let (fields, codes, options, match_decl) = members_into_model(pos, self.members)?;

        for value in self.values {
            let ordinal = ordinals.next(&value.ordinal, pos)
                .map_err(|e| Error::pos(e.description().into(), pos.clone()))?;

            let value = RpToken::new((value.inner, ordinal).into_model(pos)?,
                                     (pos.0.clone(), value.pos.0, value.pos.1));

            if fields.len() != value.arguments.len() {
                return Err(Error::pos(format!("expected {} arguments", fields.len()),
                                      value.pos.clone()));
            }

            if let Some(other) = values.iter().find(|v| *v.name == *value.name) {
                return Err(ErrorKind::EnumValueConflict(other.name.pos.clone(),
                                                        value.name.pos.clone())
                    .into());
            }

            /// need to tack on an ordinal value.
            values.push(value);
        }

        let options = Options::new(pos, options);

        let serialized_as: Option<RpToken<String>> = options.find_one_identifier("serialized_as")?
            .to_owned();

        let serialized_as_name = options.find_one_boolean("serialized_as_name")?
            .to_owned()
            .map(|t| t.inner)
            .unwrap_or(false);

        let en = EnumBody {
            name: self.name,
            values: values,
            fields: fields,
            codes: codes,
            match_decl: match_decl,
            serialized_as: serialized_as,
            serialized_as_name: serialized_as_name,
        };

        Ok(Rc::new(en))
    }
}

/// enum value with assigned ordinal
impl IntoModel for (ast::EnumValue, u32) {
    type Output = Rc<EnumValue>;

    fn into_model(self, pos: &Pos) -> Result<Self::Output> {
        let value = self.0;
        let ordinal = self.1;

        let value = EnumValue {
            name: value.name.into_model(pos)?,
            arguments: value.arguments.into_model(pos)?,
            ordinal: ordinal,
        };

        Ok(Rc::new(value))
    }
}

impl IntoModel for ast::TypeBody {
    type Output = Rc<TypeBody>;

    fn into_model(self, pos: &Pos) -> Result<Rc<TypeBody>> {
        let (fields, codes, options, match_decl) = members_into_model(&pos, self.members)?;

        let options = Options::new(&pos, options);

        let reserved: HashSet<RpToken<String>> =
            options.find_all_identifiers("reserved")?.into_iter().collect();

        let type_body = TypeBody {
            name: self.name,
            fields: fields,
            codes: codes,
            match_decl: match_decl,
            reserved: reserved,
        };

        Ok(Rc::new(type_body))
    }
}

impl IntoModel for ast::SubType {
    type Output = Rc<SubType>;

    fn into_model(self, pos: &Pos) -> Result<Rc<SubType>> {
        let mut fields: Vec<RpToken<Field>> = Vec::new();
        let mut codes = Vec::new();
        let mut options = Vec::new();
        let mut match_decl = MatchDecl::new();

        for member in self.members {
            let pos = (pos.0.to_owned(), member.pos.0, member.pos.1);

            match member.inner {
                ast::Member::Field(field) => {
                    let field = field.into_model(&pos)?;

                    if let Some(other) = fields.iter().find(|f| f.name == field.name) {
                        return Err(Error::field_conflict(field.name.clone(),
                                                         pos,
                                                         other.pos.clone()));
                    }

                    fields.push(RpToken::new(field, pos));
                }
                ast::Member::Code(context, lines) => {
                    codes.push(code(&pos, member.pos, context, lines));
                }
                ast::Member::Option(option) => {
                    options.push(option.into_model(&pos)?);
                }
                ast::Member::Match(m) => {
                    for member in m.members {
                        match_decl.push(member.into_model(&pos)?)?;
                    }
                }
            }
        }

        let options = Options::new(&pos, options);

        let names = options.find_all_strings("name")?;

        let sub_type = SubType {
            name: self.name,
            fields: fields,
            codes: codes,
            names: names,
        };

        Ok(Rc::new(sub_type))
    }
}

impl IntoModel for ast::TupleBody {
    type Output = Rc<TupleBody>;

    fn into_model(self, pos: &Pos) -> Result<Rc<TupleBody>> {
        let (fields, codes, options, match_decl) = members_into_model(&pos, self.members)?;

        let _options = Options::new(&pos, options);

        let tuple_body = TupleBody {
            name: self.name,
            fields: fields,
            codes: codes,
            match_decl: match_decl,
        };

        Ok(Rc::new(tuple_body))
    }
}

impl IntoModel for ast::Decl {
    type Output = Decl;

    fn into_model(self, pos: &Pos) -> Result<Decl> {
        let decl = match self {
            ast::Decl::Type(body) => Decl::Type(body.into_model(pos)?),
            ast::Decl::Interface(body) => Decl::Interface(body.into_model(pos)?),
            ast::Decl::Enum(body) => Decl::Enum(body.into_model(pos)?),
            ast::Decl::Tuple(body) => Decl::Tuple(body.into_model(pos)?),
        };

        Ok(decl)
    }
}

impl IntoModel for ast::OptionDecl {
    type Output = OptionDecl;

    fn into_model(self, pos: &Pos) -> Result<OptionDecl> {
        let decl = OptionDecl {
            name: self.name,
            values: self.values.into_model(pos)?,
        };

        Ok(decl)
    }
}

impl IntoModel for ast::Field {
    type Output = Field;

    fn into_model(self, pos: &Pos) -> Result<Field> {
        let field_as = self.field_as.into_model(pos)?;

        let field_as = if let Some(field_as) = field_as {
            if let Value::String(name) = field_as.inner {
                Some(RpToken::new(name, field_as.pos.clone()))
            } else {
                return Err(Error::pos("must be a string".to_owned(), field_as.pos));
            }
        } else {
            None
        };

        let field = Field {
            modifier: self.modifier,
            name: self.name,
            ty: self.ty,
            field_as: field_as,
        };

        Ok(field)
    }
}

impl IntoModel for ast::Value {
    type Output = Value;

    fn into_model(self, pos: &Pos) -> Result<Value> {
        let value = match self {
            ast::Value::String(string) => Value::String(string),
            ast::Value::Number(number) => Value::Number(number),
            ast::Value::Boolean(boolean) => Value::Boolean(boolean),
            ast::Value::Identifier(identifier) => Value::Identifier(identifier),
            ast::Value::Type(ty) => Value::Type(ty),
            ast::Value::Instance(instance) => Value::Instance(instance.into_model(pos)?),
            ast::Value::Constant(constant) => Value::Constant(constant.with_prefix(pos.0.clone())),
            ast::Value::Array(values) => Value::Array(values.into_model(pos)?),
        };

        Ok(value)
    }
}

impl IntoModel for ast::FieldInit {
    type Output = FieldInit;

    fn into_model(self, pos: &Pos) -> Result<FieldInit> {
        let field_init = FieldInit {
            name: self.name.into_model(pos)?,
            value: self.value.into_model(pos)?,
        };

        Ok(field_init)
    }
}

impl IntoModel for ast::Instance {
    type Output = Instance;

    fn into_model(self, pos: &Pos) -> Result<Instance> {
        let instance = Instance {
            ty: self.ty,
            arguments: self.arguments.into_model(pos)?,
        };

        Ok(instance)
    }
}

impl IntoModel for String {
    type Output = String;

    fn into_model(self, _pos: &Pos) -> Result<String> {
        Ok(self)
    }
}

impl IntoModel for ast::MatchVariable {
    type Output = MatchVariable;

    fn into_model(self, pos: &Pos) -> Result<MatchVariable> {
        let match_variable = MatchVariable {
            name: self.name.into_model(pos)?,
            ty: self.ty,
        };

        Ok(match_variable)
    }
}

impl IntoModel for ast::MatchCondition {
    type Output = MatchCondition;

    fn into_model(self, pos: &Pos) -> Result<MatchCondition> {
        let match_condition = match self {
            ast::MatchCondition::Value(value) => MatchCondition::Value(value.into_model(pos)?),
            ast::MatchCondition::Type(ty) => MatchCondition::Type(ty.into_model(pos)?),
        };

        Ok(match_condition)
    }
}

impl IntoModel for ast::MatchMember {
    type Output = MatchMember;

    fn into_model(self, pos: &Pos) -> Result<MatchMember> {
        let member = MatchMember {
            condition: self.condition.into_model(pos)?,
            value: self.value.into_model(pos)?,
        };

        Ok(member)
    }
}
