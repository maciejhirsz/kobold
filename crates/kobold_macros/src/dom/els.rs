use std::ops::Deref;
use std::fmt::{self, Debug};

use fnv::FnvHashMap;
use once_cell::sync::Lazy;

macro_rules! build_tags {
    (
        $($variant:ident $tag:literal $closing:tt)*
    ) => {
        const VARIANTS: usize = 0 $(+ { let _ = ElementTag::$variant; 1 })*;

        #[derive(Clone, Copy, PartialEq, Eq)]
        pub enum ElementTag {
            $($variant,)*
        }

        static TAGS: [&str; VARIANTS] = [$($tag,)*];

        static CLOSING: [ClosingRules; VARIANTS] = [$(closing!($closing),)*];

        static TAG_BY_NAME: Lazy<FnvHashMap<&'static str, ElementTag>> = Lazy::new(|| {
            let mut m = FnvHashMap::default();
            $(
                m.insert($tag, ElementTag::$variant);
            )*
            m
        });
    };
}

impl Debug for ElementTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

macro_rules! closing {
    (_) => {ClosingRules::Required};
    (/) => {ClosingRules::Void};
    ([$($tag:ident)*]) => {ClosingRules::Optional(&[$(ElementTag::$tag,)*])};
}

#[derive(Clone, Copy)]
pub enum ClosingRules {
    /// Void element, no closing tag, e.g. `<img>` or `<br>`
    Void,
    /// Required closing tag, e.g. `<div>`
    Required,
    /// Optional closing tag plus opening tags that close this if any, e.g. `<li>`
    Optional(&'static [ElementTag]),
}

impl ElementTag {
    pub fn from_str(tag: &str) -> Option<Self> {
        TAG_BY_NAME.get(tag).copied()
    }

    pub fn closing(self) -> ClosingRules {
        CLOSING[self as usize]
    }

    pub fn forbids_children(self) -> bool {
        matches!(self.closing(), ClosingRules::Void)
    }
}

impl Deref for ElementTag {
    type Target = str;

    fn deref(&self) -> &str {
        TAGS[*self as usize]
    }
}

#[rustfmt::skip]
build_tags! {
    Anchor      "a"             _
    Abbr        "abbr"          _
    Address     "address"       _
    Area        "area"          /
    Article     "article"       _
    Aside       "aside"         _
    Audio       "audio"         _
    Bold        "b"             _
    Base        "base"          /
    Bdi         "bdi"           _
    Bdo         "bdo"           _
    Blockquote  "blockquote"    _
    Body        "body"          []
    Br          "br"            /
    Button      "button"        _
    Canvas      "canvas"        _
    Caption     "caption"       []
    Cite        "cite"          _
    Code        "code"          _
    Col         "col"           /
    ColGroup    "colgroup"      []
    Data        "data"          _
    DataList    "datalist"      _
    Dd          "dd"            [Dd Dt]
    Del         "del"           _
    Definition  "dfn"           _
    Details     "details"       _
    Dialog      "dialog"        _
    Div         "div"           _
    Dl          "dl"             _
    Dt          "dt"            [Dd Dt]
    Em          "em"            _
    Embed       "embed"         /
    FieldSet    "fieldset"      _
    FigCaption  "figcaption"    _
    Figure      "figure"        _
    Footer      "footer"        _
    Form        "form"          _
    Head        "head"          []
    Header      "header"        _
    Header1     "h1"            _
    Header2     "h2"            _
    Header3     "h3"            _
    Header4     "h4"            _
    Header5     "h5"            _
    Header6     "h6"            _
    HGroup      "hgroup"        _
    Hr          "hr"            /
    Html        "html"          []
    Italic      "i"             _
    IFrame      "iframe"        _
    Img         "img"           /
    Input       "input"         /
    Ins         "ins"           _
    Kbd         "kbd"           _
    Label       "label"         _
    Legend      "legend"        _
    Li          "li"            [Li]
    Link        "link"          /
    Main        "main"          _
    Map         "map"           _
    Mark        "mark"          _
    Menu        "menu"          _
    Meta        "meta"          /
    Meter       "meter"         _
    Nav         "nav"           _
    NoScript    "noscript"      _
    Object      "object"        _
    Ol          "ol"            _
    OptGroup    "optgroup"      [OptGroup]
    Option      "option"        [Option OptGroup]
    Output      "output"        _
    Paragraph   "p"             [
                                    Paragraph Address Article Aside Blockquote Details Div
                                    Dl FieldSet FigCaption Figure Footer Form Header
                                    Header1 Header2 Header3 Header4 Header5 Header6 HGroup
                                    Hr Main Menu Nav Ol Pre Search Section Table Ul
                                ]
    Picture     "picture"       _
    Pre         "pre"           _
    Progress    "progress"      _
    Quote       "q"             _
    Rp          "rp"            [Rp Rt]
    Rt          "rt"            [Rp Rt]
    Ruby        "ruby"          _
    Strike      "s"             _
    Samp        "samp"          _
    Search      "search"        _
    Script      "script"        _
    Section     "section"       _
    Select      "select"        _
    Slot        "slot"          _
    Small       "small"         _
    Source      "source"        /
    Span        "span"          _
    Strong      "strong"        _
    Style       "style"         _
    Sub         "su"            _
    Summary     "summary"       _
    Sup         "sup"           _
    Table       "table"         _
    Tbody       "tbody"         [Tbody Tfoot]
    Td          "td"            [Td Th]
    Template    "template"      _
    TextArea    "textarea"      _
    Tfoot       "tfoot"         []
    Th          "th"            [Td Th]
    Thead       "thead"         [Tbody Tfoot]
    Time        "time"          _
    Title       "title"         _
    Tr          "tr"            [Tr]
    Track       "track"         /
    Underline   "u"             _
    Ul          "ul"            _
    Var         "var"           _
    Video       "video"         _
    Wbr         "wbr"           /
}
