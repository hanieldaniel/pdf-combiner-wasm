# PDF combiner WASM

## Instructions to build

### Install **wasm-pack**

```bash
cargo install wasm-pack
```

or

```bash
npm install -g wasm-pack
```

### build using wasm-pack with target web

```bash
wasm-pack build --release --target web
```

### Serve the project from client

use a local server or use a simple http-server

```bash
python -m http.server 8000
```

or

```bash
npx http-server -o /path/to/static/content
```
