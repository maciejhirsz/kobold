use std::fmt::Write;

use crate::dom2::Node;
use crate::gen2::{DomNode, Generator, IntoGenerator, Short};

pub struct JsFragment {
    /// Variable name of the fragment, such as `e0`
    pub var: Short,

    /// All the appends to this fragment.
    pub code: String,

    /// Arguments to import from rust
    pub args: Vec<Short>,
}

impl IntoGenerator for Vec<Node> {
    fn into_gen(self, gen: &mut Generator) -> DomNode {
        assert!(!self.is_empty());

        let var = gen.out.next_el();

        let mut code = format!("let {var}=document.createDocumentFragment();\n");
        let mut args = Vec::new();

        let append = append(gen, &mut code, &mut args, self);
        let _ = writeln!(code, "{var}.{append}");

        DomNode::Fragment(JsFragment { var, code, args })
    }
}

pub fn append(
    gen: &mut Generator,
    js: &mut String,
    args: &mut Vec<Short>,
    children: Vec<Node>,
) -> String {
    let mut append = format!("append(");
    for child in children {
        let dom_node = child.into_gen(gen);

        match &dom_node {
            DomNode::Variable(value) => {
                args.push(*value);

                let _ = write!(append, "{value},");
            }
            DomNode::TextNode(text) => {
                // write the text verbatim, no need to go through `document.createTextNode`
                let _ = write!(append, "{text},");
            }
            DomNode::Element(el) => {
                let _ = writeln!(js, "let {}=document.createElement('{}');", el.var, el.tag);

                js.push_str(&el.code);

                args.extend(el.args.iter().copied());

                let _ = write!(append, "{},", el.var);
            }
            DomNode::Fragment(_) => {
                panic!("Unexpected document fragment in the middle of the DOM");
            }
        };
    }

    append.pop();
    append.push_str(");");
    append
}
