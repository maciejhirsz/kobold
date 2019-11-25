use std::mem::replace;
use proc_macro::{TokenStream, TokenTree, Ident, Span};
use proc_macro2::TokenStream as QuoteTokens;
use quote::quote;

struct IdentFactory {
    prefix: char,
    current: usize,
}

impl IdentFactory {
    fn new(prefix: char) -> Self {
        Self {
            prefix,
            current: 0,
        }
    }

    fn next(&mut self) -> QuoteTokens {
        let ident = format!("{}{}", self.prefix, self.current);
        let ident = Ident::new(&ident, Span::call_site());

        self.current += 1;

        TokenStream::from(TokenTree::Ident(ident)).into()
    }
}

pub struct Generator {
    prerender: QuoteTokens,
    render: QuoteTokens,
    update: QuoteTokens,
    stack: Vec<QuoteTokens>,
    var_factory: IdentFactory,
    field_factory: IdentFactory,
    current_field: QuoteTokens,
    current_field_js: QuoteTokens,
    fields: Vec<QuoteTokens>,
}

impl Generator {
    pub fn new() -> Self {
        let mut field_factory = IdentFactory::new('f');
        let current_field = field_factory.next();
        let current_field_js = quote! {
            @{ self.#current_field.js() }
        };

        Generator {
            prerender: QuoteTokens::new(),
            render: QuoteTokens::new(),
            update: QuoteTokens::new(),
            stack: Vec::new(),
            var_factory: IdentFactory::new('e'),
            field_factory,
            current_field,
            current_field_js,
            fields: Vec::new(),
        }
    }

    pub fn var(&mut self) -> QuoteTokens {
        self.var_factory.next()
    }

    pub fn extend(&mut self, tokens: QuoteTokens) {
        self.render.extend(tokens);
    }

    pub fn add<G: Generate>(&mut self, item: G) -> QuoteTokens {
        if let Some(update) = item.computed(&self.current_field_js) {
            self.update.extend(update);
            self.push_stack();

            let el = item.generate(self);
            let tokens = self.pop_stack();
            let field = &self.current_field;

            self.prerender.extend(quote! {
                let #field = to_node(stdweb::js! {
                    #tokens

                    return #el;
                });
            });

            let tokens = quote!{
                @{ #field.js() }
            };

            let field = self.field_factory.next();

            self.current_field_js = quote! {
                @{ self.#field.js() }
            };

            self.fields.push(replace(&mut self.current_field, field));

            tokens
        } else {
            item.generate(self)
        }
    }

    pub fn tokens(&self) -> (&QuoteTokens, &QuoteTokens, &QuoteTokens) {
        (&self.prerender, &self.render, &self.update)
    }

    pub fn fields(&self) -> &[QuoteTokens] {
        &self.fields
    }

    fn push_stack(&mut self) {
        self.stack.push(replace(&mut self.render, QuoteTokens::new()));
    }

    fn pop_stack(&mut self) -> QuoteTokens {
        replace(
            &mut self.render,
            self.stack.pop().expect("Unexpected empty stack"),
        )
    }
}

pub trait Generate {
    fn generate(&self, gen: &mut Generator) -> QuoteTokens;

    /// `None` if node is completely static HTML.
    ///
    /// Otherwise produce token stream for javascript code needed to update
    /// the element, using `el` as the reference to the element.
    fn computed(&self, el: &QuoteTokens) -> Option<QuoteTokens> {
        None
    }
}

impl<'a, T: Generate> Generate for &'a T {
    fn generate(&self, gen: &mut Generator) -> QuoteTokens {
        (**self).generate(gen)
    }

    fn computed(&self, field: &QuoteTokens) -> Option<QuoteTokens> {
        (**self).computed(field)
    }
}
