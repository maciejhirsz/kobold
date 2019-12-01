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
    var_factory: IdentFactory,
    field_factory: IdentFactory,
    fields: Vec<QuoteTokens>,
}

impl Generator {
    pub fn new() -> Self {
        Generator {
            prerender: QuoteTokens::new(),
            render: QuoteTokens::new(),
            update: QuoteTokens::new(),
            var_factory: IdentFactory::new('e'),
            field_factory: IdentFactory::new('f'),
            fields: Vec::new(),
        }
    }

    pub fn var(&mut self) -> QuoteTokens {
        self.var_factory.next()
    }

    pub fn extend(&mut self, tokens: QuoteTokens) {
        self.render.extend(tokens);
    }

    pub fn add_field(&mut self, expr: &QuoteTokens) -> QuoteTokens {
        let field = self.field_factory.next();
        let node = quote!{
            @{ #field.js() }
        };

        self.prerender.extend(quote! {
            let #field = { #expr }.render();
        });

        self.update.extend(quote! {
            { #expr }.update(&self.#field);
        });

        self.fields.push(field);

        node
    }

    pub fn add<G: Generate>(&mut self, item: G) -> QuoteTokens {
        item.generate(self)
    }

    pub fn tokens(&self) -> (&QuoteTokens, &QuoteTokens, &QuoteTokens) {
        (&self.prerender, &self.render, &self.update)
    }

    pub fn fields(&self) -> &[QuoteTokens] {
        &self.fields
    }
}

pub trait Generate {
    fn generate(&self, gen: &mut Generator) -> QuoteTokens;
}

impl<'a, T: Generate> Generate for &'a T {
    fn generate(&self, gen: &mut Generator) -> QuoteTokens {
        (**self).generate(gen)
    }
}
