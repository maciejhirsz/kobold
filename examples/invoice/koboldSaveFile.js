// generates an object url for a CSV file download by automatically generating a blob url
// (to download the file from) and associates that with a temporary hyperlink that is generated.
// it then clicks that hyperlink automatically to trigger the save file prompt for the user
// before removing the temporary hyperlink
export function koboldSaveFile(filename, data) {
    const blob = new Blob([data], { type: 'application/octet-stream' });
    console.log('created blob: ', blob);
    const link = document.createElement('a');
    link.href = window.URL.createObjectURL(blob);
    link.download = filename;
    link.click();
	console.log('clicked link');
    window.URL.revokeObjectURL(link.href);
	return true;
}
