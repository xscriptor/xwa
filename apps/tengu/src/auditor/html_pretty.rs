use ego_tree::NodeRef;
use scraper::node::Node;
use scraper::{ElementRef, Html};
use std::fmt::Write;

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
    "param", "source", "track", "wbr",
];

fn is_void(name: &str) -> bool {
    VOID_ELEMENTS.contains(&name)
}

fn render_attrs(el: &ElementRef) -> String {
    let mut attrs = String::new();
    for (name, val) in el.value().attrs() {
        let escaped = val.replace('&', "&amp;").replace('"', "&quot;");
        let _ = write!(attrs, " {}=\"{}\"", name, escaped);
    }
    attrs
}

fn pretty_node(node: &NodeRef<'_, Node>, depth: usize, output: &mut String) {
    match node.value() {
        Node::Document => {
            for child in node.children() {
                pretty_node(&child, depth, output);
            }
        }
        Node::Doctype(doctype) => {
            let name = doctype.name.as_ref();
            let _ = writeln!(output, "<!DOCTYPE {}>", name);
        }
        Node::Text(text) => {
            let trimmed = text.text.trim();
            if !trimmed.is_empty() {
                let indent = "  ".repeat(depth);
                let _ = writeln!(output, "{}{}", indent, trimmed);
            }
        }
        Node::Comment(comment) => {
            let indent = "  ".repeat(depth);
            let _ = writeln!(output, "{}<!-- {} -->", indent, comment.comment.trim());
        }
        Node::Element(_) => {
            if let Some(el) = ElementRef::wrap(node.clone()) {
                let tag = el.value().name();
                let attrs = render_attrs(&el);
                let indent = "  ".repeat(depth);

                let children: Vec<_> = node.children().collect();
                let has_content = children.iter().any(|c| match c.value() {
                    Node::Text(t) => !t.text.trim().is_empty(),
                    Node::Element(e) => !is_void(e.name()),
                    _ => false,
                });
                let has_block = children.iter().any(|c| match c.value() {
                    Node::Element(_) => true,
                    Node::Text(t) => t.text.trim().contains('\n') || t.text.trim().len() > 80,
                    _ => false,
                });

                if is_void(tag) {
                    let _ = writeln!(output, "{}<{}{} />", indent, tag, attrs);
                } else if !has_content {
                    let _ = writeln!(output, "{}<{}{}></{}>", indent, tag, attrs, tag);
                } else if !has_block {
                    let inner: String = children
                        .iter()
                        .map(|c| {
                            if let Node::Text(t) = c.value() {
                                t.text.trim().to_string()
                            } else if let Some(child_el) = ElementRef::wrap(c.clone()) {
                                let c_tag = child_el.value().name();
                                let c_attrs = render_attrs(&child_el);
                                if is_void(c_tag) {
                                    format!("<{}{} />", c_tag, c_attrs)
                                } else {
                                    let c_text: String = c
                                        .children()
                                        .filter_map(|gc| {
                                            if let Node::Text(t) = gc.value() {
                                                Some(t.text.trim().to_string())
                                            } else {
                                                None
                                            }
                                        })
                                        .collect();
                                    format!("<{}{}>{}</{}>", c_tag, c_attrs, c_text, c_tag)
                                }
                            } else {
                                String::new()
                            }
                        })
                        .collect();
                    let _ = writeln!(
                        output,
                        "{}<{}{}>{}</{}>",
                        indent, tag, attrs, inner, tag
                    );
                } else {
                    let _ = writeln!(output, "{}<{}{}>", indent, tag, attrs);
                    for child in &children {
                        pretty_node(child, depth + 1, output);
                    }
                    let _ = writeln!(output, "{}</{}>", indent, tag);
                }
            }
        }
        _ => {}
    }
}

pub fn pretty_print(html: &str) -> String {
    let document = Html::parse_document(html);
    let root = document.tree.root();
    let mut output = String::with_capacity(html.len() + html.len() / 4);
    pretty_node(&root, 0, &mut output);
    output
}
