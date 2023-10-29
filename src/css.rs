use crate::parser::Parser;

/// Represents a CSS stylesheet, which contains a list of rules.
pub struct Stylesheet {
    rules: Vec<Rule>,
}

/// A CSS rule containing a list of selectors and declarations.
#[derive(Clone)]
struct Rule {
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>,
}

/// Represents a CSS selector.
#[derive(Clone)]
enum Selector {
    /// A simple CSS selector.
    Simple(SimpleSelector),
}

/// A struct representing a simple CSS selector.
#[derive(Clone)]
struct SimpleSelector {
    tag_name: Option<String>,
    id: Option<String>,
    class: Vec<String>,
}

/// A struct representing a CSS declaration, consisting of a name and a value.
#[derive(Clone)]
struct Declaration {
    name: String,
    value: Value,
}

/// An enum representing different types of CSS values.
#[derive(Clone, PartialEq, Debug)]
enum Value {
    /// A keyword value, represented as a string.
    Keyword(String),
    /// A length value, represented as a float and a unit.
    Length(f32, Unit),
    /// A color value, represented as a `Color` struct.
    ColorValue(Color),
}

/// An enum representing different units of measurement used in CSS.
#[derive(Clone, PartialEq, Debug)]
enum Unit {
    Px,
}

/// A struct representing a color with red, green, blue, and alpha channels.
#[derive(Clone, PartialEq, Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

/// A parser for CSS files.
struct CssParser {
    position: usize,
    input: String,
}

/// A tuple representing the specificity of a CSS selector.
/// The tuple contains three values representing the number of ID selectors,
/// class selectors, and element selectors in the selector, respectively.
pub type Specificity = (usize, usize, usize);

/// Implementation of Parser trait for CssParser.
impl Parser for CssParser {
    /// Returns the current position of the parser.
    fn current_position(&self) -> usize {
        self.position
    }

    /// Returns the input string being parsed.
    fn input(&self) -> &str {
        &self.input
    }

    /// Sets the current position of the parser.
    fn set_current_position(&mut self, position: usize) {
        self.position = position;
    }
}

impl CssParser {
    /// Parses the CSS rules and returns a vector of `Rule`s.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the `Css` struct.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Rule>, &str>` - A `Result` containing a vector of `Rule`s if parsing is successful,
    /// or an error message if parsing fails.
    ///
    /// # Examples
    ///
    /// TODO: Fix example.
    /// ```
    /// let mut css = Css::new("body { background-color: red; }");
    /// let rules = css.parse_rules().unwrap();
    /// assert_eq!(rules.len(), 1);
    /// ```
    fn parse_rules(&mut self) -> Result<Vec<Rule>, &str> {
        let mut rules = Vec::new();
        loop {
            self.consume_whitespace()?;
            if self.eof() {
                break;
            }
            rules.push(self.parse_rule().unwrap());
        }
        Ok(rules)
    }

