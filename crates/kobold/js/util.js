const fragmentDecorators = new WeakMap();

export function appendBody(n) {
	document.body.appendChild(n);
}
export function createTextNode(t) {
	return document.createTextNode(t);
}

export function emptyNode() { return document.createTextNode(""); }
export function fragment()
{
	let f = document.createDocumentFragment();
	f.append("", "");
	return f;
};
export function fragmentDecorate(f) {
	fragmentDecorators.set(f, [f.firstChild, f.lastChild]);
	return f.lastChild;
}
export function fragmentUnmount(f)
{
	let [b, e] = fragmentDecorators.get(f);
	while (b.nextSibling !== e) f.appendChild(b.nextSibling);
	f.appendChild(e);
	f.insertBefore(b, f.firstChild);
}
export function fragmentReplace(f,n)
{
	let [b, e] = fragmentDecorators.get(f);
	while (b.nextSibling !== e) f.appendChild(b.nextSibling);
	b.replaceWith(n);
	f.appendChild(e);
	f.insertBefore(b, f.firstChild);
}

export function setChecked(n,v) { if (n.checked !== v) n.checked = v; }

export function addClass(n,v) { n.classList.add(v); }
export function removeClass(n,v) { n.classList.remove(v); }
export function replaceClass(n,o,v) { n.classList.replace(o,v); }
export function toggleClass(n,c,v) { n.classList.toggle(c,v); }

export function makeEventHandler(eid) { return (e) => wasmBindings.koboldTrigger(eid,e); }
export function handlePopState() { window.onpopstate = makeEventHandler(0); }
export function getPath() { return document.location.pathname; }