const fs = require("fs");

const memory = new WebAssembly.Memory({ initial: 256, maximum: 256 });
const importObj = {
  env: {
    abortStackOverflow: () => {
      throw new Error("overflow");
    },
    table: new WebAssembly.Table({
      initial: 0,
      maximum: 0,
      element: "anyfunc"
    }),
    tableBase: 0,
    memory: memory,
    memoryBase: 1024,
    STACKTOP: 0,
    STACK_MAX: memory.buffer.byteLength
  }
};

if (process.argv.length < 3) {
  console.error("Error! Use like 'node run-wasm.js [name-of-wasm-file] [name-of-invoke-function] [...arguments]'");
  process.exit(1);
}
const [_, __, name, invoke, ...args] = process.argv;
const wasm = `./${name}.wasm`;
const invoke_fn = `${invoke}`;
console.log(`Invoke "${invoke_fn}" of "${wasm}" with "${args}"`);
const buffer = fs.readFileSync(wasm);
WebAssembly.instantiate(buffer, importObj)
  .then(mod => {
    console.log(mod.instance.exports[invoke_fn](...args));
  })
  .catch(console.error);
