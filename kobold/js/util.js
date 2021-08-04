export function __kobold_mount(n,c) { n.appendChild(c); }
export function __kobold_unmount(n,c) { n.removeChild(c); }
export function __kobold_empty_node() { return document.createTextNode(""); }
export function __kobold_text_node(t) { return document.createTextNode(t); }
export function __kobold_update_text(n,t) { n.textContent = t; }
export function __kobold_create_div() { return document.createElement('div'); }
export function __kobold_create_attr(n,v) { let a = document.createAttribute(n); a.value = v; return a; }
export function __kobold_create_attr_class(v) { let a = document.createAttribute('class'); a.value = v; return a; }
export function __kobold_create_attr_style(v) { let a = document.createAttribute('style'); a.value = v; return a; }
export function __kobold_update_attr(n,v) { n.value = v; }
