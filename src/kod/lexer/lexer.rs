use std::collections::HashMap;

use super::token::*;

#[derive(Debug)]
pub struct Lexer {

    pub filename: String,
    pub contents: String,
    pub position: usize,
    pub line: usize,
    pub column: usize,
    pub ch: char,

}

#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter(String),
    UnterminatedString(String),
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerError::UnexpectedCharacter(s) => write!(f, "Unexpected character: {}", s),
            LexerError::UnterminatedString(s) => write!(f, "Unterminated string: {}", s),
        }
    }
}

impl std::error::Error for LexerError {}

impl Lexer {        
    pub fn new(filename: &str, contents: &str) -> Lexer {
        Lexer {
            filename: filename.to_string(),
            contents: contents.to_string(),
            position: 0,
            line: 1,
            column: 1,
            ch: contents.chars().nth(0).unwrap_or_default(),
        }
    }

    // A function that returns the next token (in case of error it returns an error)
    pub fn next(&mut self) -> Result<Token, LexerError> {
        loop {
            self.skip_whitespace();
            self.skip_comments();

            if !self.can_advance() {
                return Ok(eof());
            }

            if self.ch == '"' || self.ch == '\'' {
                return self.collect_string();
            }
        
            if self.ch.is_digit(10) || (self.ch == '.' && self.peek_char(1).is_digit(10)) {
                return self.collect_number();
            }

            if self.ch.is_alphabetic() || self.ch == '_' {
                return self.collect_identifier();
            }

            if self.is_symbol(self.ch) {
                self.skip_comments();
                if self.is_start_of_comment() || !self.is_symbol(self.ch) { continue; }
                return self.collect_symbol();
            }

            // check for a new line
            if self.ch == '\n' || (self.ch == '\r' && self.peek_char(1) == '\n') {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenType::NewLine, String::new(), self.line, self.column));
            }

