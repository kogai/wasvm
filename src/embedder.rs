use module::ExternalModules;
use store::Store;
use trap::Result;
use vm::Vm;

pub fn init_store() -> Store {
  unimplemented!();
}

pub fn decode_module(bytes: &[u8]) -> Vm /* Module */ {
  unimplemented!();
}

pub fn validate_module(vm: &Vm /* module: Module(PreVm) */) -> Result<()> {
  unimplemented!();
}

pub fn instantiate_module(
  store: &Store,
  /* module: Module(PreVm), */
  external_val: ExternalModules,
)
/* -> Result<Store, ModuleInst(Vm), Error> */
{
  unimplemented!();
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