    /// Parses a simple CSS selector and returns a `SimpleSelector` struct.
    ///
    /// This function reads the input string character by character and constructs a `SimpleSelector`
    /// struct based on the characters it reads. It looks for a tag name, an ID, and any number of
    /// classes in the input string, and constructs a `SimpleSelector` struct with those values.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `SimpleSelector` struct if parsing was successful, or an
    /// error message if parsing failed.
    fn parse_simple_selector(&mut self) -> Result<SimpleSelector, &str> {
        let mut selector = SimpleSelector {
            tag_name: None,
            id: None,
            class: Vec::new(),
        };

        while !self.eof() {
            match self.next_char() {
                Ok('#') => {
                    self.consume_char()?;
                    selector.id = Some(self.parse_identifier().unwrap());
                }
                Ok('.') => {
                    self.consume_char()?;
                    selector.class.push(self.parse_identifier().unwrap());
                }
                Ok('*') => {
                    self.consume_char()?;
                }
                Ok(c) if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier().unwrap());
                }
                _ => break,
            }
        }

        Ok(selector)
    }

    /// Parses a list of CSS declarations.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the `CssParser` instance.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `Declaration` instances if parsing is successful, otherwise an error message.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut parser = CssParser::new("body { background-color: red; }");
    /// let declarations = parser.parse_declarations().unwrap();
    /// assert_eq!(declarations.len(), 1);
    /// ```
    fn parse_declarations(&mut self) -> Result<Vec<Declaration>, &str> {
        assert_eq!(self.consume_char()?, '{');
        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace()?;
            if self.next_char()? == '}' {
                self.consume_char()?;
                break;
            }
            declarations.push(self.parse_declaration().unwrap());
        }
        Ok(declarations)
    }

    /// Parses an identifier from the input stream.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the parsed identifier as a `String` if successful, or an error message as a `&str` if unsuccessful.
    fn parse_identifier(&mut self) -> Result<String, &str> {
        self.consume_while(valid_identifier_char)
    }

    /// Parses a CSS rule and returns a `Result` containing a `Rule` struct or an error message.
    fn parse_rule(&mut self) -> Result<Rule, &str> {
        Ok(Rule {
            selectors: self.parse_selectors().unwrap(),
            declarations: self.parse_declarations()?,
        })
    }

    /// Parses a CSS value from the input stream.
    ///
    /// Returns a `Result` containing the parsed `Value` or an error message.
    fn parse_value(&mut self) -> Result<Value, &str> {
        match self.next_char() {
            Ok('0'..='9') => Ok(self.parse_length()?),
            Ok('#') => Ok(self.parse_color()?),
            _ => Ok(Value::Keyword(self.parse_identifier()?)),
        }
    }

    /// Parses a length value and returns a `Value` enum variant containing the parsed length value.
    ///
    /// # Arguments
    ///
    /// * `self` - a mutable reference to the `CssParser` instance.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `Value` enum variant with the parsed length value if successful,
    /// otherwise returns an error message as a `&str`.
    fn parse_length(&mut self) -> Result<Value, &str> {
        Ok(Value::Length(
            self.parse_float().unwrap(),
            self.parse_unit()?,
        ))
    }

    /// Parses a float value from the input stream.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the parsed float value if successful, otherwise an error message.
    fn parse_float(&mut self) -> Result<f32, &str> {
        let s = self.consume_while(|c| matches!(c, '0'..='9' | '.'))?;
        Ok(s.parse().unwrap())
    }

    /// Parses a unit from the input string.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the parsed `Unit` if successful, or an error message if the unit is unrecognized.
    fn parse_unit(&mut self) -> Result<Unit, &str> {
        match &*self.parse_identifier()?.to_ascii_lowercase() {
            "px" => Ok(Unit::Px),
            _ => Err("unrecognized unit"),
        }
    }

    /// Parses a color value from a CSS hex code.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the `CssParser` instance.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `Value` enum variant `ColorValue` with the parsed `Color` struct.
    ///
    /// # Errors
    ///
    /// Returns an error if the first character consumed is not `#`.
    fn parse_color(&mut self) -> Result<Value, &str> {
        assert_eq!(self.consume_char()?, '#');
        Ok(Value::ColorValue(Color {
            r: self.parse_hex_pair(),
            g: self.parse_hex_pair(),
            b: self.parse_hex_pair(),
            a: 255,
        }))
    }

    /// Parses a hexadecimal pair from the input string and returns the corresponding u8 value.
    /// Advances the parser's position by 2.
    fn parse_hex_pair(&mut self) -> u8 {
        let s = &self.input[self.position..self.position + 2];
        self.position += 2;
        u8::from_str_radix(s, 16).unwrap()
    }

    /// Parses a CSS declaration and returns a `Declaration` struct.
    ///
    /// # Arguments
    ///
    /// * `self` - The mutable reference to the `CssParser` instance.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Declaration` struct if parsing is successful,
    /// otherwise returns an error message as a `&str`.
    fn parse_declaration(&mut self) -> Result<Declaration, &str> {
        let property_name = self.parse_identifier().unwrap();
        self.consume_whitespace()?;
        assert_eq!(self.consume_char()?, ':');
        self.consume_whitespace()?;
        let value = self.parse_value().unwrap();
        self.consume_whitespace()?;
        assert_eq!(self.consume_char()?, ';');

        Ok(Declaration {
            name: property_name,
            value,
        })
    }

    /// Parses a list of selectors and returns a vector of `Selector`s.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of `Selector`s if parsing is successful,
    /// otherwise returns an error message as a string slice.
    fn parse_selectors(&mut self) -> Result<Vec<Selector>, &str> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Simple(self.parse_simple_selector().unwrap()));
            self.consume_whitespace()?;
            match self.next_char() {
                Ok(',') => {
                    self.consume_char()?;
                    self.consume_whitespace()?;
                }
                Ok('{') | Err(_) => break,
                Ok(c) => panic!("Unexpected character {} in selector list", c),
            }
        }
        selectors.sort_by_key(|b| std::cmp::Reverse(b.specificity()));
        Ok(selectors)
    }
}

