use std::fmt::{self, Debug};
use std::ops::Deref;

use fnv::FnvHashMap;
use once_cell::sync::Lazy;

macro_rules! build_tags {
    (
        $($variant:ident $tag:literal $closing:expr;)*
    ) => {
        const VARIANTS: usize = 0 $(+ { let _ = ElementTag::$variant; 1 })*;

        #[derive(Clone, Copy, PartialEq, Eq)]
        #[repr(u8)]
        pub enum ElementTag {
            $($variant,)*
        }

        static TAGS: [&str; VARIANTS] = [$($tag,)*];

        static CLOSING: [ClosingRules; VARIANTS] = [$(closing!($closing),)*];

        static TAG_BY_NAME: Lazy<FnvHashMap<&str, ElementTag>> = Lazy::new(|| {
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
    (_) => {
        ClosingRules::Standard
    };
    ($e:expr) => {
        $e
    };
}

enum ClosingRules {
    /// No special rules apply
    Standard,
    /// Element can have no children and no closing tag, e.g. `<img>` or `<br>`
    ForbidsChildren,
    /// Closes if a tag with a specific name is encountered
    ClosesOn(ElementTag),
    /// Closes on either of the two tags
    ClosesOn2([ElementTag; 2]),
    /// Closes on either of the three tags
    ClosesOn3([ElementTag; 3]),
    /// Closes on either of the five tags
    ClosesOn5([ElementTag; 5]),
    /// Closes on one of many tags: https://html.spec.whatwg.org/#the-p-element
    ClosesOnParagraph,
}

impl ElementTag {
    pub fn from_str(tag: &str) -> Option<Self> {
        thread_local! {
            // Make a thread-local reference to avoid going through a sync lock on each call
            static LOCAL_TAG_BY_NAME: &'static FnvHashMap<&'static str, ElementTag> = &TAG_BY_NAME;
        }
        LOCAL_TAG_BY_NAME.with(|map| map.get(tag).copied())
    }

    pub fn forbids_children(self) -> bool {
        matches!(CLOSING[self as usize], ClosingRules::ForbidsChildren)
    }

    pub fn closes_on(self, other: ElementTag) -> bool {
        use ElementTag::*;

        let tags = match &CLOSING[self as usize] {
            ClosesOn(tag) => std::slice::from_ref(tag),
            ClosesOn2(tags) => tags,
            ClosesOn3(tags) => tags,
            ClosesOn5(tags) => tags,
            ClosesOnParagraph => &[
                Address, Article, Aside, Blockquote, Details, Div, Dl, FieldSet, FigCaption,
                Figure, Footer, Form, Header, Header1, Header2, Header3, Header4, Header5, Header6,
                HGroup, Hr, Main, Menu, Nav, Ol, Paragraph, Pre, Search, Section, Table, Ul,
            ],
            _ => return false,
        };

        tags.contains(&other)
    }
}

impl Deref for ElementTag {
    type Target = str;

    fn deref(&self) -> &str {
        TAGS[*self as usize]
    }
}

const __: ClosingRules = ClosingRules::Standard;

use ClosingRules::*;

use ElementTag::{Tbody, Td, Tfoot, Th, Tr};

#[rustfmt::skip]
build_tags! {
    Anchor      "a"             __;
    Abbr        "abbr"          __;
    Address     "address"       __;
    Area        "area"          ForbidsChildren;
    Article     "article"       __;
    Aside       "aside"         __;
    Audio       "audio"         __;
    Bold        "b"             __;
    Base        "base"          ForbidsChildren;
    Bdi         "bdi"           __;
    Bdo         "bdo"           __;
    Blockquote  "blockquote"    __;
    Body        "body"          __;
    Br          "br"            ForbidsChildren;
    Button      "button"        __;
    Canvas      "canvas"        __;
    Caption     "caption"       __;
    Cite        "cite"          __;
    Code        "code"          __;
    Col         "col"           ForbidsChildren;
    ColGroup    "colgroup"      __;
    Data        "data"          __;
    DataList    "datalist"      __;
    Dd          "dd"            ClosesOn2([ElementTag::Dd, ElementTag::Dt]);
    Del         "del"           __;
    Definition  "dfn"           __;
    Details     "details"       __;
    Dialog      "dialog"        __;
    Div         "div"           __;
    Dl          "dl"             __;
    Dt          "dt"            ClosesOn2([ElementTag::Dd, ElementTag::Dt]);
    Em          "em"            __;
    Embed       "embed"         ForbidsChildren;
    FieldSet    "fieldset"      __;
    FigCaption  "figcaption"    __;
    Figure      "figure"        __;
    Footer      "footer"        __;
    Form        "form"          __;
    Head        "head"          __;
    Header      "header"        __;
    Header1     "h1"            __;
    Header2     "h2"            __;
    Header3     "h3"            __;
    Header4     "h4"            __;
    Header5     "h5"            __;
    Header6     "h6"            __;
    HGroup      "hgroup"        __;
    Hr          "hr"            ForbidsChildren;
    Html        "html"          __;
    Italic      "i"             __;
    IFrame      "iframe"        __;
    Img         "img"           ForbidsChildren;
    Input       "input"         ForbidsChildren;
    Ins         "ins"           __;
    Kbd         "kbd"           __;
    Label       "label"         __;
    Legend      "legend"        __;
    Li          "li"            ClosesOn(ElementTag::Li);
    Link        "link"          ForbidsChildren;
    Main        "main"          __;
    Map         "map"           __;
    Mark        "mark"          __;
    Menu        "menu"          __;
    Meta        "meta"          ForbidsChildren;
    Meter       "meter"         __;
    Nav         "nav"           __;
    NoScript    "noscript"      __;
    Object      "object"        __;
    Ol          "ol"            __;
    OptGroup    "optgroup"      ClosesOn(ElementTag::OptGroup);
    Option      "option"        ClosesOn2([ElementTag::Option, ElementTag::OptGroup]);
    Output      "output"        __;
    Paragraph   "p"             ClosesOnParagraph;
    Picture     "picture"       __;
    Pre         "pre"           __;
    Progress    "progress"      __;
    Quote       "q"             __;
    Rp          "rp"            ClosesOn2([ElementTag::Rp, ElementTag::Rt]);
    Rt          "rt"            ClosesOn2([ElementTag::Rp, ElementTag::Rt]);
    Ruby        "ruby"          __;
    Strike      "s"             __;
    Samp        "samp"          __;
    Search      "search"        __;
    Script      "script"        __;
    Section     "section"       __;
    Select      "select"        __;
    Slot        "slot"          __;
    Small       "small"         __;
    Source      "source"        ForbidsChildren;
    Span        "span"          __;
    Strong      "strong"        __;
    Style       "style"         __;
    Sub         "su"            __;
    Summary     "summary"       __;
    Sup         "sup"           __;
    Table       "table"         __;
    Tbody       "tbody"         ClosesOn2([Tbody, Tfoot]);
    Td          "td"            ClosesOn5([Td, Th, Tr, Tbody, Tfoot]);
    Template    "template"      __;
    TextArea    "textarea"      __;
    Tfoot       "tfoot"         __;
    Th          "th"            ClosesOn5([Td, Th, Tr, Tbody, Tfoot]);
    Thead       "thead"         ClosesOn2([Tbody, Tfoot]);
    Time        "time"          __;
    Title       "title"         __;
    Tr          "tr"            ClosesOn3([Tr, Tbody, Tfoot]);
    Track       "track"         ForbidsChildren;
    Underline   "u"             __;
    Ul          "ul"            __;
    Var         "var"           __;
    Video       "video"         __;
    Wbr         "wbr"           ForbidsChildren;
}
