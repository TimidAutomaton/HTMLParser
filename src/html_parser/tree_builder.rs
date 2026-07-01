
use std::{collections::HashMap, hash::Hash};

use crate::html_parser::{token_parser::{HTMLTagToken, TagPairType, TagType}};


#[derive(Clone)]
pub struct HTMLTag {
    pub index: usize,
    pub tag_type: TagType,
    pub content: Option<String>,
    pub attributes: Option<HashMap<String, String>>,

    pub id: usize,
    pub parent: Option<usize>,
    pub children: Option<Vec<usize>>,
}


pub struct HTMLTreeBuilder {
    current_index: usize,

    current_parent: usize,
    current_tag: usize,
    tag_list: HashMap<usize, HTMLTag>
}

impl HTMLTreeBuilder {
    pub fn new() -> Self {
        Self {
            current_index: 0,

            current_parent: 0,
            current_tag: 0,
            tag_list: HashMap::new(),
        }
    }

    pub fn build_tree(&mut self, tag_token_list: &Vec<HTMLTagToken>) -> HashMap<usize, HTMLTag>{
        
        // create root node
        // match tag
        // if open

        let mut root = HTMLTag {
            index: 0,
            tag_type: TagType::Root,
            content: None,
            attributes: None,
            id: self.create_tag_id(),
            parent: None,
            children: None,
        };

        self.tag_list.insert(root.id, root);

        loop {
            let t = &tag_token_list[self.current_index];
            if t.tag_type == TagType::EOF {
                let eof_tag = HTMLTag {
                    index: t.char_index,
                    tag_type: t.tag_type.clone(),
                    content: None,
                    attributes: None,
                    id: self.create_tag_id(),
                    parent: None,
                    children: None,
                };
                self.tag_list.insert(eof_tag.id, eof_tag);
                break;
            }
            else if t.tag_type == TagType::DOCTYPE {
                self.advance_tag(tag_token_list);
                println!("I still don't understand DOCTYPE");
            }
            else if t.pair_type == TagPairType::Opening {
                let mut new_tag = HTMLTag {
                    index: t.char_index,
                    tag_type: t.tag_type.clone(),
                    content: None,
                    attributes: None,
                    id: self.create_tag_id(),
                    parent: None,
                    children: None,
                };
                let tag_id = new_tag.id;
                self.add_child(self.current_parent, &mut new_tag);
                self.tag_list.insert(new_tag.id, new_tag);
                self.current_parent = tag_id;
                self.advance_tag(tag_token_list);
            }
            else if t.pair_type == TagPairType::Closing {
                self.current_parent = self.tag_list[&self.current_parent].parent.unwrap();
                self.advance_tag(tag_token_list);
            }
            else if t.pair_type == TagPairType::Content {
                let mut parent = self.tag_list.remove(&self.current_parent).unwrap();
                parent.content = t.content.clone();
                self.tag_list.insert(parent.id, parent);
                self.advance_tag(tag_token_list);
            }
        }

        println!("End of file!");
        println!("");


        let mut to_print = Vec::new();

        let root_children = self.tag_list[&0].children.as_ref().unwrap();
        for i in 0..root_children.len() {
            let end = root_children.len() - 1;
            to_print.push(root_children[end - i]);
        }

        println!("Root");
        while to_print.len() > 0 {
            let current = to_print.pop().unwrap();
            let tag = self.tag_list[&current].clone();
            println!("tag: {:?}", tag.tag_type);
            if let Some(children) = tag.children {
                for i in 0..children.len() {
                    let end = children.len() - 1;
                    to_print.push(children[end - i]);
                }
            }
        }

        self.tag_list.clone()
    }  

    fn add_child(&mut self, parent: usize, child_node: &mut HTMLTag) {
        let mut parent_node = self.tag_list.remove(&parent).unwrap();
        if let Some(mut children) = parent_node.children.as_mut() {
            children.push(child_node.id);
        }
        else {
            let mut children = Vec::new();
            children.push(child_node.id);
            parent_node.children = Some(children);
        }
        child_node.parent = Some(parent);
        self.tag_list.insert(parent, parent_node);
    }

    fn create_tag_id(&mut self) -> usize {
        self.current_tag += 1;
        self.current_tag - 1 
    }

    pub fn is_single_tag(&self, token: &TagType) -> bool {
        if *token == TagType::Img {
            return true
        }
        false
    }

    pub fn advance_tag(&mut self, tag_list: &Vec<HTMLTagToken>) -> HTMLTagToken {
        self.current_index += 1;
        if self.current_index >= tag_list.len() {
            self.current_index = tag_list.len() - 1;
        }
        tag_list[self.current_index].clone()
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