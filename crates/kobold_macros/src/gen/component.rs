use proc_macro::TokenStream;

use crate::dom::{Component, Property};
use crate::gen::{DomNode, Field, Generator, IntoGenerator, TokenStreamExt};
use crate::tokenize::prelude::*;

impl Component {
    fn into_expression(self) -> TokenStream {
        let mut stream = self.path;

        if let Some(generics) = self.generics {
            stream.write(("::<", generics, '>'));
        }

        let mut body = None;

        for Property { name, expr } in self.props {
            let body = body.get_or_insert_with(TokenStream::new);

            body.write((name, ':', expr.stream, ','));
        }

        if let Some(spread) = self.spread {
            let body = body.get_or_insert_with(TokenStream::new);

            body.write(("..", spread.stream));
        }

        if let Some(body) = body {
            stream.write(block(body));
        }

        stream.write(".render()");
        stream
    }
}

impl IntoGenerator for Component {
    fn into_gen(self, gen: &mut Generator) -> DomNode {
        let name = gen.names.next();
        let value = self.into_expression();

        gen.out.fields.push(Field::Html { name, value });

        DomNode::Variable(name)
    }
}
