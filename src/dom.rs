use std::collections::HashMap;

/// Represents a node in the Document Object Model (DOM).
#[derive(Debug)]
pub struct Node {
    /// The child nodes of this node.
    pub children: Vec<Node>,
    /// The type of this node.
    pub node_type: NodeType,
}

/// Represents the type of a node in the DOM tree.
#[derive(Debug)]
pub enum NodeType {
    /// A text node containing a string.
    Text(String),
    /// An element node containing element data.
    Element(ElementData),
    // A comment node containing a string.
    Comment(String),
}

/// Represents the data associated with an HTML element.
#[derive(Debug)]
pub struct ElementData {
    /// The name of the HTML tag.
    pub tag_name: String,
    /// A map of attributes associated with the HTML element.
    pub attributes: AttrMap,
}

pub type AttrMap = HashMap<String, String>;

/// Creates a new text node with the given data.
pub fn text(data: String) -> Node {
    Node {
        children: Vec::new(),
        node_type: NodeType::Text(data),
    }
}

/// Creates a new element node with the given tag name and attributes.
pub fn elem(name: String, attrs: AttrMap, children: Vec<Node>) -> Node {
    Node {
        children,
        node_type: NodeType::Element(ElementData {
            tag_name: name,
            attributes: attrs,
        }),
    }
}

/// Creates a new comment node with the given data.
pub fn comment(data: String) -> Node {
    Node {
        children: Vec::new(),
        node_type: NodeType::Comment(data),
    }
}

impl Node {
    /// Pretty-prints the DOM tree rooted at this node.
    pub fn pretty_print(&self) {
        self.pretty_print_helper(0);
    }

    /// Helper function for pretty-printing the DOM tree.
    fn pretty_print_helper(&self, indent: usize) {
        match &self.node_type {
            NodeType::Text(text) => {
                println!("{}{}", " ".repeat(indent), text);
            }
            NodeType::Element(elem_data) => {
                println!("{}<{}>", " ".repeat(indent), elem_data.tag_name);
                for child in &self.children {
                    child.pretty_print_helper(indent + 2);
                }
                println!("{}<\\{}>", " ".repeat(indent), elem_data.tag_name);
            }
            NodeType::Comment(comment) => {
                println!("{}<!-- {} -->", " ".repeat(indent), comment);
            }
        }
    }
}

