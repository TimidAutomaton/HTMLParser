

#![allow(warnings)]
use std::{collections::HashMap, fs};


// mod html_parser;
// use crate::html_parser::{lexer::HTMLLexer};

mod html_parser;
use crate::html_parser::{lexer::HTMLLexer, token_parser::TokenParser, tree_builder::{HTMLTag, HTMLTreeBuilder}};

fn main() {
    let html_file = fs::read_to_string("./resources/html_test.html")
        .expect("Should have been able to read the file");
    
    // let mut parser = HTMLLexer::new();
    // let token_list = parser.parse_text(&html_file);
    println!("------------------Lexer------------------");
    let mut lexer = HTMLLexer::new();
    let token_list = lexer.lex_file(&html_file);

    println!();
    println!();
    println!();
    println!("------------------Tag Parser------------------");

    let mut parser = TokenParser::new();
    let tag_token_list = parser.parse_tokens(&token_list);

    println!();
    println!();
    println!();
    println!("------------------Tree Builder------------------");
    let mut tree_builder = HTMLTreeBuilder::new();
    let tag_token_list = tree_builder.build_tree(&tag_token_list);

    let strings = get_tree_string(0, &tag_token_list);

    println!("------------------Tree Result------------------");
    for i in 0..strings.len() {
        let mut cur_string = "".to_string();
        for i in 0..strings[i].0 {
            cur_string += " ";
        }
        println!("{}{}", cur_string, strings[i].1);
    }
}


fn get_tree_string(root: usize, map: &HashMap<usize, HTMLTag>) -> Vec<(usize, String)> {
    let mut depth = 0;
    let mut to_print = Vec::new();
    to_print.push((depth, root));
    let mut strings = Vec::new();

    while to_print.len() > 0 {
        // get children
        let (tag_depth, tag_index) = to_print.pop().unwrap();
        let tag = &map[&tag_index];
        if let Some(children) = &tag.children {
            for i in 0..children.len() {
                let rev_i = children.len() - 1 - i;
                to_print.push((tag_depth + 1, children[rev_i]));
            }
        }

        let mut content_string = "".to_string();
        if let Some(content) = &tag.content {
            content_string = format!(": {}", content);
        }

        let tag_string = format!("{:?}{}", tag.tag_type, content_string);
        strings.push((tag_depth, tag_string));
    } 
    strings
}