// Must be kept in sync with runtime::events
const events = [
	"click",
	"dblclick",
	"pointerdown",
	"pointerup",
	"pointermove",
	"pointerover",
	"pointerout",
	"pointercancel",
	"contextmenu",
	"wheel",
	"touchstart",
	"touchmove",
	"touchend",
	"touchcancel",
	"keydown",
	"keyup",
	"focusin",
	"focusout",
	"change",
	"reset",
	"invalid",
	"beforeinput",
	"select",
];

const eventSymbols = window.$_koboldSym = Array(23);

const fragmentDecorators = Symbol();

export function appendBody(n) {
	document.body.appendChild(n);
}
export function createTextNode(t) {
	return document.createTextNode(t);
}
export function delegateEvent(i) {
	let event = events[i];
	let symbol = eventSymbols[i] = Symbol(event);

	document.body.addEventListener(event, e => {
		let probe = e.target;

		while !Object.hasOwn(probe, symbol) {
			probe = probe.parentElement;

			if (probe == null) {
				return;
			}
		}

		wasmBindings.koboldTrigger(probe[symbol], e);
	});
}

export function emptyNode() { return document.createTextNode(""); }
export function fragment()
{
	let f = document.createDocumentFragment();
	f.append("", "");
	return f;
};
export function fragmentDecorate(f) {
	f[fragmentDecorators] = [f.firstChild, f.lastChild];
	return f.lastChild;
}
export function fragmentUnmount(f)
{
	let [b, e] = f[fragmentDecorators];
	while (b.nextSibling !== e) f.appendChild(b.nextSibling);
	f.appendChild(e);
	f.insertBefore(b, f.firstChild);
}
export function fragmentReplace(f,n)
{
	let [b, e] = f[fragmentDecorators];
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

export function popState() { window.onpopstate = makeEventHandler(0); }
export function pushState(e) { e.preventDefault(); history.pushState(null,'',e.currentTarget.href); }
export function getPath() { return document.location.pathname; }
