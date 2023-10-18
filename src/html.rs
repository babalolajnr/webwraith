use std::collections::HashMap;

use crate::dom::{elem, text, AttrMap, Node};

/// A struct representing a parser for HTML.
struct Parser {
    /// The current position of the parser.
    current_position: usize,
    /// The input string being parsed.
    input: String,
}

impl Parser {
    /// Returns the next character in the input string.
    ///
    /// If there are no more characters in the input string, an error is returned.
    /// Otherwise, the next character is returned.
    fn next_char(&self) -> Result<char, &'static str> {
        if self.current_position >= self.input.len() {
            return Err("No more characters in input string");
        }

        self.input[self.current_position..]
            .chars()
            .next()
            .ok_or("Failed to get next character")
    }

    /// Checks if the input starting from the current position matches the given byte slice.
    ///
    /// # Arguments
    ///
    /// * `s` - A byte slice to check if it matches the input starting from the current position.
    ///
    /// # Returns
    ///
    /// A boolean value indicating whether the input starting from the current position matches the given byte slice.
    fn starts_with(&self, s: &[u8]) -> bool {
        self.input[self.current_position..]
            .as_bytes()
            .starts_with(s)
    }

    /// Returns true if the current position is at the end of the input.
    fn eof(&self) -> bool {
        self.current_position >= self.input.len()
    }

    /// Consumes the next character in the input string and returns it.
    /// If there are no more characters to consume, returns an error.
    fn consume_char(&mut self) -> Result<char, &'static str> {
        let mut iter = self.input[self.current_position..].char_indices();
        let (_, current_char) = iter.next().ok_or("Failed to get next character")?;
        let (next_position, _) = iter.next().unwrap_or((1, ' '));
        self.current_position += next_position;
        Ok(current_char)
    }

    /// Consumes characters from the input string while the given condition is true.
    /// Returns the consumed characters as a string.
    ///
    /// # Arguments
    ///
    /// * `condition` - A closure that takes a `char` and returns a `bool`.
    ///
    /// # Returns
    ///
    /// * `Ok(result)` - A `Result` containing the consumed characters as a `String`.
    /// * `Err("Failed to get next character")` - A `Result` containing an error message if the next character could not be retrieved.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut input = "123abc".chars();
    /// let result = consume_while(&mut input, |c| c.is_numeric());
    /// assert_eq!(result, Ok(String::from("123")));
    /// ```
    fn consume_while<F>(&mut self, condition: F) -> Result<String, &'static str>
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();

        loop {
            match self.next_char() {
                Ok(c) if condition(c) => result.push(self.consume_char()?),
                Ok(_) => break,
                Err(_) => return Err("Failed to get next character"),
            }
        }

        Ok(result)
    }

    /// Consumes all whitespace characters from the input stream until a non-whitespace character is encountered.
    fn consume_whitespace(&mut self) -> Result<(), &'static str> {
        self.consume_while(char::is_whitespace)?;
        Ok(())
    }

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

            if self.next_char()? == '>' {
                break;
            }

            let (name, value) = self.parse_attribute()?;
            attributes.insert(name, value);
        }

        Ok(attributes)
    }

    /// Parses an HTML attribute and returns a tuple containing the attribute name and value.
    ///
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

            if self.eof() || self.starts_with(b"</") {
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
        let mut parser = Parser {
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
