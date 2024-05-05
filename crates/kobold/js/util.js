const fragmentDecorators = new WeakMap();

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

export function makeEventHandler(c,f) { return (e) => wasmBindings.koboldCallback(e,c,f); }
export function checkEventHandler() { if (typeof wasmBindings !== "object") console.error(
`Missing \`wasmBindings\` in global scope.
As of Kobold v0.10 and Trunk v0.17.16 you no longer need to export bindings manually, \
please remove the custom \`pattern_script\' from your \`Trunk.toml\` file.
`) }