            return Err(LexerError::UnexpectedCharacter(format!("{}:{}:{}: {}", self.filename, self.line, self.column, self.ch)));
        }
    }

    // A function that peeks the next token
    pub fn peek(&mut self) -> Result<Token, LexerError> {
        let old_position = self.position;
        let old_line = self.line;
        let old_column = self.column;
        let old_ch = self.ch;

        let token = self.next();

        self.position = old_position;
        self.line = old_line;
        self.column = old_column;
        self.ch = old_ch;

        token
    }

    fn skip_whitespace(&mut self) {
        while self.can_advance() && (self.ch == ' ' || self.ch == '\t') {
            self.advance();
        }
    }

    fn skip_comments(&mut self) {
        if !self.can_advance() { return }

        // Check if the next character is a /
        if self.ch != '/' { return }
    
        // Check if the next character is a / or *
        let peek = self.peek_char(1);
        match char::from_u32(peek as u32).unwrap() {
            '/' => {
                self.skip_until_char('\n');
                self.advance();
            }
            '*' => {
                self.skip_until_string("*/");
            }
            _ => return
            
        }
    }

    fn skip_until_char(&mut self, ch: char) {
        while self.can_advance() && self.ch != ch {
            self.advance();
        }
    }

    fn skip_until_string(&mut self, string: &str) {
        while self.can_advance() {
            let remaining_content = &self.contents[self.position..];
            if remaining_content.starts_with(string) {
                break;
            }
            self.advance();
        }
    }

    fn advance(&mut self) {
        if !self.can_advance() {
            return;
        }

        if self.ch == '\n' {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }

        self.position += 1;
        self.ch = self.contents.chars().nth(self.position).unwrap_or_default();
    }

    fn can_advance(&mut self) -> bool {
        self.ch != '\0'
    }

    fn is_start_of_comment(&mut self) -> bool {
        return self.ch == '/' && (self.peek_char(1) == '/' || self.peek_char(1) == '*');
    }

    fn peek_char(&mut self, offset: usize) -> char {
        self.contents.chars().nth(self.position + offset).unwrap_or_default()
    }

    fn collect_identifier(&mut self) -> Result<Token, LexerError> {
        let mut identifier = String::new();
        let mut ttype = TokenType::ID;
        let this_line = self.line;
        let this_column = self.column;

        while self.can_advance() && (self.ch.is_alphanumeric() || self.ch == '_') {
            identifier.push(self.ch);
            self.advance();
        }

        let ktype = self.find_keyword_type(&identifier);

        if ktype != KeywordType::Unknown {
            ttype = TokenType::KEYWORD;
        }

        let mut tok = Token::new(ttype, identifier, this_line, this_column);
        tok.keyword_type = ktype;
        Ok(tok)
    }

    fn collect_number(&mut self) -> Result<Token, LexerError> {
        let mut number = String::new();
        let this_line = self.line;
        let this_column = self.column;
    
        let mut dot = false;
        let mut is_hex = false;
        let mut is_bin = false;
        let mut is_oct = false;
    
        let mut ttype = TokenType::INT;
    
        if self.ch == '0' {
            self.advance();
            if self.can_advance() {
                match self.ch {
                    'x' => {
                        is_hex = true;
                        self.advance();
                    }
                    'b' => {
                        is_bin = true;
                        self.advance();
                    }
                    'o' => {
                        is_oct = true;
                        self.advance();
                    }
                    _ => {
                        number.push('0');
                    }
                }
            } else {
                number.push('0');
            }
        } else if self.ch == '.' {
            number.push('0');
        }
    
        while self.can_advance() && ((is_hex && self.ch.is_digit(16)) || 
                                      (is_bin && (self.ch == '0' || self.ch == '1')) ||
                                      (is_oct && self.ch.is_digit(8)) ||
                                      (!is_hex && !is_bin && !is_oct && (self.ch.is_digit(10) || self.ch == '.'))) {
            if self.ch == '.' {
                if dot {
                    break;
                }
                dot = true;
            }
            number.push(self.ch);
            self.advance();
        }
    
        if dot {
            while self.can_advance() && self.ch.is_digit(10) {
                number.push(self.ch);
                self.advance();
            }
    
            ttype = TokenType::FLOAT;
        }
    
        let mut tok = Token::new(ttype, number.clone(), this_line, this_column);
    
        if is_hex {
            tok.int_value = i64::from_str_radix(&number, 16).unwrap();
        } else if is_bin {
            tok.int_value = i64::from_str_radix(&number, 2).unwrap();
        } else if is_oct {
            tok.int_value = i64::from_str_radix(&number, 8).unwrap();
        } else if dot {
            tok.float_value = number.parse().unwrap();
        } else {
            tok.int_value = number.parse().unwrap();
        }
    
        Ok(tok)
    }

    fn collect_string(&mut self) -> Result<Token, LexerError> {
        let mut string = String::new();
        let ttype = TokenType::STRING;
        let single_quote = self.ch == '\'';
        let this_line = self.line;
        let this_column = self.column;
    
        // Eating first quote/s
        self.advance();
    
        while self.can_advance()
            && ((self.ch != '\'' && single_quote) || (self.ch != '"' && !single_quote))
        {
            if self.ch == '\\' {
                self.advance();
                if !self.can_advance() {
                    break;
                }
                // check what kind of escape
                match self.ch {
                    'b' => string.push('\x08'),
                    'n' => string.push('\n'),
                    't' => string.push('\t'),
                    'r' => string.push('\r'),
                    '\'' => string.push('\''),
                    '\"' => string.push('\"'),
                    '\\' => string.push('\\'),
                    // Add more escape characters if needed
                    _ => {
                        // Invalid escape sequence
                        panic!("Invalid escape sequence");
                    }
                }
                self.advance();
                continue;
            }
            string.push(self.ch);
            self.advance();
        }
    
        // Check if end of string
        if (self.ch != '\'' && single_quote) || (self.ch != '"' && !single_quote) {
            return Err(LexerError::UnterminatedString(string))
        }
    
        // Eat the other quote/s
        self.advance();
    
        Ok(Token::new(ttype, string, this_line, this_column))
    }

    fn collect_symbol(&mut self) -> Result<Token, LexerError> {
        let this_line = self.line;
        let this_column = self.column;
        let mut symbol = String::new();
        symbol.push(self.ch);
        self.advance();
    
        let mut ttype = self.find_symbol_type(&symbol);
    
        // If it's a double character symbol
        if self.can_advance() && self.is_symbol(self.ch) {
            let new_symbol = format!("{}{}", symbol, self.ch);
    
            let second_type = self.find_symbol_type(&new_symbol);

            if second_type != TokenType::Unknown {
                ttype = second_type;
                symbol = new_symbol;
                self.advance();
            }
        }
    
        Ok(Token::new(ttype, symbol, this_line, this_column))
    }

    fn find_symbol_type(&mut self, s: &str) -> TokenType {
        
        if let Some(token_type) = get_symbols().get(s) {
            *token_type
        } else {
            TokenType::Unknown
        }

    }

    fn find_keyword_type(&mut self, identifier: &str) -> KeywordType {
        let keywords: HashMap<&str, KeywordType> = HashMap::from([
            ("null", KeywordType::Null),
            ("if", KeywordType::If),
            ("else", KeywordType::Else),
            ("while", KeywordType::While),
            ("for", KeywordType::For),
            ("return", KeywordType::Return),
            ("import", KeywordType::Import),
            ("as", KeywordType::As),
            ("from", KeywordType::From),
            ("break", KeywordType::Break),
            ("continue", KeywordType::Continue),
        ]);
    
        // Use an iterator to find the keyword
        if let Some(keyword_type) = keywords.get(identifier) {
            *keyword_type
        } else {
            KeywordType::Unknown
        }
    }

    fn is_symbol(&mut self, ch: char) -> bool {
        return "()[]{}=@#,.:;?\\+-/*%&|^<>!~".contains(ch);
    }
}