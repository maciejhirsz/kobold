export function __kobold_click_element() {
	let elemId = 'link-file-download';
	console.log('elem: ', document.getElementById(elemId));
	if (document.getElementById(elemId) == undefined) {
		console.log('cannot click');

		return false;
	}
	document.getElementById(elemId).click();
	console.log('clicked');

	return true;
}
