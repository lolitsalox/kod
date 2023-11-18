use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    #[allow(dead_code)]
    pub token_type: TokenType,
    #[allow(dead_code)]
    pub keyword_type: KeywordType,
    #[allow(dead_code)]
    pub value: String,
    #[allow(dead_code)]
    pub int_value: i64,
    #[allow(dead_code)]
    pub float_value: f64,
    #[allow(dead_code)]
    pub line: usize,
    #[allow(dead_code)]
    pub column: usize,
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum TokenType {
    Unknown,
    EndOfFile,

    ADD,                //  +
    SUB,                //  -
    DIV,                //  /
    MUL,                //  *
    MOD,                //  %
    POW,                //  **

    AddEq,                //  +=
    SubEq,                //  -=
    DivEq,                //  /=
    MulEq,                //  *=
    ModEq,                //  %=

    AND,                //  &
    OR,                 //  |
    HAT,                //  ^
    SHL,                //  <<
    SHR,                //  >>
    NOT,                //  ~

    BoolNot,           //  !
    BoolEq,            //  ==
    BoolNe,            //  !=
    BoolLt,            //  <
    BoolGt,            //  >
    BoolLte,           //  <=
    BoolGte,           //  >=
    BoolAnd,           //  &&
    BoolOr,            //  ||
    ID,                 //  main x y foo
    KEYWORD,            //  NOT USED

    STRING,             //  "Hello world"
    INT,                //  5 6 456
    FLOAT,              //  6.9 7893.6   

    LPAREN,             //  (   
    RPAREN,             //  )              
    LBRACKET,           //  [              
    RBRACKET,           //  ]              
    LBRACE,             //  {          
    RBRACE,             //  }

    EQUALS,             //  =   
    COMMA,              //  ,  
    DOT,                //  .  
    COLON,              //  :  
    NAMESPACE,          //  ::  
    SEMI,               //  ;   
    QUESTION,           //  ?   
    AT,                 //  @
    HASH,               //  #
    LineComment,       // //
    MultilineCommentStart,     // /*
    MultilineCommentEnd,       // */
    POINTER,            //  ->
    ARROW,              //  =>
    BACKSLASH,          // 

    NewLine,           //  \n
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum KeywordType {
    Unknown,
    Null,
    If,
    Else,
    While,
    For,
    Return,
    Import,
    As,
    From,
    Break,
    Continue,
}

pub fn get_symbols() -> HashMap<&'static str, TokenType> {
    HashMap::from([
        ("::", TokenType::NAMESPACE),
        ("&&", TokenType::BoolAnd),
        ("||", TokenType::BoolOr),
        ("**", TokenType::POW),
        ("//", TokenType::LineComment),
        ("/*", TokenType::MultilineCommentStart),
        ("*/", TokenType::MultilineCommentEnd),
        ("->", TokenType::POINTER),
        ("!=", TokenType::BoolNe),
        ("==", TokenType::BoolEq),
        ("=>", TokenType::ARROW),
        ("<<", TokenType::SHL),
        (">>", TokenType::SHR),
        ("<=", TokenType::BoolLte),
        (">=", TokenType::BoolGte),
        ("(", TokenType::LPAREN),
        (")", TokenType::RPAREN),
        ("[", TokenType::LBRACKET),
        ("]", TokenType::RBRACKET),
        ("{", TokenType::LBRACE),
        ("}", TokenType::RBRACE),
        ("=", TokenType::EQUALS),
        (",", TokenType::COMMA),
        (":", TokenType::COLON),
        (";", TokenType::SEMI),
        ("?", TokenType::QUESTION),
        ("%", TokenType::MOD),
        ("\\", TokenType::BACKSLASH),
        ("#", TokenType::HASH),
        ("@", TokenType::AT),
        ("+", TokenType::ADD),
        ("-", TokenType::SUB),
        ("/", TokenType::DIV),
        ("*", TokenType::MUL),
        ("&", TokenType::AND),
        ("|", TokenType::OR),
        ("^", TokenType::HAT),
        ("<", TokenType::BoolLt),
        (">", TokenType::BoolGt),
        ("~", TokenType::NOT),
        ("!", TokenType::BoolNot),
        (".", TokenType::DOT),
        ("+=", TokenType::AddEq),
        ("-=", TokenType::SubEq),
        ("/=", TokenType::DivEq),
        ("*=", TokenType::MulEq),
        ("%=", TokenType::ModEq),
    ])
}

impl Token {
    pub fn new(type_: TokenType, value: String, line: usize, column: usize) -> Self {
        Token {
            token_type: type_,
            keyword_type: KeywordType::Unknown,
            value,
            int_value: 0,
            float_value: 0.0,
            line,
            column,
        }
    }

    // pub fn to_string(&self) -> String {
    //     // Implement the to_string method logic
    //     // You can use the format! macro or other string formatting options in Rust
    //     format!("Token: {:?}, Value: {:}, Line: {}, Column: {}", self.token_type, self.value, self.line, self.column)
    // }
}

pub fn eof() -> Token {
    Token {
        token_type: TokenType::EndOfFile,
        keyword_type: KeywordType::Unknown,
        value: String::new(),
        int_value: 0,
        float_value: 0.0,
        line: 0,
        column: 0,
    }
}

