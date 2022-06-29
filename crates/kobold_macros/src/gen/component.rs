use proc_macro::TokenStream;

use crate::dom::{Component, Property};
use crate::gen::{DomNode, Field, Generator, IntoGenerator, TokenStreamExt};
use crate::tokenize::prelude::*;

impl Component {
    fn into_expression(self) -> TokenStream {
        let mut stream = self.path;

        if let Some(generics) = self.generics {
            stream.write(("::", generics));
        }

        let mut props = None;

        for Property { name, expr } in self.props {
            let props = props.get_or_insert_with(TokenStream::new);

            props.write((name, ':', expr.stream, ','));
        }

        if let Some(spread) = self.spread {
            let props = props.get_or_insert_with(TokenStream::new);

            props.write(("..", spread.stream));
        }

        if let Some(props) = props {
            stream.write(block(props));
        }

        if let Some(children) = self.children {
            let children = crate::gen::generate(children);

            stream.write(call(".render_with", children));
        } else {
            stream.write(".render()");
        }

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
