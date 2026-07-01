

const END_OF_FILE: &str = "EOF";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LexerTokenType {
    LeftParenthesis,
    RightParenthesis,
    LeftCurlyBrace,
    RightCurlyBrace,
    LeftSquareBracket,
    RightSquareBracket,
    LeftAngleBracket,
    RightAngleBracket,
    DoubleQuote,
    SingleQuote,
    Comma,
    Dot,
    Star,
    Slash,
    BackSlash,
    Colon,
    SemiColon,
    Hyphen,
    Underscore,
    
    Bang,
    Ampersand,
    Equal,

    String, 
    Number,

    Space,
    CarriageReturn,
    NewLine,
    Tab,

    LineBreak,
    HorizontalBreak,

    EOF,
    Comment,
    Error(String),
    
}

#[derive(Clone)]
pub struct LexerToken {
    pub index: usize,
    pub length: usize,
    pub raw_string: String,
    pub token_type: LexerTokenType,
    pub position: (usize, usize),
}

impl LexerToken {
    pub fn new(index: usize, pos: (usize, usize), str: &str, token_type: LexerTokenType) -> Self {
        Self {
            index, 
            length: str.len(), 
            raw_string: str.to_string(), 
            token_type,
            position: pos,
        }
    }

    pub fn new_error_message(error_text: &str, index: usize, position: (usize, usize)) -> Self {
        let message = format!("ERROR AT {}: {}", index, error_text);
        Self {
            index, 
            length: 0, 
            raw_string: "".to_string(), 
            token_type: LexerTokenType::Error(message),
            position,
        }
    }

    pub fn get_print_string(&self) -> String {
        let mut token_type_string = format!("{:?}", self.token_type);
        while token_type_string.len() < 20 {
            token_type_string += " ";
        }
        format!("Row: {}, Col: {} Token: {:15}, index: {:4}, str: {}", self.position.1, self.position.0, token_type_string, self.index, self.raw_string.escape_debug())
    }
}

pub struct HTMLLexer {
    current_index: usize,
    position: (usize, usize),
}

impl HTMLLexer {
    pub fn new() -> Self {
        Self {
            current_index: 0,
            position: (1, 1),
        }
    }

    pub fn lex_file(&mut self, text: &str) -> Vec<LexerToken> {
        self.current_index = 0;
        let mut token_list = Vec::new();

        loop {
            let c = self.get_next_nth(0, text);
            let pos = self.position;
            let current_index = self.current_index;
            if self.consume_char(END_OF_FILE, text) {
                token_list.push(LexerToken::new(current_index, pos, END_OF_FILE, LexerTokenType::EOF));
                break;
            }
            else if c == "<" {
                if self.compare_string("<!--", text) {
                    token_list.push(self.match_comment(text));
                }
                else if self.compare_string("<br>", text) {
                    token_list.push(self.match_line_break(text));
                }
                else if self.compare_string("<hr>", text) {
                    token_list.push(self.match_horizontal_break(text));
                }
                else {
                    token_list.push(LexerToken::new(current_index, pos, &c, LexerTokenType::LeftAngleBracket));
                    self.advance_char(text);
                }
            }
            else if self.consume_char(">", text) {
                token_list.push(LexerToken::new(current_index, pos, &c, LexerTokenType::RightAngleBracket));
            }
            else if is_ascii_letter(&c) {
                token_list.push(self.match_string(text));
            }
            else if is_number(&c) {
                token_list.push(self.match_number(text));
            }
            else if self.consume_char("!", text) {
                token_list.push(LexerToken::new(current_index, pos, &c, LexerTokenType::Bang));
            }
            else if self.consume_char("/", text) {
                token_list.push(LexerToken::new(current_index, pos, &c, LexerTokenType::Slash));
            }
            else if self.consume_char("\'", text) {
                token_list.push(LexerToken::new(current_index, pos, &c, LexerTokenType::SingleQuote));
            }
            else if self.consume_char("\"", text) {
                token_list.push(LexerToken::new(current_index, pos, &c, LexerTokenType::DoubleQuote));
            }
            else if self.consume_char("\r", text) {
                token_list.push(LexerToken::new(current_index, pos, &c, LexerTokenType::CarriageReturn));
            }
            else if self.consume_char("\n", text) {
                token_list.push(LexerToken::new(current_index, pos, &c, LexerTokenType::NewLine));
            }
            else if self.consume_char(" ", text) {
                token_list.push(LexerToken::new(current_index, pos, &c, LexerTokenType::Space));
            }
            else if self.consume_char("=", text) {
                token_list.push(LexerToken::new(current_index, pos, &c, LexerTokenType::Equal));
            }
            else if self.consume_char(".", text) {
                token_list.push(LexerToken::new(current_index, pos, &c, LexerTokenType::Dot));
            }
            else if self.consume_char(",", text) {
                token_list.push(LexerToken::new(current_index, pos, &c, LexerTokenType::Comma));
            }
            else {
                // Any chars to add?
                // Error
                println!("Unmatched char: '{}' at {}", c, self.current_index);
                self.advance_char(text);
            }
        }

        for i in 0..token_list.len() {
            println!("{}", token_list[i].get_print_string())
        }

        token_list
    }

    fn get_next_nth(&self, n: usize, text: &str) -> String {
        let i = self.current_index + n;
        if i >= text.len() {
            return END_OF_FILE.to_string()
        }
        text[i..i + 1].to_string()
    }

