// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tokens::TokenStream;

use crate::dom::{Component, Property};
use crate::gener::{DomNode, Field, Generator, IntoGenerator, TokenStreamExt};
use crate::tokenize::prelude::*;

impl Component {
    fn into_expression(self) -> TokenStream {
        let mut render = self.path.clone();

        render.write("::render");

        if let Some(generics) = self.generics {
            render.write(("::", generics));
        }

        let mut params = self.path;

        params.write("::props()");

        for Property { name, expr } in self.props {
            params.write(('.', call(name, expr.stream)));
        }

        if let Some(children) = self.children {
            let children = crate::gener::generate(children);

            params.write(('.', call("children", children)));
        }

        call(render, params)
    }
}

impl IntoGenerator for Component {
    fn into_generator(self, gener: &mut Generator) -> DomNode {
        let name = gener.names.next();
        let value = self.into_expression();

        gener.out.fields.push(Field::new(name, value));

        DomNode::Variable(name)
    }
}
