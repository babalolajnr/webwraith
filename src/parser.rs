pub trait Parser {
    /// Returns the current position of the parser in the input string.
    fn current_position(&self) -> usize;

    /// Returns a reference to the input string being parsed.
    fn input(&self) -> &str;

    /// Sets the current position of the parser in the input string.
    fn set_current_position(&mut self, position: usize);

    /// Returns the next character in the input string.
    ///
    /// If there are no more characters in the input string, an error is returned.
    /// Otherwise, the next character is returned.
    fn next_char(&self) -> Result<char, &'static str> {
        if self.eof() {
            return Err("No more characters in input string");
        }

        self.input()[self.current_position()..]
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
    fn starts_with(&self, s: &[u8]) -> Result<bool, &'static str> {
        if self.eof() {
            return Err("No more characters in input string");
        }

        Ok(self.input()[self.current_position()..]
            .as_bytes()
            .starts_with(s))
    }

    /// Returns true if the current position is at the end of the input.
    fn eof(&self) -> bool {
        self.current_position() >= self.input().len()
    }

    /// Consumes the next character in the input string and returns it.
    ///
    /// # Errors
    ///
    /// Returns an error if there are no more characters to consume.
    ///
    /// # Returns
    ///
    /// Returns the next character in the input string.
    ///
    fn consume_char(&mut self) -> Result<char, &'static str> {
        let input_slice = &self.input()[self.current_position()..];

        if input_slice.is_empty() {
            return Err("No more characters in the input string");
        }

        let current_char = input_slice.chars().next().unwrap();
        let next_position = self.current_position() + current_char.len_utf8();
        self.set_current_position(next_position);

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
                Err(_) => break,
            }
        }

        Ok(result)
    }

    /// Consumes all whitespace characters from the input stream until a non-whitespace character is encountered.
    fn consume_whitespace(&mut self) -> Result<(), &'static str> {
        self.consume_while(char::is_whitespace)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ParserImplementor {
        current_position: usize,
        input: String,
    }

    impl Parser for ParserImplementor {
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

    #[test]
    fn test_current_position() {
        let parser = ParserImplementor {
            current_position: 0,
            input: String::from("hello"),
        };

        assert_eq!(parser.current_position(), 0);
    }

    #[test]
    fn test_input() {
        let parser = ParserImplementor {
            current_position: 0,
            input: String::from("hello"),
        };

        assert_eq!(parser.input(), "hello");
    }

    #[test]
    fn test_set_current_position() {
        let mut parser = ParserImplementor {
            current_position: 0,
            input: String::from("hello"),
        };

        parser.set_current_position(1);
        assert_eq!(parser.current_position(), 1);

        parser.set_current_position(5);
        assert_eq!(parser.current_position(), 5);
    }

    #[test]
    fn test_next_char() {
        let mut parser = ParserImplementor {
            current_position: 0,
            input: String::from("hello"),
        };

        assert_eq!(parser.next_char(), Ok('h'));

        parser.current_position = 1;
        assert_eq!(parser.next_char(), Ok('e'));

        parser.current_position = 5;
        assert_eq!(
            parser.next_char(),
            Err("No more characters in input string")
        );
    }

    #[test]
    fn test_starts_with() {
        let mut parser = ParserImplementor {
            current_position: 0,
            input: String::from("hello"),
        };

        assert!(parser.starts_with(b"hello").unwrap());

        parser.current_position = 1;
        assert!(parser.starts_with(b"ello").unwrap());

        parser.current_position = 5;
        assert!(parser.starts_with(b"hello").is_err());

        parser.current_position = 6;
        assert!(parser.starts_with(b"hello").is_err());
    }

    #[test]
    fn test_eof() {
        let mut parser = ParserImplementor {
            current_position: 0,
            input: String::from("hello"),
        };

        assert!(!parser.eof());

        parser.current_position = 5;
        assert!(parser.eof());
    }

    #[test]
    fn test_consume_char() {
        let mut parser = ParserImplementor {
            current_position: 0,
            input: String::from("hello"),
        };

        assert_eq!(parser.consume_char(), Ok('h'));
        assert_eq!(parser.current_position(), 1);

        assert_eq!(parser.consume_char(), Ok('e'));
        assert_eq!(parser.current_position(), 2);

        assert_eq!(parser.consume_char(), Ok('l'));
        assert_eq!(parser.current_position(), 3);

        assert_eq!(parser.consume_char(), Ok('l'));
        assert_eq!(parser.current_position(), 4);

        assert_eq!(parser.consume_char(), Ok('o'));
        assert_eq!(parser.current_position(), 5);

        assert!(parser.consume_char().is_err());
        assert_eq!(parser.current_position(), 5);
    }

    #[test]
    fn test_consume_while() {
        let mut parser = ParserImplementor {
            current_position: 0,
            input: String::from("123abc"),
        };

        assert_eq!(
            parser.consume_while(|c| c.is_numeric()),
            Ok(String::from("123"))
        );
        assert_eq!(parser.current_position, 3);

        assert_eq!(
            parser.consume_while(|c| c.is_alphabetic()),
            Ok(String::from("abc"))
        );
        assert_eq!(parser.current_position, 6);

        assert_eq!(
            parser.consume_while(|c| c.is_alphanumeric()),
            Ok(String::from(""))
        );
        assert_eq!(parser.current_position, 6);
    }

    #[test]
    fn test_consume_whitespace() {
        let mut parser = ParserImplementor {
            current_position: 0,
            input: String::from("  \t\n\r"),
        };

        assert_eq!(parser.consume_whitespace(), Ok(()));
        assert_eq!(parser.current_position, 5);

        assert_eq!(parser.consume_whitespace(), Ok(()));
        assert_eq!(parser.current_position, 5);
    }
}
