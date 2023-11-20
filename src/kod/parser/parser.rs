use super::node::*;
use crate::kod::lexer::{lexer::Lexer, token::{Token, TokenType, KeywordType}};

#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
    getting_params: bool,
}

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(Token),
    UnexpectedTokenExpected(Token, TokenType),
    UnfinishedList(Token),
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::UnexpectedToken(t) => write!(f, "Unexpected token: {:?}", t),
            ParserError::UnexpectedTokenExpected(t, expected) => write!(f, "Unexpected token: {:?}, expected: {:?}", t, expected),
            ParserError::UnfinishedList(t) => write!(f, "Unfinished list: {:?}", t),
        }
    }
}

impl std::error::Error for ParserError {}   

impl Parser {
    pub fn new(lexer: Lexer) -> Parser {
        Parser {
            lexer,
            getting_params: false,
        }
    }

    fn eat(&mut self, token_type: &TokenType) -> Result<(), Box<dyn std::error::Error>> {
        let next = self.lexer.next()?;
        if next.token_type == *token_type {
            Ok(())
        } else {
            Err(ParserError::UnexpectedTokenExpected(next, *token_type).into())
        }
    }

    fn skip_newline_or_semicolon(&mut self) {
        while match self.lexer.peek().unwrap().token_type {
            TokenType::NewLine | TokenType::SEMI => true,
            _ => false  
        } {
            self.lexer.next().unwrap();
        }
    }

    pub fn parse(&mut self) -> Result<Box<dyn Node>, Box<dyn std::error::Error>> {
        let mut block = Box::new(BlockNode { statements: vec![] });

        loop {
            self.skip_newline_or_semicolon();
            if let Some(statement) = self.parse_statement()? {
                block.statements.push(statement);
                continue;
            }
            
            break;
        }

        Ok(block)
    }

    fn parse_statement(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        let left = self.parse_commas()?;
        if left.is_none() { return Ok(None); }

        let op = self.lexer.peek().unwrap().token_type;

        if !vec![TokenType::EQUALS, TokenType::AddEq, TokenType::SubEq, TokenType::MulEq, TokenType::DivEq, TokenType::ModEq].contains(&op) {
            return Ok(left);
        }

        self.eat(&op)?;

        let right = self.parse_assignment()?;
        if right.is_none() { return Err(ParserError::UnexpectedToken(self.lexer.peek().unwrap()).into()); }

        Ok(Some(Box::new(AssignmentNode { left: left.unwrap(), right: right.unwrap(), op })))
    }

    fn parse_commas(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        let value = self.parse_bool_or()?;

        if !self.getting_params && value.is_some() && self.lexer.peek().unwrap().token_type == TokenType::COMMA {
            let mut tuple = Box::new(TupleNode { values: vec![value.unwrap()], is_list: false });

            while self.lexer.peek().unwrap().token_type == TokenType::COMMA {
                self.eat(&TokenType::COMMA)?;
                if self.lexer.peek().unwrap().token_type == TokenType::EndOfFile {
                    break;
                }
                let v = self.parse_bool_or()?;
                if v.is_none() { break; }

                tuple.values.push(v.unwrap());
            }

            return Ok(Some(tuple));
        }

        Ok(value)
    }

    fn parse_bool_or(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        self.parse_binary_op(vec![TokenType::OR], |p| p.parse_bool_and())
    }

    fn parse_bool_and(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        self.parse_binary_op(vec![TokenType::AND], |p| p.parse_bitwise_or())
    }

    fn parse_bitwise_or(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        self.parse_binary_op(vec![TokenType::OR], |p| p.parse_bitwise_xor())
    }

    fn parse_bitwise_xor(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        self.parse_binary_op(vec![TokenType::HAT], |p| p.parse_bitwise_and())
    }

    fn parse_bitwise_and(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        self.parse_binary_op(vec![TokenType::AND], |p| p.parse_bool_equals())
    }

    fn parse_bool_equals(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        self.parse_binary_op(vec![TokenType::BoolEq, TokenType::BoolNe], |p| p.parse_gltl())
    }

    fn parse_gltl(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        self.parse_binary_op(vec![TokenType::BoolGt, TokenType::BoolLt, TokenType::BoolGte, TokenType::BoolLte], |p| p.parse_shlr())
    }

