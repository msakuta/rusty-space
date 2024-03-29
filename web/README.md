# Build for the web

Follow these steps to build the web page with Wasm. All commands should run in this `web/` directory.

1. Make sure you have both `Rust`, `npm` (which should include `npx`) and `wasm-pack` installed.

2. Run

```console
$ npm install
```

3. Run (The following command builds the `triangle` example. Replace the path to the example to build other examples. You can find an overview of all examples in `examples/README.md`.)

```console
$ wasm-pack build ".." --target web --out-name web --out-dir ./web/pkg
```

4. Run

```console
$ npm run serve
```

5. Open `http://localhost:8080` in a browser
