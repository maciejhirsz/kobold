use std::fmt::Write;

use crate::dom::Node;
use crate::gen::{DomNode, Generator, IntoGenerator, JsArgument, Short};

pub struct JsFragment {
    /// Variable name of the fragment, such as `e0`
    pub var: Short,

    /// All the appends to this fragment.
    pub code: String,

    /// Arguments to import from rust
    pub args: Vec<JsArgument>,
}

impl IntoGenerator for Vec<Node> {
    fn into_gen(self, gen: &mut Generator) -> DomNode {
        assert!(!self.is_empty());

        let var = gen.names.next_el();

        let mut code = format!("let {var}=document.createDocumentFragment();\n");
        let mut args = Vec::new();

        let append = append(gen, &mut code, &mut args, self);
        let _ = writeln!(code, "{var}.{append};");
        let _ = writeln!(code, "return {var};");

        DomNode::Fragment(JsFragment { var, code, args })
    }
}

pub fn append(
    gen: &mut Generator,
    js: &mut String,
    args: &mut Vec<JsArgument>,
    children: Vec<Node>,
) -> String {
    let mut append = String::from("append(");
    for child in children {
        let dom_node = child.into_gen(gen);

        match dom_node {
            DomNode::Variable(value) => {
                args.push(JsArgument::new(value));

                let _ = write!(append, "{value},");
            }
            DomNode::TextNode(text) => {
                // write the text verbatim, no need to go through `document.createTextNode`
                let _ = write!(append, "{text},");
            }
            DomNode::Element(el) => {
                let var = el.var;
                if el.hoisted {
                    gen.hoist(DomNode::Element(el));

                    args.push(JsArgument::new(var));
                } else {
                    let _ = writeln!(js, "let {}=document.createElement(\"{}\");", el.var, el.tag);

                    js.push_str(&el.code);

                    args.extend(el.args);
                }

                let _ = write!(append, "{var},");
            }
            DomNode::Fragment(_) => {
                panic!("Unexpected document fragment in the middle of the DOM");
            }
        };
    }

    append.pop();
    append.push(')');
    append
}