    fn parse_shlr(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        self.parse_binary_op(vec![TokenType::SHL, TokenType::SHR], |p| p.parse_add_sub())
    }

    fn parse_add_sub(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        self.parse_binary_op(vec![TokenType::ADD, TokenType::SUB], |p| p.parse_mul_div_mod())
    }

    fn parse_mul_div_mod(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        self.parse_binary_op(vec![TokenType::MUL, TokenType::DIV, TokenType::MOD], |p| p.parse_pow())
    }

    fn parse_pow(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        self.parse_binary_op(vec![TokenType::POW], |p| p.parse_before())
    }

    fn parse_before(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        let unary_ops = vec![TokenType::NOT, TokenType::BoolNot, TokenType::SUB, TokenType::ADD];

        let ttype = self.lexer.peek().unwrap().token_type;
        if !unary_ops.contains(&ttype) {
            return self.parse_after(None);
        }

        self.eat(&ttype)?;
      
        let mut value = self.parse_after(None)?;
        if value.is_none() {
            return Err(ParserError::UnexpectedToken(self.lexer.peek().unwrap()).into());
        }

        if let Some(int_node) = value.as_deref_mut().unwrap().get_int_mut() {
            match ttype {
                TokenType::NOT => {
                    int_node.value = !int_node.value;
                },
                TokenType::BoolNot => {
                    int_node.value = (int_node.value == 0) as i64;
                },
                TokenType::SUB => {
                    int_node.value = -int_node.value;
                },
                _ => (),
            }

            return Ok(value);
        } else if let Some(float_node) = value.as_deref_mut().unwrap().get_float_mut() {
            match ttype {
                TokenType::BoolNot => {
                    float_node.value = (float_node.value == 0.0) as i64 as f64;
                },
                TokenType::SUB => {
                    float_node.value = -float_node.value;
                },
                _ => (),
            }

            return Ok(value);
        }
        
        return Ok(Some(Box::new(UnaryOpNode { op: ttype, value: value.unwrap() })));
    }

    fn parse_after(&mut self, prev: Option<Box<dyn Node>>) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        let value = if prev.is_none() { self.parse_factor()? } else { prev.into() };

        match self.lexer.peek().unwrap().token_type {
            TokenType::LPAREN => {
                self.getting_params = true;
                let list: Vec<Box<dyn Node>> = match self.parse_tuple(true)? {
                    Some(ls) if ls.get_tuple().is_some() => ls.take_tuple().unwrap().values,
                    Some(ls) => vec![ls],
                    _ => vec![],
                };
                self.getting_params = false;

                if value.as_ref().unwrap().get_id().is_some() {
                    if self.lexer.peek().unwrap().token_type == TokenType::LBRACE {
                        return Ok(Some(Box::new(FuncDefNode { 
                            name: Box::new(value.unwrap().take_id().unwrap()), 
                            params: list, 
                            body: Box::new(self.parse_block()?.unwrap().take_block().unwrap()) 
                        })));
                    }
                }

                return self.parse_after(Some(Box::new(FuncCallNode {
                    callie: value.unwrap(),
                    args: list,
                })))
            },
            
            TokenType::DOT => {
                self.eat(&TokenType::DOT)?;

                let field = self.parse_id()?;
                return self.parse_after(Some(Box::new(AccessNode { value: value.unwrap(), field: field.unwrap(), load_self: false })));
            },

            TokenType::LBRACKET => {
                
                self.eat(&TokenType::LBRACKET)?;
                let subscript = self.parse_statement()?;
                self.eat(&TokenType::RBRACKET)?;

                return self.parse_after(Some(Box::new(SubscriptNode { value: value.unwrap(), subscript: subscript.unwrap() })));
            },

            _ => (),
        }

