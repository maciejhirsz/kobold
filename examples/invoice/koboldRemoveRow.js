export function koboldRemoveRow(elemId) {
    // use the `id` from the destroy button, which is the same
    // value as the `id` used by the associated parent `div` element
    console.log('koboldRemoveRow: elemId: ', elemId);
    let row = document.querySelectorAll(`tr#${elemId}`)[0];
    console.log('row to remove: ', row);
    row.parentNode.removeChild(row);
	return true;
}
