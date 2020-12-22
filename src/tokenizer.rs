use regex_lexer::{Lexer, LexerBuilder};

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Data(DataToken),
    Keyword(KeywordToken),
    Text(&'a str),
    Symbol(SymbolToken),
    Label(&'a str),
    NewLine,
}

#[derive(Debug, PartialEq, Eq)]
pub enum KeywordToken {
    Exit,
}

impl From<DataToken> for Token<'_> {
    fn from(data: DataToken) -> Self {
        Token::Data(data)
    }
}

impl From<SymbolToken> for Token<'_> {
    fn from(bracket: SymbolToken) -> Self {
        Token::Symbol(bracket)
    }
}

impl From<KeywordToken> for Token<'_> {
    fn from(key: KeywordToken) -> Self {
        Token::Keyword(key)
    }
}

#[derive(Debug, PartialEq)]
pub enum DataToken {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Character(char),
    String(String),
    None,
}

impl std::cmp::Eq for DataToken {}

#[derive(Debug, PartialEq, Eq)]
pub enum SymbolToken {
    RoundOpen,
    RoundClose,
    Semicolon
}

pub fn build_lexer<'t>() -> Result<Lexer<'t, Token<'t>>, regex::Error> {
    LexerBuilder::new()
        .token(r"-?[0-9]+", |tok| {
            Some(DataToken::Integer(tok.parse().unwrap()).into())
        })
        .token(r"-?[0-9]+\.[0-9]+", |tok| {
            Some(DataToken::Float(tok.parse().unwrap()).into())
        })
        .token(r"'.'", |tok| {
            Some(DataToken::Character(tok[1..tok.len() - 1].parse().unwrap()).into())
        })
        //NOTE This regex might change
        .token(r"(_|[a-zA-Z])[a-zA-Z_0-9]*", |tok| {
            Some(Token::Text(tok))
        })
        .token(r"(true|false)", |tok| {
            Some(DataToken::Bool(tok.parse().unwrap()).into())
        })
        .token(r"^\w*:\s*\n", |text| { 
            let trimmed = text.trim();
            Some(Token::Label(&trimmed[..trimmed.len()-1]))
        })
        .token("none", |_| Some(DataToken::None.into()))
        .token(r"\s", |_| None)
        .token("\n", |_| Some(Token::NewLine))
        .token(";", |_| Some(SymbolToken::Semicolon.into()))
        .token(r"#.*?\n", |_| None)
        .token(r"\(", |_| Some(SymbolToken::RoundOpen.into()))
        .token(r"\)", |_| Some(SymbolToken::RoundClose.into()))
        .token("exit", |_| Some(KeywordToken::Exit.into()))
        .token("\".*?\"", |tok| {
            let s = Some(DataToken::String(tok[1..tok.len() - 1].replace("\\n", "\n")).into());
            s
        })
        //.token(r"#.*", |_| Some(Token::Newline))
        //.token("\n", |_| Some(Token::Newline))
        .build()
}
