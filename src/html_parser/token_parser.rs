use std::collections::HashMap;

use crate::html_parser::lexer::{LexerToken, LexerTokenType};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TagPairType {
    Opening,
    Closing,
    Singular,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TagType {
    Root,
    DOCTYPE,
    HTML,

    Head,
    Title,

    Body,

    H1,
    H2,
    P,
    
    LineBreak,
    HorizontalBreak,

    Img,

    Comment,
    Text,
    Unhandled,

    EOF,
}

#[derive(Clone)]
pub struct HTMLTagToken {
    pub char_index: usize,
    pub char_length: usize,
    pub position: (usize, usize),
    pub tag_type: TagType,
    pub pair_type: TagPairType,
    pub content: Option<String>,
    pub attributes: Option<HashMap<String, String>>,

}

impl HTMLTagToken {
    pub fn new_eof_token(lexer_token: &LexerToken) -> Self {
        Self {
            char_index: lexer_token.index - 1,
            char_length: 1,
            position: lexer_token.position,
            tag_type: TagType::EOF,
            pair_type: TagPairType::Opening,
            content: None,
            attributes: None,
        }
    }
}




pub struct TokenParser {
    current_index: usize,
}

impl TokenParser {
    pub fn new() -> Self {
        Self {
            current_index: 0,
        }
    }

    pub fn parse_tokens(&mut self, token_list: &Vec<LexerToken>) -> Vec<HTMLTagToken> {
        
        let mut tag_list = Vec::new();

        loop {
            let token = &token_list[self.current_index];
            if self.consume_token(&LexerTokenType::EOF, &token_list) {
                tag_list.push(HTMLTagToken::new_eof_token(token));
                break;
            }
            else if self.consume_token(&LexerTokenType::LeftAngleBracket, &token_list) {
                if self.consume_token(&LexerTokenType::Slash, &token_list) {
                    tag_list.push(self.match_closing_tag(&token_list));
                }
                else if self.consume_token(&LexerTokenType::Bang, &token_list) {
                    tag_list.push(self.match_doctype(&token_list));
                }
                else {
                    tag_list.push(self.match_tag(&token_list));
                }
                
            }
            else if self.compare_token(&LexerTokenType::String, &token_list) {
                // Data
                tag_list.push(self.match_data(&token_list));
            }
            else if self.consume_token(&LexerTokenType::NewLine, &token_list) || self.consume_token(&LexerTokenType::CarriageReturn, &token_list) || 
                self.consume_token(&LexerTokenType::Space, &token_list) {
                // consume whitespace
            }
            else if self.consume_token(&LexerTokenType::Comment, &token_list) {
                println!("Matched Comment");
            }
            else {
                println!("Unhandled token: {:4} of {:4} at {}: {:?}, {} ", self.current_index, token_list.len(), token.index, token.token_type, token.raw_string.escape_debug());
                self.advance_token(&token_list);
            }
        }

        for i in 0..tag_list.len() {
            let tag = &tag_list[i];
            let mut content_str = "None";
            if tag.content.is_some() {
                content_str = tag.content.as_ref().unwrap();
            }
            println!("{}: label: '{:?}' type: '{:?}' content: {}", tag.char_index, tag.tag_type, tag.pair_type, content_str);
        }

        println!("End of file Parser");

        tag_list
    }

    pub fn match_data(&mut self, token_list: &Vec<LexerToken>) -> HTMLTagToken {
        let mut content_str = "".to_string();
        let index = token_list[self.current_index].index;
        let position = token_list[self.current_index].position;

        loop {
            if self.consume_token(&LexerTokenType::EOF, token_list) {
                println!("Error: Reached end of file while matching data");
                break;
            }
            else if self.compare_token(&LexerTokenType::LeftAngleBracket, token_list) {
                break;
            }
            else {
                let t = &token_list[self.current_index];
                content_str += &t.raw_string;
                self.advance_token(token_list);
            }
        }

        let length = token_list[self.current_index].index - index;

        return HTMLTagToken { 
            char_index: index,
            char_length: length,
            position,
            tag_type: TagType::Text, 
            pair_type: TagPairType::Singular, 
            content: Some(content_str), 
            attributes: None, 
        }
    }

    pub fn match_tag(&mut self, token_list: &Vec<LexerToken>) -> HTMLTagToken {
        let index = token_list[self.current_index].index - 1;
        let position = token_list[self.current_index - 1].position;
        

        let t1 = &token_list[self.current_index];
        println!("Match {}", t1.raw_string.escape_debug());

        let mut tag_type = TagType::Unhandled;
        let mut pair_type = TagPairType::Opening;
        if self.compare_token(&LexerTokenType::String, token_list) {
            tag_type = self.get_tag_enum(&token_list[self.current_index].raw_string);
            if TokenParser::is_singular(&tag_type) {
                pair_type = TagPairType::Singular;
            }
            self.advance_token(token_list);
        }

        self.consume_whitespace(token_list);
        if self.consume_token(&LexerTokenType::RightAngleBracket, token_list) {
            let length = token_list[self.current_index].index - index;

            return HTMLTagToken { 
                char_index: index,
                char_length: length,
                position,
                tag_type, 
                pair_type, 
                content: None, 
                attributes: None, 
            }
        }

        let mut attribute_list = HashMap::new();
        for _ in 0..10 {
            if let Some(attribute) = self.match_attribute(token_list) {
                println!("Built key val pair {}={}", attribute.0, attribute.1);
                attribute_list.insert(attribute.0, attribute.1);
                self.consume_whitespace(token_list);
                if self.consume_token(&LexerTokenType::RightAngleBracket, token_list) {
                    break;
                }
                
            }
            else {
                break;
            }
        } 

        let mut attributes = None;
        if attribute_list.len() > 0 {
            attributes = Some(attribute_list);
        }

        let length = token_list[self.current_index].index - index;
        return HTMLTagToken { 
            char_index: index,
            char_length: length,
            position,
            tag_type, 
            pair_type, 
            content: None, 
            attributes, 
        }

    }

    pub fn match_closing_tag(&mut self, token_list: &Vec<LexerToken>) -> HTMLTagToken {
        let index = token_list[self.current_index - 2].index; 
        let position = token_list[self.current_index - 2].position;
        
        let mut tag_type = TagType::Unhandled;
        if self.compare_token(&LexerTokenType::String, &token_list) {
            tag_type = self.get_tag_enum(&token_list[self.current_index].raw_string);
            self.advance_token(token_list);
        }
        else {
            let t = &token_list[self.current_index];
            println!("Error at {}: Expected string after '</' and found {:?}: {}", t.index, t.token_type, t.raw_string);
        }

        if !self.consume_token(&LexerTokenType::RightAngleBracket, token_list) {
            let t = &token_list[self.current_index];
            println!("Error at {}: Expected '>' to close tag after '</{}', found {:?}: '{}'", t.index, t.raw_string, t.token_type, t.raw_string.escape_debug());
        }

        let length = token_list[self.current_index].index - index; 
        HTMLTagToken { 
            char_index: index,
            char_length: length,
            position,
            tag_type, 
            pair_type: TagPairType::Closing, 
            content: None, 
            attributes: None, 
        }
    }

    fn match_doctype(&mut self, token_list: &Vec<LexerToken>) -> HTMLTagToken {
        let index = token_list[self.current_index - 2].index;
        let position = token_list[self.current_index - 2].position;

        let mut tag_type = TagType::Unhandled;
        if self.compare_token(&LexerTokenType::String, token_list) {
            tag_type = self.get_tag_enum(&token_list[self.current_index].raw_string);
            self.advance_token(token_list);
        }
        else {
            let t = &token_list[self.current_index];
            println!("Error at {}: Expected to match string in !DOCTYPE tag, but found {:?}: '{}'", t.index, t.token_type, t.raw_string.escape_debug());
        }
        self.consume_whitespace(token_list);

        let mut content = None;
        if self.compare_token(&LexerTokenType::String, token_list) {
            content = Some(token_list[self.current_index].raw_string.clone());
            self.advance_token(token_list);
        }
        else {
            let t = &token_list[self.current_index];
            println!("Error at {}: I still don't understand why the fuck this tag exists, but 'html' wasn't found in !DOCTYPE {:?}: '{}'", t.index, t.token_type, t.raw_string.escape_debug());
        }
        if !self.consume_token(&LexerTokenType::RightAngleBracket, token_list) {
            let t = &token_list[self.current_index];
            println!("Error at {}: !DOCTYPE not terminated correctly with '>' {:?}: '{}'", t.index, t.token_type, t.raw_string.escape_debug());
        }

        let length = token_list[self.current_index].index - index;
        HTMLTagToken { 
            char_index: index,
            char_length: length,
            position,
            tag_type, 
            pair_type: TagPairType::Opening, 
            content, 
            attributes: None, 
        }
    }

    fn match_attribute(&mut self, token_list: &Vec<LexerToken>) -> Option<(String, String)> {
        let t_test = &token_list[self.current_index];

        if self.consume_token(&LexerTokenType::Dot, token_list) {
            return None
        }

        let mut key = "None".to_string();
        if self.compare_token(&LexerTokenType::String, token_list) {
            key = token_list[self.current_index].raw_string.clone();
            self.advance_token(token_list);
        }
        else {
            let t = &token_list[self.current_index];
            println!("Error at {}: No key String found for match_attribute, found {:?}: '{}'", t.index, t.token_type, t.raw_string.escape_debug());
            return None
        }

        if !self.consume_token(&LexerTokenType::Equal, token_list) {
            let t = &token_list[self.current_index];
            println!("Error at {}: Expected equal sign in match_attribute, found {:?}: '{}'", t.index, t.token_type, t.raw_string.escape_debug());
            return None
        }

        let mut val = "None".to_string();
        if self.compare_token(&LexerTokenType::String, token_list) {
            val = token_list[self.current_index].raw_string.clone();
        }
        else if self.consume_token(&LexerTokenType::SingleQuote, token_list) {
            val = self.build_quoted_string(&LexerTokenType::SingleQuote, token_list);
        }
        else if self.consume_token(&LexerTokenType::DoubleQuote, token_list) {
            val = self.build_quoted_string(&LexerTokenType::DoubleQuote, token_list);
        }
        else {
            let t = &token_list[self.current_index];
            println!("Error at {}: No value String found for match_attribute, found {:?}: '{}'", t.index, t.token_type, t.raw_string.escape_debug());
            return None
        }
        Some((key, val))
    }

    pub fn consume_whitespace(&mut self, token_list: &Vec<LexerToken>) {
        loop {
            let t = &token_list[self.current_index];
            if t.token_type == LexerTokenType::NewLine || t.token_type == LexerTokenType::CarriageReturn || t.token_type == LexerTokenType::Space {
                self.current_index += 1;
                continue;
            }
            break;
        }
    }
    

    fn build_quoted_string(&mut self, delimiter: &LexerTokenType, token_list: &Vec<LexerToken>) -> String {
        let mut str = "".to_string();
        loop {
            let t = &token_list[self.current_index];
            // while token isn't a delimiter, build the string from raw_String
            if t.token_type == LexerTokenType::EOF {
                println!("Error at {}: Reached end of file without matching '{:?}'", t.index, delimiter);
            }
            else if self.consume_token(delimiter, token_list) {
                break;
            }
            else {
                str += &t.raw_string;
                self.advance_token(token_list);
            }
        }
        str
    }

    fn get_next_nth(&self, n: usize, token_list: &Vec<LexerToken>) -> LexerToken {
        if self.current_index + n >= token_list.len() {
            return token_list[token_list.len() - 1].clone();
        }
        token_list[self.current_index + n].clone()
    }

    fn advance_token(&mut self, token_list: &Vec<LexerToken>) -> LexerToken {
        self.current_index += 1;
        if self.current_index >= token_list.len() {
            self.current_index = token_list.len() - 1;
        }
        token_list[self.current_index].clone()
    }

    fn compare_token(&self, pattern: &LexerTokenType, token_list: &Vec<LexerToken>) -> bool {
        token_list[self.current_index].token_type == *pattern
    }

    fn consume_token(&mut self, pattern: &LexerTokenType, token_list: &Vec<LexerToken>) -> bool {
        if token_list[self.current_index].token_type == *pattern {
            self.advance_token(token_list);
            return true
        }
        false
    }

    fn get_tag_enum(&self, str: &str) -> TagType {
        let tag;
        if str == "DOCTYPE" {
            tag = TagType::DOCTYPE;
        }
        else if str == "html" {
            tag = TagType::HTML;
        }
        else if str == "head" {
            tag = TagType::Head;
        }
        else if str == "title" {
            tag = TagType::Title;
        }
        else if str == "body" {
            tag = TagType::Body;
        }
        else if str == "h1" {
            tag = TagType::H1;
        }
        else if str == "h2" {
            tag = TagType::H2;
        }
        else if str == "p" {
            tag = TagType::P;
        }
        else if str == "img" {
            tag = TagType::Img;
        }
        else if str == "br" {
            tag = TagType::LineBreak;
        }
        else if str == "hr" {
            tag = TagType::HorizontalBreak;
        }
        else {
            tag = TagType::Unhandled;
        }

        tag
    }

    fn is_singular(tag_type_enum: &TagType) -> bool {
        match tag_type_enum {
            TagType::LineBreak => true,
            TagType::HorizontalBreak => true,
            TagType::Img => true,
            _ => false,
        }
    }
}