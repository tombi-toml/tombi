pub struct Container<'a> {
    tokens: Vec<syntax::Token>,
    pub builder: &'a mut rowan::GreenNodeBuilder<'static>,
}

impl<'a> Container<'a> {
    #[allow(dead_code)]
    pub fn new(builder: &'a mut rowan::GreenNodeBuilder<'static>) -> Self {
        Self {
            tokens: vec![],
            builder,
        }
    }

    #[allow(dead_code)]
    pub fn push(&mut self, token: syntax::Token) {
        self.builder.start_node(token.into());
        self.tokens.push(token);
    }

    pub fn pop(&mut self) -> Option<syntax::Token> {
        if let Some(token) = self.tokens.pop() {
            self.builder.finish_node();
            Some(token)
        } else {
            None
        }
    }
}

impl<'a> Drop for Container<'a> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syntax::Token;

    #[test]
    fn test_container() {
        let green_node = {
            let mut builder = rowan::GreenNodeBuilder::default();
            {
                let mut container = Container::new(&mut builder);
                container.push(Token::ROOT);
                container.builder.token(Token::COMMENT.into(), "comment");
                container.builder.token(Token::NEWLINE.into(), "newline");
            }
            builder.finish()
        };

        println!("{:?}", green_node.children());
        assert!(green_node.children().len() == 2);
    }
}
