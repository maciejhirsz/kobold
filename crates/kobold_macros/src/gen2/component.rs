use proc_macro::{Delimiter, Group, TokenStream};

use crate::dom2::{Component, Property};
use crate::gen2::{DomNode, Field, Generator, IntoGenerator};
use crate::parse::TokenStreamExt;

impl Component {
    fn into_expression(self) -> TokenStream {
        let mut stream = self.path;

        if let Some(generics) = self.generics {
            stream.write("::<");
            stream.extend(generics);
            stream.write(">");
        }

        let mut body = None;

        for Property { name, expr } in self.props {
            let body = body.get_or_insert_with(TokenStream::new);

            body.push(name);
            body.write(":");
            body.extend(expr.stream);
            body.write(",");
        }

        if let Some(spread) = self.spread {
            let body = body.get_or_insert_with(TokenStream::new);

            body.write("..");
            body.extend(spread.stream);
        }

        if let Some(body) = body {
            stream.push(Group::new(Delimiter::Brace, body));
        }

        stream
    }
}

impl IntoGenerator for Component {
    fn into_gen(self, gen: &mut Generator) -> DomNode {
        let name = gen.names.next();
        let value = self.into_expression();

        let var = gen.out.add(Field::Html { name, value });

        DomNode::Variable(*var)
    }
}
