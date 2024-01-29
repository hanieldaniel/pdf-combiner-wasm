import initSync, { list_filename } from '../pkg/pdf_combiner_wasm.js'

const fs = window.requestFileSystem()


let wasm = await initSync();

let filesarray = []
const fileSelector = document.getElementById('file-selector');
const filesUi = document.querySelector('#file_list')

fileSelector.addEventListener('change', (event) => {
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

    list_filename(fileList);
});

console.log(fs)