/// Parse a whole CSS stylesheet.
pub fn parse(source: String) -> Result<Stylesheet, &'static str> {
    let mut parser = CssParser {
        position: 0,
        input: source,
    };

    Ok(Stylesheet {
        rules: parser.parse_rules().unwrap(),
    })
}

impl Selector {
    fn specificity(&self) -> Specificity {
        let Selector::Simple(ref simple) = *self;
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag_name.iter().count();
        (a, b, c)
    }
}

/// Returns true if the given character is a valid identifier character in CSS.
///
/// # Arguments
///
/// * `c` - A character to check for validity as a CSS identifier character.
///
/// # Examples
///
/// ```
/// assert_eq!(true, valid_identifier_char('a'));
/// assert_eq!(true, valid_identifier_char('-'));
/// assert_eq!(false, valid_identifier_char(' '));
/// ```
fn valid_identifier_char(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => true, // TODO: Include U+00A0 and higher.
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_selector() {
        let mut parser = CssParser {
            position: 0,
            input: String::from("body"),
        };
        let selector = parser.parse_simple_selector().unwrap();
        assert_eq!(selector.tag_name, Some(String::from("body")));
        assert_eq!(selector.id, None);
        assert_eq!(selector.class.len(), 0);

        let mut parser = CssParser {
            position: 0,
            input: String::from("#id"),
        };
        let selector = parser.parse_simple_selector().unwrap();
        assert_eq!(selector.tag_name, None);
        assert_eq!(selector.id, Some(String::from("id")));
        assert_eq!(selector.class.len(), 0);

        let mut parser = CssParser {
            position: 0,
            input: String::from(".class"),
        };
        let selector = parser.parse_simple_selector().unwrap();
        assert_eq!(selector.tag_name, None);
        assert_eq!(selector.id, None);
        assert_eq!(selector.class.len(), 1);
        assert_eq!(selector.class[0], String::from("class"));

        let mut parser = CssParser {
            position: 0,
            input: String::from("body#id.class"),
        };
        let selector = parser.parse_simple_selector().unwrap();
        assert_eq!(selector.tag_name, Some(String::from("body")));
        assert_eq!(selector.id, Some(String::from("id")));
        assert_eq!(selector.class.len(), 1);
        assert_eq!(selector.class[0], String::from("class"));
    }

    #[test]
    fn test_parse_rules() {
        let mut parser = CssParser {
            position: 0,
            input: String::from("body { background-color: red; }"),
        };
        let rules = parser.parse_rules().unwrap();
        assert_eq!(rules.len(), 1);

        let mut parser = CssParser {
            position: 0,
            input: String::from("body { background-color: red; } p { color: #000000; }"),
        };
        let rules = parser.parse_rules().unwrap();
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_parse_declarations() {
        let mut parser = CssParser {
            position: 0,
            input: String::from("{background-color: red;}"),
        };
        let declarations = parser.parse_declarations().unwrap();
        assert_eq!(declarations.len(), 1);

        let mut parser = CssParser {
            position: 0,
            input: String::from("{background-color: red; color: #000000;}"),
        };
        let declarations = parser.parse_declarations().unwrap();
        assert_eq!(declarations.len(), 2);
    }

    #[test]
    fn test_parse_identifier() {
        let mut parser = CssParser {
            position: 0,
            input: String::from("body"),
        };
        let identifier = parser.parse_identifier().unwrap();
        assert_eq!(identifier, String::from("body"));

        let mut parser = CssParser {
            position: 0,
            input: String::from("body#id"),
        };
        let identifier = parser.parse_identifier().unwrap();
        assert_eq!(identifier, String::from("body"));

        let mut parser = CssParser {
            position: 0,
            input: String::from("body#id.class"),
        };
        let identifier = parser.parse_identifier().unwrap();
        assert_eq!(identifier, String::from("body"));
    }

    #[test]
    fn test_parse_rule() {
        let mut parser = CssParser {
            position: 0,
            input: String::from("body { background-color: red; }"),
        };
        let rule = parser.parse_rule().unwrap();
        assert_eq!(rule.selectors.len(), 1);
        assert_eq!(rule.declarations.len(), 1);

        let mut parser = CssParser {
            position: 0,
            input: String::from("body { background-color: red; color: #000000; }"),
        };
        let rule = parser.parse_rule().unwrap();
        assert_eq!(rule.selectors.len(), 1);
        assert_eq!(rule.declarations.len(), 2);
    }

    #[test]
    fn test_parse_value() {
        let mut parser = CssParser {
            position: 0,
            input: String::from("red"),
        };
        let value = parser.parse_value().unwrap();
        assert_eq!(value, Value::Keyword(String::from("red")));

        let mut parser = CssParser {
            position: 0,
            input: String::from("1px"),
        };
        let value = parser.parse_value().unwrap();
        assert_eq!(value, Value::Length(1.0, Unit::Px));

        let mut parser = CssParser {
            position: 0,
            input: String::from("#000000"),
        };
        let value = parser.parse_value().unwrap();
        assert_eq!(
            value,
            Value::ColorValue(Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255
            })
        );
    }

    #[test]
    fn test_parse_length() {
        let mut parser = CssParser {
            position: 0,
            input: String::from("1px"),
        };
        let value = parser.parse_length().unwrap();
        assert_eq!(value, Value::Length(1.0, Unit::Px));
    }

    #[test]
    fn test_parse_float() {
        let mut parser = CssParser {
            position: 0,
            input: String::from("1.0"),
        };
        let value = parser.parse_float().unwrap();
        assert_eq!(value, 1.0);
    }

    #[test]
    fn test_parse_unit() {
        let mut parser = CssParser {
            position: 0,
            input: String::from("px"),
        };
        let value = parser.parse_unit().unwrap();
        assert_eq!(value, Unit::Px);
    }

    #[test]
    fn test_parse_color() {
        let mut parser = CssParser {
            position: 0,
            input: String::from("#000000"),
        };
        let value = parser.parse_color().unwrap();
        assert_eq!(
            value,
            Value::ColorValue(Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255
            })
        );
    }

    #[test]
    fn test_parse_hex_pair() {
        let mut parser = CssParser {
            position: 0,
            input: String::from("00"),
        };
        let value = parser.parse_hex_pair();
        assert_eq!(value, 0);
    }

    #[test]
    fn test_parse_declaration() {
        let mut parser = CssParser {
            position: 0,
            input: String::from("background-color: red;"),
        };
        let declaration = parser.parse_declaration().unwrap();
        assert_eq!(declaration.name, String::from("background-color"));
        assert_eq!(declaration.value, Value::Keyword(String::from("red")));

        let mut parser = CssParser {
            position: 0,
            input: String::from("background-color: red; color: #000000;"),
        };
        let declaration = parser.parse_declaration().unwrap();
        assert_eq!(declaration.name, String::from("background-color"));
        assert_eq!(declaration.value, Value::Keyword(String::from("red")));
    }

    #[test]
    fn test_parse_selectors() {
        let mut parser = CssParser {
            position: 0,
            input: String::from("body"),
        };
        let selectors = parser.parse_selectors().unwrap();
        assert_eq!(selectors.len(), 1);

        let mut parser = CssParser {
            position: 0,
            input: String::from("body, p"),
        };
        let selectors = parser.parse_selectors().unwrap();
        assert_eq!(selectors.len(), 2);

        let mut parser = CssParser {
            position: 0,
            input: String::from("body, p { background-color: red; }"),
        };
        let selectors = parser.parse_selectors().unwrap();
        assert_eq!(selectors.len(), 2);
    }

    #[test]
    fn test_parse() {
        let stylesheet = parse(String::from("body { background-color: red; }")).unwrap();
        assert_eq!(stylesheet.rules.len(), 1);

        let stylesheet = parse(String::from(
            "body { background-color: red; } p { color: #000000; }",
        ))
        .unwrap();
        assert_eq!(stylesheet.rules.len(), 2);
    }

    #[test]
    fn test_valid_identifier_char() {
        assert!(valid_identifier_char('a'));
        assert!(valid_identifier_char('-'));
        assert!(!valid_identifier_char(' '));
    }
}
