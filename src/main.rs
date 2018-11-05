use std::fs;

fn main() {
    let buf = fs::read("dist/constant.wasm");
    println!("{:x?}", buf);
}
