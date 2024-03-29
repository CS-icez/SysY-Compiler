//! Token generator for generating unique tokens with the same prefix.

pub struct TokenGenerator {
    prefix: String,
    counter: u32,
}

impl TokenGenerator {
    /// Create a new token generator.
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            counter: 0,
        }
    }

    /// Resets generator state.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.counter = 0;
    }

    /// Returns a new token.
    pub fn generate(&mut self) -> String {
        let cur = self.counter;
        self.counter += 1;
        self.prefix.to_string() + &cur.to_string()
    }

    /// Peeks the next token to be generated, i.e.,
    /// what will be returned if `generate` is called.
    pub fn peek(&self) -> String {
        let next = self.counter;
        self.prefix.to_string() + &next.to_string()
    }
}
