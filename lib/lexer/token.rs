use core::RpNumber;
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'input> {
    Identifier(Cow<'input, str>),
    TypeIdentifier(Cow<'input, str>),
    PackageDocComment(Vec<Cow<'input, str>>),
    DocComment(Vec<Cow<'input, str>>),
    Number(RpNumber),
    LeftCurly,
    RightCurly,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    SemiColon,
    Colon,
    Equal,
    Comma,
    Dot,
    Scope,
    QuestionMark,
    Hash,
    Bang,
    RightArrow,
    CodeOpen,
    CodeClose,
    CodeContent(Cow<'input, str>),
    String(String),
    // identifier-style keywords
    InterfaceKeyword,
    TypeKeyword,
    EnumKeyword,
    TupleKeyword,
    ServiceKeyword,
    UseKeyword,
    AsKeyword,
    AnyKeyword,
    FloatKeyword,
    DoubleKeyword,
    Signed32,
    Signed64,
    Unsigned32,
    Unsigned64,
    BooleanKeyword,
    StringKeyword,
    DateTimeKeyword,
    BytesKeyword,
    StreamKeyword,
}

impl<'input> Token<'input> {
    /// Get the keywords-safe variant of the given keyword.
    pub fn keyword_safe(&self) -> Option<&'static str> {
        let out = match *self {
            Token::AnyKeyword => "_any",
            Token::InterfaceKeyword => "_interface",
            Token::TypeKeyword => "_type",
            Token::EnumKeyword => "_enum",
            Token::TupleKeyword => "_tuple",
            Token::ServiceKeyword => "_service",
            Token::UseKeyword => "_use",
            Token::AsKeyword => "_as",
            Token::FloatKeyword => "_float",
            Token::DoubleKeyword => "_double",
            Token::Signed32 => "_i32",
            Token::Signed64 => "_i64",
            Token::Unsigned32 => "_u32",
            Token::Unsigned64 => "_u64",
            Token::BooleanKeyword => "_boolean",
            Token::StringKeyword => "_string",
            Token::DateTimeKeyword => "_datetime",
            Token::BytesKeyword => "_bytes",
            Token::StreamKeyword => "_stream",
            _ => return None,
        };

        Some(out)
    }
}
