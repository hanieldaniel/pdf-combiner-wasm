import initSync, { join_files } from '../pkg/pdf_combiner_wasm.js'

let wasm = await initSync();

let filesarray = []
const fileSelector = document.getElementById('file-selector');
const filesUi = document.querySelector('#file_list');
const link = document.getElementById('link');
let objectURL;

fileSelector.addEventListener('change', async (event) => {
    const fileList = event.target.files;
    const files = Array.from(fileList);
    filesarray = [...filesarray, ...files]

    // re-remder files
    filesUi.innerHTML = null;
    filesarray.forEach(x => {
        const liElem = document.createElement('li')
        liElem.innerHTML = x.name
        filesUi.append(liElem)
    })

    const file = await join_files(fileList);
    console.log(file);

    if (objectURL) {
        URL.revokeObjectURL(objectURL);
    }

    var url = URL.createObjectURL(file);
    link.setAttribute("download", 'out.pdf');

    link.href = url;
});
