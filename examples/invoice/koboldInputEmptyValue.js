export function koboldInputEmptyValue(elementId) {
    const el = document.getElementById(elementId);
    el.value = "";
	console.log('input value emptied: ', elementId);
}