    fn advance_char(&mut self, text: &str) -> String {
        let c = self.get_next_nth(0, text);
        if c == *"\n" {
            self.position.0 = 1;
            self.position.1 += 1;
        }
        else if c != END_OF_FILE {
            self.position.0 += 1;
        }

        self.current_index += 1;
        if self.current_index >= text.len() {
            self.current_index = text.len();
            return END_OF_FILE.to_string()
        }
        text[self.current_index..self.current_index + 1].to_string()
    }
    
    pub fn consume_char(&mut self, char: &str, text: &str) -> bool {

        let curr_c = self.get_next_nth(0, text);
        if char == curr_c {
            self.advance_char(text);
            return true;
        }
        false
    }

    pub fn compare_string(&self, cmp: &str, text: &str) -> bool {
        if self.current_index + cmp.len() < text.len() {
            let next_string = &text[self.current_index..(self.current_index + cmp.len())];
            if cmp == next_string {
                return true;
            }
        }
        false
    }

    fn match_string(&mut self, text: &str) -> LexerToken {
        let start_index = self.current_index;
        let start_pos = self.position;
        let mut str = "".to_string();
        loop {
            let c = &text[self.current_index..self.current_index + 1];
            if c == END_OF_FILE {
                break;
            }
            else if is_valid_string_char(&c) {
                str += &c;
                self.current_index += 1;
                if c == "\n" {
                    self.position.0 = 1;
                    self.position.1 += 1;
                }
                else if c != END_OF_FILE {
                    self.position.0 += 1;
                }
            }
            else {
                break;
            }
        }
        LexerToken {
            index: start_index,
            length: str.len(),
            raw_string: str.to_string(),
            token_type: LexerTokenType::String,
            position: start_pos,
        }
    }

    fn match_number(&mut self, text: &str) -> LexerToken {
        let start_index = self.current_index;
        let start_pos = self.position;
        let mut str = "".to_string();
        loop {
            let c = &text[self.current_index..self.current_index + 1];
            if c == END_OF_FILE {
                break;
            }
            else if is_number(&c) {
                str += &c;
                self.current_index += 1;
                self.position.0 += 1;
            }
            else if c == "." {
                str += &c;
                self.current_index += 1;
                self.position.0 += 1;
                loop {
                    let c = &text[self.current_index..self.current_index + 1];
                    if c == END_OF_FILE {
                        break;
                    }
                    else if is_number(&c) {
                        str += &c;
                        self.current_index += 1;
                        self.position.0 += 1;
                    }
                    else {
                        break;
                    }
                }
                break;
            }
            else {
                break;
            } 
        }
        LexerToken {
            index: start_index,
            length: str.len(),
            raw_string: str.to_string(),
            token_type: LexerTokenType::Number,
            position: start_pos,
        }
    }

    fn match_comment(&mut self, text: &str) -> LexerToken {
        let start_index = self.current_index;
        let start_pos = self.position;
        let mut token_string = "".to_string() + &text[self.current_index..self.current_index + 4];
        let mut length = 3;
        self.current_index += 3;
        self.position.0 += 3;

        let mut c = self.advance_char(text);
        
        loop {
            if c == END_OF_FILE.to_string() {
                return LexerToken::new_error_message("No end found for comment", start_index, self.position);
            }
            if self.compare_string("-->", text) {
                token_string += "-->";
                self.current_index += 3;
                self.position.0 += 3;
                break;
            }
            
            token_string += &c.to_string();
            length += 1;
            c = self.advance_char(text);
        }

        LexerToken { 
            index: start_index, 
            length, 
            raw_string: token_string, 
            token_type: LexerTokenType::Comment,
            position: start_pos,
        }
    }

    fn match_line_break(&mut self, text: &str) -> LexerToken {
        let start_index = self.current_index;

        let mut str = "".to_string();
        loop {
            let c = self.advance_char(text);
            if c == "END_OF_FILE" {
                break;
            }
            else if c == ">" {
                self.advance_char(text);
                break;
            }
            else {
                str += &c;
            }
        }

        LexerToken { 
            index: start_index, 
            length: str.len(), 
            raw_string: str, 
            token_type: LexerTokenType::LineBreak,
            position: self.position,
        }
    }

    fn match_horizontal_break(&mut self, text: &str) -> LexerToken {
        let start_index = self.current_index;

        let mut str = "".to_string();
        loop {
            let c = self.advance_char(text);
            if c == "END_OF_FILE" {
                break;
            }
            else if c == ">" {
                self.advance_char(text);
                break;
            }
            else {
                str += &c;
            }
        }

        LexerToken { 
            index: start_index, 
            length: str.len(), 
            raw_string: str, 
            token_type: LexerTokenType::HorizontalBreak,
            position: self.position,
        }
    }
    
}


fn is_ascii_letter(c: &str) -> bool {
    let byte = c.as_bytes()[0];
    if byte >= 0x41 && byte <= 0x5A {
        return true;
    }
    if byte >= 0x61 && byte <= 0x7A {
        return true;
    }

    false
}

fn is_number(c: &str) -> bool {
    let byte = c.as_bytes()[0];
    if byte >= 0x30 && byte <= 0x39 {
        return true;
    }
    false
}

fn is_valid_string_char(c: &str) -> bool {
    if is_ascii_letter(c) || is_number(c) || c == "-" || c == "_" {
        return true
    }
    false
}


