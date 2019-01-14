use decode::{Byte, Section};
use frame::Frame;
use module::ExternalModules;
use stack::Stack;
use store::Store;
use trap::Result;
use validate;
use validate::Context;
use vm::Vm;

pub fn init_store() -> Store {
  Default::default()
}

pub fn decode_module(bytes: &[u8]) -> Result<Section> {
  Byte::new_with_drop(&bytes)?.decode()
}

pub fn validate_module(module: &Result<Section>) -> validate::Result<()> {
  match module {
    Ok(module) => Context::new(module).validate(),
    Err(err) => Err(validate::TypeError::Trap(err.to_owned())),
  }
}

pub fn instantiate_module(
  mut store: Store,
  section: Result<Section>, // module: Module(PreVm)
  external_modules: ExternalModules,
) -> Result<Vm> {
  // TODO: Return pair of (Store, Vm) by using Rc<Store> type.
  let internal_module = section?.complete(&external_modules, &mut store)?;
  let mut vm = Vm::new_from(store, internal_module, external_modules)?;
  if let Some(idx) = vm.start_index().clone() {
    let function_instance = vm.get_function_instance(&idx)?;
    let frame = Frame::new(
      vm.stack.stack_ptr,
      vm.stack.frame_ptr,
      function_instance,
      &mut vec![],
    );
    vm.stack.push_frame(frame)?;
    vm.evaluate()?;
    vm.stack = Stack::new(65536);
  };
  Ok(vm)
}

// module_imports(module):(name,name,externtype)∗¶
// module_exports(module):(name,externtype)∗¶
// get_export(moduleinst,name):externval | error¶

// Function
// alloc_func(store,functype,hostfunc):(store,funcaddr)¶
// type_func(store,funcaddr):functype¶
// invoke_func(store,funcaddr,val∗):(store,val∗ | error)¶

// Table
// alloc_table(store,tabletype):(store,tableaddr)¶
// type_table(store,tableaddr):tabletype¶
// read_table(store,tableaddr,i):funcaddr? | error¶
// write_table(store,tableaddr,i,funcaddr?):store | error¶
// size_table(store,tableaddr):i32¶
// grow_table(store,tableaddr,n):store | error¶

// Memory
// alloc_mem(store,memtype):(store,memaddr)
// type_mem(store,memaddr):memtype¶
// read_mem(store,memaddr,i):byte | error
// write_mem(store,memaddr,i,byte):store | error
// size_mem(store,memaddr):i32
// grow_mem(store,memaddr,n):store | error

// Global
// alloc_global(store,globaltype,val):(store,globaladdr)
// type_global(store,globaladdr):globaltype
// read_global(store,globaladdr):val
// write_global(store,globaladdr,val):store | error
