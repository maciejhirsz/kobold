export function __kobold_start(n) { document.body.appendChild(n); }
export function __kobold_append(n,c) { n.appendChild(c); }
export function __kobold_unmount(n) { n.remove(); }
export function __kobold_replace(o,n) { o.replaceWith(n); }
export function __kobold_empty_node() { return document.createTextNode(""); }
export function __kobold_fragment()
{
	let f = document.createDocumentFragment();
	f.append(f.$begin = document.createTextNode(""), f.$end = document.createTextNode(""));
	return f;
};
export function __kobold_fragment_append(f,c) { f.$end.before(c); }
export function __kobold_fragment_unmount(f)
{
	let b = f.$begin, e = f.$end;
	while (b.nextSibling !== e) f.appendChild(b.nextSibling);
	f.appendChild(e);
	f.insertBefore(b, f.firstChild);
}
export function __kobold_fragment_replace(f,n)
{
	let b = f.$begin, e = f.$end;
	while (b.nextSibling !== e) f.appendChild(b.nextSibling);
	b.replaceWith(n);
	f.appendChild(e);
	f.insertBefore(b, f.firstChild);
}
export function __kobold_fragment_drop(f)
{
	delete f.$begin;
	delete f.$end;
}
export function __kobold_text_node(t) { return document.createTextNode(t); }
export function __kobold_update_text(n,t) { n.textContent = t; }
export function __kobold_create_div() { return document.createElement('div'); }
export function __kobold_create_input() { return document.createElement('input'); }
export function __kobold_create_attr(n,v) { let a = document.createAttribute(n); a.value = v; return a; }
export function __kobold_create_attr_class(v) { let a = document.createAttribute('class'); a.value = v; return a; }
export function __kobold_create_attr_style(v) { let a = document.createAttribute('style'); a.value = v; return a; }
export function __kobold_update_attr(n,v) { n.value = v; }