        Ok(value)
    }

    fn parse_factor(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        match self.lexer.peek().unwrap().token_type {

            TokenType::INT => self.parse_int(),
            TokenType::FLOAT => self.parse_float(),
            TokenType::STRING => self.parse_string(),

            TokenType::ID => self.parse_id(),

            TokenType::LPAREN => self.parse_tuple(false),
            TokenType::LBRACKET => self.parse_list(),

            TokenType::KEYWORD => self.parse_keyword(),

            TokenType::EndOfFile | TokenType::NewLine => Ok(None),

            _ => {
                Err(ParserError::UnexpectedToken(self.lexer.peek().unwrap()).into())
            }
        }
    }

    fn parse_int(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        Ok(Some(Box::new(IntNode { value: self.lexer.next()?.int_value })))
    }
    
    fn parse_float(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        Ok(Some(Box::new(FloatNode { value: self.lexer.next()?.float_value })))
    }

    fn parse_string(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        Ok(Some(Box::new(StringNode { value: self.lexer.next()?.value })))
    }

    fn parse_id(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        Ok(Some(Box::new(IdNode { value: self.lexer.next()?.value })))
    }

    fn parse_keyword(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        let ktype = self.lexer.next()?.keyword_type;

        match ktype {
            KeywordType::If => self.parse_if(),
            KeywordType::While => self.parse_while(),
            KeywordType::Return => self.parse_return(),
            _ => Err(ParserError::UnexpectedToken(self.lexer.peek().unwrap()).into()),
        }
    }

    fn parse_condition_block(&mut self) -> Result<(Box<dyn Node>, Box<dyn Node>), Box<dyn std::error::Error>> {
        let condition = self.parse_statement()?;
        if condition.is_none() {
            return Err(ParserError::UnexpectedToken(self.lexer.peek().unwrap()).into());
        }

        let block = self.parse_block()?;
        if block.is_none() {
            return Err(ParserError::UnexpectedToken(self.lexer.peek().unwrap()).into());
        }

        return Ok((condition.unwrap(), block.unwrap()));
    }

    fn parse_if(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        let (condition, block) = self.parse_condition_block()?;
        return Ok(Some(Box::new(IfNode { condition, block })));
    }

    fn parse_while(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        let (condition, block) = self.parse_condition_block()?;
        return Ok(Some(Box::new(WhileNode { condition, block })));
    }

    fn parse_return(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        Ok(Some(Box::new(ReturnNode { value: self.parse_statement()? })))
    }

    fn parse_binary_op(&mut self, ops: Vec<TokenType>, parse_func: fn(&mut Self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>>) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        let mut left = parse_func(self)?;

        while true && !self.getting_params {
            let op = self.lexer.peek()?.token_type;

            if !ops.contains(&op) { break; }

            self.eat(&op)?;
            let mut right = parse_func(self)?;
            if right.is_none() {
                return Err(ParserError::UnexpectedToken(self.lexer.peek().unwrap()).into());
            }

            // optimize for addition
            match op {
                TokenType::ADD => {
                    if let Some(int_node) = left.as_deref_mut().unwrap().get_int_mut() {
                        if let Some(int_node_right) = right.as_deref_mut().unwrap().get_int_mut() {
                            int_node.value += int_node_right.value;
                            continue;
                        }
                        if let Some(float_node_right) = right.as_deref_mut().unwrap().get_float_mut() {
                            float_node_right.value += int_node.value as f64;
                            left = Some(Box::new(FloatNode { value: float_node_right.value }));
                            continue;
                        }
                    }
                    else if let Some(float_node) = left.as_deref_mut().unwrap().get_float_mut() {
                        if let Some(float_node_right) = right.as_deref_mut().unwrap().get_float_mut() {
                            float_node.value += float_node_right.value;
                            continue;
                        }
                        if let Some(int_node_right) = right.as_deref_mut().unwrap().get_int_mut() {
                            float_node.value += int_node_right.value as f64;
                            continue;
                        }
                    }
                },
                TokenType::SUB => {
                    if let Some(int_node) = left.as_deref_mut().unwrap().get_int_mut() {
                        if let Some(int_node_right) = right.as_deref_mut().unwrap().get_int_mut() {
                            int_node.value -= int_node_right.value;
                            continue;
                        }
                        if let Some(float_node_right) = right.as_deref_mut().unwrap().get_float_mut() {
                            left = Some(Box::new(FloatNode { value: int_node.value as f64 - float_node_right.value }));
                            continue;
                        }
                    }
                    else if let Some(float_node) = left.as_deref_mut().unwrap().get_float_mut() {
                        if let Some(float_node_right) = right.as_deref_mut().unwrap().get_float_mut() {
                            float_node.value -= float_node_right.value;
                            continue;
                        }
                        if let Some(int_node_right) = right.as_deref_mut().unwrap().get_int_mut() {
                            float_node.value -= int_node_right.value as f64;
                            continue;
                        }
                    }
                },
                TokenType::MUL => {
                    if let Some(int_node) = left.as_deref_mut().unwrap().get_int_mut() {
                        if let Some(int_node_right) = right.as_deref_mut().unwrap().get_int_mut() {
                            int_node.value *= int_node_right.value;
                            continue;
                        }
                        if let Some(float_node_right) = right.as_deref_mut().unwrap().get_float_mut() {
                            float_node_right.value *= int_node.value as f64;
                            left = Some(Box::new(FloatNode { value: float_node_right.value }));
                            continue;
                        }
                    }
                    else if let Some(float_node) = left.as_deref_mut().unwrap().get_float_mut() {
                        if let Some(float_node_right) = right.as_deref_mut().unwrap().get_float_mut() {
                            float_node.value *= float_node_right.value;
                            continue;
                        }
                        if let Some(int_node_right) = right.as_deref_mut().unwrap().get_int_mut() {
                            float_node.value *= int_node_right.value as f64;
                            continue;
                        }
                    }
                },
                TokenType::SHR => {
                    if let Some(int_node) = left.as_deref_mut().unwrap().get_int_mut() {
                        if let Some(int_node_right) = right.as_deref_mut().unwrap().get_int_mut() {
                            int_node.value >>= int_node_right.value;
                            continue;
                        }
                    }
                },
                TokenType::SHL => {
                    if let Some(int_node) = left.as_deref_mut().unwrap().get_int_mut() {
                        if let Some(int_node_right) = right.as_deref_mut().unwrap().get_int_mut() {
                            int_node.value <<= int_node_right.value;
                            continue;
                        }
                    }
                },
                _ => (),
            }

            left = Some(Box::new(BinaryOpNode { left: left.unwrap(), op, right: right.unwrap() }));
        }

        return Ok(left);
    }

    fn parse_body(
            &mut self,
            delims: (TokenType, TokenType),
            parse_commas: bool,
            got_comma: &mut Option<&mut bool>,
        ) -> Result<Vec<Box<dyn Node>>, Box<dyn std::error::Error>> 
    {
        let mut nodes = Vec::new();
        if let Some(ref mut got_comma_ref) = got_comma {
            **got_comma_ref = false;
        }

        self.eat(&delims.0)?;
    
        self.skip_newline_or_semicolon();
    
        loop {
            let ttype = self.lexer.peek()?.token_type;

            if ttype == delims.1 || ttype == TokenType::EndOfFile {
                self.eat(&delims.1)?;
                break;
            }
    
            self.skip_newline_or_semicolon();
            let expr = self.parse_statement()?;

            if expr.is_none() {
                return Err(ParserError::UnexpectedToken(self.lexer.peek().unwrap()).into());
            }
            
            nodes.push(expr.unwrap());
            self.skip_newline_or_semicolon();
    
            if parse_commas {
                if self.lexer.peek()?.token_type == TokenType::COMMA {
                    self.eat(&TokenType::COMMA)?;
                    if let Some(ref mut got_comma_ref) = got_comma {
                        **got_comma_ref = true;
                    }

                    if got_comma.is_none() && self.lexer.peek()?.token_type == delims.1{
                        return Err(ParserError::UnfinishedList(self.lexer.peek().unwrap()).into());
                    }
                } else {
                    self.eat(&delims.1)?;
                    break;
                }
            }
        }
    
        Ok(nodes)
    }

    fn parse_tuple(&mut self, must_be_tuple: bool) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        let mut got_comma = false;

        let mut values = self.parse_body((TokenType::LPAREN, TokenType::RPAREN), true, &mut Some(&mut got_comma))?;
        
        if must_be_tuple {
            return Ok(Some(Box::new(TupleNode { values, is_list: false })));
        }
    
        if values.len() == 1 && !got_comma{
            return Ok(values.pop());
        }
    
        Ok(Some(Box::new(TupleNode { values, is_list: false })))
    }

    fn parse_list(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        self.getting_params = true;
        let values = self.parse_body((TokenType::LBRACKET, TokenType::RBRACKET), true, &mut None)?;
        self.getting_params = false;
        
        Ok(Some(Box::new(TupleNode { values, is_list: true })))
    }

    fn parse_block(&mut self) -> Result<Option<Box<dyn Node>>, Box<dyn std::error::Error>> {
        Ok(Some(Box::new(BlockNode { statements: self.parse_body((TokenType::LBRACE, TokenType::RBRACE), false, &mut None)? })))
    }
}