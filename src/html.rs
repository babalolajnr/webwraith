use std::collections::HashMap;

use crate::{
    dom::{elem, text, AttrMap, Node},
    parser::Parser,
};

/// A struct representing a parser for HTML.
#[derive(Debug, PartialEq)]
struct HtmlParser {
    /// The current position of the parser.
    current_position: usize,
    /// The input string being parsed.
    input: String,
}

impl Parser for HtmlParser {
    fn current_position(&self) -> usize {
        self.current_position
    }

    fn input(&self) -> &str {
        &self.input
    }

    fn set_current_position(&mut self, position: usize) {
        self.current_position = position;
    }
}

// TODO: Implement comment parsing
impl HtmlParser {
    /// Parses the tag name from the input stream.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the parsed tag name as a `String` if successful, or an error message
    /// as a `&'static str` if parsing fails.
    fn parse_tag_name(&mut self) -> Result<String, &'static str> {
        self.consume_while(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9'))
    }

    /// Parses the next node in the HTML document.
    ///
    /// Returns a `Node` representing the parsed node on success, or a static string slice
    /// with an error message on failure.
    fn parse_node(&mut self) -> Result<Node, &'static str> {
        match self.next_char() {
            Ok('<') => self.parse_element(),
            Ok(_) => self.parse_text(),
            Err(_) => Err("Failed to get next character"),
        }
    }

    /// Parses the text content of an HTML node.
    ///
    /// # Returns
    ///
    /// Returns a `Node` containing the parsed text content.
    ///
    /// # Errors
    ///
    /// Returns a `&'static str` error if there is an error parsing the text content.
    fn parse_text(&mut self) -> Result<Node, &'static str> {
        Ok(text(self.consume_while(|c| c != '<')?))
    }

    /// Parses an HTML element and returns a `Node` representing it.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the parsed `Node` if successful, or an error message if parsing fails.
    fn parse_element(&mut self) -> Result<Node, &'static str> {
        let (tag_name, attrs) = self.parse_opening_tag()?;
        let tag_name = tag_name.to_ascii_lowercase();

        // Contents
        let children = self.parse_nodes()?;

        let closing_tag_name = self.parse_closing_tag()?;
        let closing_tag_name = closing_tag_name.to_ascii_lowercase();

        if tag_name != closing_tag_name {
            return Err("Opening and closing tag names do not match");
        }

        Ok(elem(tag_name, attrs, children))
    }

    /// Parses an opening tag and returns the tag name and its attributes.
    ///
    /// # Returns
    ///
    /// A tuple containing the tag name and its attributes.
    ///
    /// # Errors
    ///
    /// Returns an error if the opening tag is not valid.
    fn parse_opening_tag(&mut self) -> Result<(String, AttrMap), &'static str> {
        self.consume_char()?; // Consume '<'
        let tag_name = self.parse_tag_name()?;
        let attrs = self.parse_attributes()?;
        self.consume_char()?; // Consume '>'

        Ok((tag_name, attrs))
    }

    /// Parses a closing HTML tag and returns the tag name.
    ///
    /// # Arguments
    ///
    /// None
    ///
    /// # Returns
    ///
    /// Returns the tag name if successful, otherwise an error message.
    ///
    /// # Examples
    /// TODO: Fix this example
    ///
    /// ```
    /// let mut parser = HtmlParser::new();
    /// let result = parser.parse_closing_tag();
    /// assert_eq!(result, Ok("div".to_string()));
    /// ```
    fn parse_closing_tag(&mut self) -> Result<String, &'static str> {
        self.consume_char()?; // Consume '<'
        self.consume_char()?; // Consume '/'
        let tag_name = self.parse_tag_name()?;
        self.consume_char()?; // Consume '>'

        Ok(tag_name)
    }

    /// Parses the attributes of an HTML element and returns a map of attribute names to values.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `AttrMap` if parsing was successful, or a `&'static str`
    /// error message if an error occurred.
    ///
    /// # Examples
    ///
    /// TODO: Fix this example
    /// ```
    /// use std::collections::HashMap;
    /// use webwraith::html::AttrMap;
    ///
    /// let mut parser = Parser::new("<div class=\"example\" id=\"main\">");
    /// let attributes = parser.parse_attributes().unwrap();
    ///
    /// let mut expected = HashMap::new();
    /// expected.insert("class".to_string(), "example".to_string());
    /// expected.insert("id".to_string(), "main".to_string());
    ///
    /// assert_eq!(attributes, expected);
    /// ```
    fn parse_attributes(&mut self) -> Result<AttrMap, &'static str> {
        let mut attributes = HashMap::new();

        loop {
            self.consume_whitespace()?;

            if self.eof() || self.next_char()? == '>' || self.next_char()? == '/' {
                break;
            }

            let (name, value) = self.parse_attribute()?;
            attributes.insert(name, value);
        }

        Ok(attributes)
    }

    /// Parses an HTML attribute and returns a tuple containing the attribute name and value.
    ///
    /// TODO: Parse attributes with no value (e.g. `<input disabled>`)
    /// TODO: Parse attributes with no quotes (e.g. `<input type=text>`)
    /// TODO: Parse attributes with multiple values (e.g. `<input class="form-input bg-green">`)
    /// # Returns
    ///
    /// A tuple containing the attribute name and value as strings.
    ///
    /// # Errors
    ///
    /// Returns an error if the attribute name or value cannot be parsed.
    fn parse_attribute(&mut self) -> Result<(String, String), &'static str> {
        let name = self.parse_tag_name()?;
        self.consume_whitespace()?;
        self.consume_char()?; // Consume '='
        self.consume_whitespace()?;
        let value = self.parse_attr_value()?;
        Ok((name, value))
    }

    /// Parses the value of an HTML attribute.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the parsed attribute value as a `String` if successful,
    /// or a `&'static str` error message if unsuccessful.
    fn parse_attr_value(&mut self) -> Result<String, &'static str> {
        let open_quote = self.consume_char()?;
        let value = self.consume_while(|c| c != open_quote)?;
        self.consume_char()?; // Consume closing quote
        Ok(value)
    }

    /// Parses a sequence of nodes from the input string.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec` of `Node`s if parsing is successful, or an error message if parsing fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut parser = HtmlParser::new("<html><body><h1>Hello, world!</h1></body></html>");
    /// let nodes = parser.parse_nodes().unwrap();
    /// assert_eq!(nodes.len(), 1);
    /// assert_eq!(nodes[0].name, "html");
    /// ```
    fn parse_nodes(&mut self) -> Result<Vec<Node>, &'static str> {
        let mut nodes = Vec::new();

        loop {
            self.consume_whitespace()?;

            if self.eof() || self.starts_with(b"</")? {
                break;
            }

            nodes.push(self.parse_node()?);
        }

        Ok(nodes)
    }

    /// Parses the given HTML source code and returns a `Node` representing the root of the parsed tree.
    ///
    /// # Arguments
    ///
    /// * `source` - A `String` containing the HTML source code to parse.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the root `Node` of the parsed tree if successful, or an error message if parsing failed.
    pub fn parse(source: String) -> Result<Node, &'static str> {
        let mut parser = HtmlParser {
            current_position: 0,
            input: source,
        };

        let mut nodes = parser.parse_nodes()?;

        if nodes.len() == 1 {
            Ok(nodes.remove(0))
        } else {
            Ok(elem("html".to_string(), HashMap::new(), nodes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_parse_tag_name() {
        let mut parser = HtmlParser {
            current_position: 0,
            input: String::from("div"),
        };

        assert_eq!(parser.parse_tag_name(), Ok(String::from("div")));
        assert_eq!(parser.current_position, 3);

        parser.current_position = 0;
        parser.input = String::from("123");
        assert_eq!(parser.parse_tag_name(), Ok(String::from("123")));
        assert_eq!(parser.current_position, 3);

        parser.current_position = 0;
        parser.input = String::from("div123");
        assert_eq!(parser.parse_tag_name(), Ok(String::from("div123")));
        assert_eq!(parser.current_position, 6);

        parser.current_position = 0;
        parser.input = String::from("div-123");
        assert_eq!(parser.parse_tag_name(), Ok(String::from("div")));
        assert_eq!(parser.current_position, 3);

        parser.current_position = 0;
        parser.input = String::from("div_123");
        assert_eq!(parser.parse_tag_name(), Ok(String::from("div")));
        assert_eq!(parser.current_position, 3);

        parser.current_position = 0;
        parser.input = String::from("div.123");
        assert_eq!(parser.parse_tag_name(), Ok(String::from("div")));
        assert_eq!(parser.current_position, 3);

        parser.current_position = 0;
        parser.input = String::from("div#123");
        assert_eq!(parser.parse_tag_name(), Ok(String::from("div")));
        assert_eq!(parser.current_position, 3);
    }

    // #[test]
    // fn test_parse_node() {
    //     let mut parser = HtmlParser {
    //         current_position: 0,
    //         input: String::from("<div>"),
    //     };

    //     assert_eq!(
    //         parser.parse_node(),
    //         Ok(elem("div".to_string(), HashMap::new(), vec![]))
    //     );
    //     assert_eq!(parser.current_position, 5);

    //     parser.current_position = 0;
    //     parser.input = String::from("hello");
    //     assert_eq!(parser.parse_node(), Ok(text("hello".to_string())));
    //     assert_eq!(parser.current_position, 5);
    // }

    #[test]
    fn test_parse_text() {
        let mut parser = HtmlParser {
            current_position: 0,
            input: String::from("hello"),
        };

        assert_eq!(parser.parse_text(), Ok(text("hello".to_string())));
        assert_eq!(parser.current_position, 5);
    }

    #[test]
    fn test_parse_element() {
        let mut parser = HtmlParser {
            current_position: 0,
            input: String::from("<div></div>"),
        };

        assert_eq!(
            parser.parse_element(),
            Ok(elem("div".to_string(), HashMap::new(), vec![]))
        );
        assert_eq!(parser.current_position, 11);

        parser.current_position = 0;
        parser.input = String::from("<div>hello</div>");
        assert_eq!(
            parser.parse_element(),
            Ok(elem(
                "div".to_string(),
                HashMap::new(),
                vec![text("hello".to_string())]
            ))
        );
        assert_eq!(parser.current_position, 16);

        parser.current_position = 0;
        parser.input = String::from("<div class=\"example\"></div>");
        let mut attrs = HashMap::new();
        attrs.insert("class".to_string(), "example".to_string());
        assert_eq!(
            parser.parse_element(),
            Ok(elem("div".to_string(), attrs, vec![]))
        );
        assert_eq!(parser.current_position, 27);

        parser.current_position = 0;
        parser.input = String::from("<div class=\"example\" id=\"main\"></div>");
        let mut attrs = HashMap::new();
        attrs.insert("class".to_string(), "example".to_string());
        attrs.insert("id".to_string(), "main".to_string());
        assert_eq!(
            parser.parse_element(),
            Ok(elem("div".to_string(), attrs, vec![]))
        );
        assert_eq!(parser.current_position, 37);

        parser.current_position = 0;
        parser.input = String::from("<div class=\"example\" id=\"main\">hello</div>");
        let mut attrs = HashMap::new();
        attrs.insert("class".to_string(), "example".to_string());
        attrs.insert("id".to_string(), "main".to_string());
        assert_eq!(
            parser.parse_element(),
            Ok(elem(
                "div".to_string(),
                attrs,
                vec![text("hello".to_string())]
            ))
        );
        assert_eq!(parser.current_position, 42);
    }

    #[test]
    fn test_parse_opening_tag() {
        let mut parser = HtmlParser {
            current_position: 0,
            input: String::from("<div>"),
        };

        assert_eq!(
            parser.parse_opening_tag(),
            Ok(("div".to_string(), HashMap::new()))
        );
        assert_eq!(parser.current_position, 5);

        parser.current_position = 0;
        parser.input = String::from("<div class=\"example\">");
        let mut attrs = HashMap::new();
        attrs.insert("class".to_string(), "example".to_string());
        assert_eq!(parser.parse_opening_tag(), Ok(("div".to_string(), attrs)));
        assert_eq!(parser.current_position, 21);

        parser.current_position = 0;
        parser.input = String::from("<div class=\"example\" id=\"main\">");
        let mut attrs = HashMap::new();
        attrs.insert("class".to_string(), "example".to_string());
        attrs.insert("id".to_string(), "main".to_string());
        assert_eq!(parser.parse_opening_tag(), Ok(("div".to_string(), attrs)));
        assert_eq!(parser.current_position, 31);
    }

    #[test]
    fn test_parse_closing_tag() {
        let mut parser = HtmlParser {
            current_position: 0,
            input: String::from("</div>"),
        };

        assert_eq!(parser.parse_closing_tag(), Ok("div".to_string()));
        assert_eq!(parser.current_position, 6);
    }

    #[test]
    fn test_parse_attributes() {
        let mut parser = HtmlParser {
            current_position: 0,
            input: String::from("class=\"example\""),
        };

        let mut attrs = HashMap::new();
        attrs.insert("class".to_string(), "example".to_string());
        assert_eq!(parser.parse_attributes(), Ok(attrs));
        assert_eq!(parser.current_position, 15);

        parser.current_position = 0;
        parser.input = String::from("class=\"example\" id=\"main\"");
        let mut attrs = HashMap::new();
        attrs.insert("class".to_string(), "example".to_string());
        attrs.insert("id".to_string(), "main".to_string());
        assert_eq!(parser.parse_attributes(), Ok(attrs));
        assert_eq!(parser.current_position, 25);
    }

    #[test]
    fn test_parse_attribute() {
        let mut parser = HtmlParser {
            current_position: 0,
            input: String::from("class=\"example\""),
        };

        assert_eq!(
            parser.parse_attribute(),
            Ok(("class".to_string(), "example".to_string()))
        );
        assert_eq!(parser.current_position, 15);

        parser.current_position = 0;
        parser.input = String::from("class=\"example\" id=\"main\"");
        assert_eq!(
            parser.parse_attribute(),
            Ok(("class".to_string(), "example".to_string()))
        );
        assert_eq!(parser.current_position, 15);
    }

    #[test]
    fn test_parse_attr_value() {
        let mut parser = HtmlParser {
            current_position: 0,
            input: String::from("\"example\""),
        };

        assert_eq!(parser.parse_attr_value(), Ok("example".to_string()));
        assert_eq!(parser.current_position, 9);
    }

    #[test]
    fn test_parse_nodes() {
        let mut parser = HtmlParser {
            current_position: 0,
            input: String::from("<html><body><h1>Hello, world!</h1></body></html>"),
        };

        let nodes = vec![elem(
            "html".to_string(),
            HashMap::new(),
            vec![elem(
                "body".to_string(),
                HashMap::new(),
                vec![elem(
                    "h1".to_string(),
                    HashMap::new(),
                    vec![text("Hello, world!".to_string())],
                )],
            )],
        )];

        assert_eq!(parser.parse_nodes(), Ok(nodes));
        assert_eq!(parser.current_position, 48);
    }

    #[test]
    fn test_parse() {
        let source = String::from("<html><body><h1>Hello, world!</h1></body></html>");

        let nodes = elem(
            "html".to_string(),
            HashMap::new(),
            vec![elem(
                "body".to_string(),
                HashMap::new(),
                vec![elem(
                    "h1".to_string(),
                    HashMap::new(),
                    vec![text("Hello, world!".to_string())],
                )],
            )],
        );

        assert_eq!(HtmlParser::parse(source), Ok(nodes));
    }
}
