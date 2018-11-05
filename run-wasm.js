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

if (process.argv.length !== 3) {
  console.error("Error! Use like 'node run-wasm.js name-of-wasm'");
  process.exit(1);
}

const wasm = `./dist/${process.argv[2]}.wasm`;
const buffer = fs.readFileSync(wasm);
WebAssembly.instantiate(buffer, importObj)
  .then(mod => {
    console.log(mod.instance.exports._subject(2, 4));
  })
  .catch(console.error);
