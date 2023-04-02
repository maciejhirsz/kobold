macro_rules! enum_map {
    (
        pub static $MAP:ident: [$out:ident] for $enum:ident {
            $($variant:ident => $args:tt,)*
        }
    ) => {
        #[derive(Clone, Copy)]
        pub enum $enum {
            $($variant,)*
        }

        pub static $MAP: [$out; 0 $(+ { let _ = $enum::$variant; 1 })*] = [
            $($out::new $args,)*
        ];
    };
}

pub enum ClosingRules {
    /// Void element, no closing tag, e.g. `<img>` or `<br>`
    Void,
    /// Required closing tag, e.g. `<div>`
    Required,
    /// Optional closing tag plus opening tags that close this if any, e.g. `<li>`
    Optional(&'static [El]),
}

pub struct ElDefinition {
    name: TagStr,
    closing: ClosingRules,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TagStr {
    bytes: [u8; 15],
    len: u8,
}

#[rustfmt::skip]
const fn tagstr<const N: usize>(input: &[u8; N]) -> TagStr {
    let mut bytes = [0u8; 15];

    if 0 < N { bytes[0] = input[0]; }
    if 1 < N { bytes[1] = input[1]; }
    if 2 < N { bytes[2] = input[2]; }
    if 3 < N { bytes[3] = input[3]; }
    if 4 < N { bytes[4] = input[4]; }
    if 5 < N { bytes[5] = input[5]; }
    if 6 < N { bytes[6] = input[6]; }
    if 7 < N { bytes[7] = input[7]; }
    if 8 < N { bytes[8] = input[8]; }
    if 9 < N { bytes[9] = input[9]; }
    if 10 < N { bytes[10] = input[10]; }
    if 11 < N { bytes[11] = input[11]; }
    if 12 < N { bytes[12] = input[12]; }
    if 13 < N { bytes[13] = input[13]; }
    if 14 < N { bytes[14] = input[14]; }

    TagStr {
        bytes,
        len: N as u8,
    }
}

impl ElDefinition {
    const fn new<const N: usize>(name: &'static [u8; N], closing: ClosingRules) -> Self {
        ElDefinition {
            name: tagstr(name),
            closing,
        }
    }
}

use ClosingRules::*;

enum_map! {
    pub static ELS: [ElDefinition] for El {
        Anchor => (b"a", Required),
        Abbr => (b"abbr", Required),
        Address => (b"address", Required),
        Area => (b"area", Void),
        Article => (b"article", Required),
        Aside => (b"aside", Required),
        Audio => (b"audio", Required),
        Bold => (b"b", Required),
        Base => (b"base", Void),
        Bdi => (b"bdi", Required),
        Bdo => (b"bdo", Required),
        Blockquote => (b"blockquote", Required),
        Body => (b"body", Optional(&[])),
        Br => (b"br", Void),
        Button => (b"button", Required),
        Canvas => (b"canvas", Required),
        Caption => (b"caption", Optional(&[])),
        Cite => (b"cite", Required),
        Code => (b"code", Required),
        Col => (b"col", Void),
        ColGroup => (b"colgroup", Optional(&[])),
        Data => (b"data", Required),
        DataList => (b"datalist", Required),
        Dd => (b"dd", Optional(&[El::Dd, El::Dt])),
        Del => (b"del", Required),
        Definition => (b"dfn", Required),
        Details => (b"details", Required),
        Dialog => (b"dialog", Required),
        Div => (b"div", Required),
        Dl => (b"dl", Required),
        Dt => (b"dt", Optional(&[El::Dd, El::Dt])),
        Em => (b"em", Required),
        Embed => (b"embed", Void),
        FieldSet => (b"fieldset", Required),
        FigCaption => (b"figcaption", Required),
        Figure => (b"figure", Required),
        Footer => (b"footer", Required),
        Form => (b"form", Required),
        Head => (b"head", Optional(&[])),
        Header => (b"header", Required),
        Header1 => (b"h1", Required),
        Header2 => (b"h1", Required),
        Header3 => (b"h1", Required),
        Header4 => (b"h1", Required),
        Header5 => (b"h1", Required),
        Header6 => (b"h1", Required),
        HGroup => (b"hgroup", Required),
        Hr => (b"hr", Void),
        Html => (b"html", Optional(&[])),
        Italic => (b"i", Required),
        IFrame => (b"iframe", Required),
        Img => (b"img", Void),
        Input => (b"input", Void),
        Ins => (b"ins", Required),
        Kbd => (b"kbd", Required),
        Label => (b"label", Required),
        Legend => (b"legend", Required),
        Li => (b"li", Optional(&[El::Li])),
        Link => (b"link", Void),
        Main => (b"main", Required),
        Map => (b"map", Required),
        Mark => (b"mark", Required),
        Menu => (b"menu", Required),
        Meta => (b"meta", Void),
        Meter => (b"meter", Required),
        Nav => (b"nav", Required),
        NoScript => (b"noscript", Required),
        Object => (b"object", Required),
        Ol => (b"ol", Required),
        OptGroup => (b"optgroup", Optional(&[El::OptGroup])),
        Option => (b"option", Optional(&[El::Option, El::OptGroup])),
        Output => (b"output", Required),
        Paragraph => (b"p", Optional(&[El::Paragraph])),
        Picture => (b"picture", Required),
        Pre => (b"pre", Required),
        Progress => (b"progress", Required),
        Quote => (b"q", Required),
        Rp => (b"rp", Optional(&[El::Rp, El::Rt])),
        Rt => (b"rt", Optional(&[El::Rp, El::Rt])),
        Ruby => (b"ruby", Required),
        Strike => (b"s", Required),
        Samp => (b"samp", Required),
        Script => (b"script", Required),
        Section => (b"section", Required),
        Select => (b"select", Required),
        Slot => (b"slot", Required),
        Small => (b"small", Required),
        Source => (b"source", Void),
        Span => (b"span", Required),
        Strong => (b"strong", Required),
        Style => (b"style", Required),
        Sub => (b"sub", Required),
        Summary => (b"summary", Required),
        Sup => (b"sup", Required),
        Table => (b"table", Required),
        Tbody => (b"tbody", Optional(&[El::Tbody, El::Tfoot])),
        Td => (b"td", Optional(&[El::Td, El::Th])),
        Template => (b"template", Required),
        TextArea => (b"textarea", Required),
        Tfoot => (b"tfoot", Optional(&[])),
        Th => (b"th", Optional(&[El::Td, El::Th])),
        Thead => (b"thead", Optional(&[El::Tbody, El::Tfoot])),
        Time => (b"time", Required),
        Title => (b"title", Required),
        Tr => (b"tr", Optional(&[El::Tr])),
        Track => (b"track", Required),
        Underline => (b"u", Required),
        Ul => (b"ul", Required),
        Var => (b"var", Required),
        Video => (b"video", Required),
        Wbr => (b"wbr", Void),
    }
}
