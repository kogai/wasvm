use super::error::{Result, TypeError};
use alloc::collections::VecDeque;
#[cfg(not(test))]
use alloc::prelude::*;
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
use decode::{Data, Element, Section, TableType};
use function::FunctionType;
use global::GlobalType;
use isa::{Indice, Inst};
use memory::Limit;
use module::{
  ExportDescriptor, ExternalInterface, ExternalInterfaces, ImportDescriptor, ModuleDescriptor,
};
use value_type::{ValueTypes, TYPE_F32, TYPE_F64, TYPE_I32, TYPE_I64};

type ResultType = [ValueTypes; 1];

#[derive(Debug, Clone)]
enum Entry {
  Type(ValueTypes),
  Label,
}

#[derive(Debug)]
struct TypeStack(RefCell<Vec<Entry>>);

impl TypeStack {
  fn new() -> Self {
    TypeStack(RefCell::new(Vec::new()))
  }

  fn len(&self) -> usize {
    self.0.borrow().len()
  }

  fn push(&self, ty: ValueTypes) {
    self.0.borrow_mut().push(Entry::Type(ty));
  }

  fn push_label(&self) {
    self.0.borrow_mut().push(Entry::Label);
  }

  fn pop(&self) -> Option<Entry> {
    self.0.borrow_mut().pop()
  }

  fn pop_type(&self) -> Result<ValueTypes> {
    match self.pop() {
      Some(Entry::Type(ty)) => Ok(ty),
      _ => Err(TypeError::TypeMismatch),
    }
  }

  fn pop_until_label(&self) -> Result<Vec<Entry>> {
    let mut buf = Vec::new();
    while let Some(Entry::Type(ty)) = self.pop() {
      buf.push(Entry::Type(ty));
    }
    Ok(buf)
  }

  fn pop_i32(&self) -> Result<ValueTypes> {
    match self.0.borrow_mut().pop() {
      Some(Entry::Type(ValueTypes::I32)) => Ok(ValueTypes::I32),
      _ => Err(TypeError::TypeMismatch),
    }
  }
}

#[derive(Debug)]
struct Function<'a> {
  function_type: &'a FunctionType,
  locals: &'a [ValueTypes],
  body: &'a [Inst],
  body_ptr: Cell<usize>,
  type_stack: TypeStack,
}

impl<'a> Function<'a> {
  fn new(
    function_type: &'a FunctionType,
    locals: &'a [ValueTypes],
    body: &'a [Inst],
  ) -> Function<'a> {
    Function {
      function_type,
      locals,
      body,
      body_ptr: Cell::new(0),
      type_stack: TypeStack::new(),
    }
  }

  fn pop(&self) -> Option<&Inst> {
    let ptr = self.body_ptr.get();
    self.body_ptr.set(ptr + 1);
    self.body.get(ptr)
  }

  fn pop_value_type(&self) -> Option<ValueTypes> {
    match self.pop() {
      Some(Inst::RuntimeValue(ty)) => Some(ty.to_owned()),
      _ => None,
    }
  }

  fn pop_raw_u32(&self) -> Result<u32> {
    let mut buf = [0; 4];
    for i in 0..buf.len() {
      let raw_byte = match self.pop() {
        Some(Inst::ExperimentalByte(b)) => b,
        _ => return Err(TypeError::NotFound),
      };
      buf[i] = *raw_byte;
    }
    let idx: u32 = unsafe { core::mem::transmute(buf) };
    Ok(idx)
  }
}

pub struct Context<'a> {
  function_types: &'a Vec<FunctionType>,
  functions: Vec<Function<'a>>,
  exports: &'a ExternalInterfaces,
  imports: &'a ExternalInterfaces,
  datas: &'a Vec<Data>,
  limits: &'a Vec<Limit>,
  tables: &'a Vec<TableType>,
  globals: &'a Vec<(GlobalType, Vec<Inst>)>,
  elements: &'a Vec<Element>,
  start: &'a Option<u32>,
  locals: RefCell<Vec<ValueTypes>>,
  labels: RefCell<VecDeque<ResultType>>,
  return_type: RefCell<ResultType>,
}

macro_rules! bin_op {
  ($stack: ident) => {{
    let l = $stack.pop_type()?;
    let r = $stack.pop_type()?;
    if l != r {
      return Err(TypeError::TypeMismatch);
    }
    $stack.push(l);
  }};
}

macro_rules! rel_op {
  ($stack: ident) => {{
    let l = $stack.pop_type()?;
    let r = $stack.pop_type()?;
    if l != r {
      return Err(TypeError::TypeMismatch);
    }
    $stack.push(ValueTypes::I32);
  }};
}

impl<'a> Context<'a> {
  pub fn new(module: &'a Section) -> Result<Self> {
    Ok(Context {
      function_types: &module.function_types,
      functions: module
        .codes
        .iter()
        .enumerate()
        .map(|(idx, code)| {
          let idx = module.functions.get(idx).map(|n| Indice::from(*n))?;
          let function_type = module.function_types.get(idx.to_usize())?;
          let (body, locals) = match code {
            Ok((body, locals)) => Ok((body, locals)),
            Err(ref err) => Err(TypeError::Trap(err.to_owned())),
          }?;
          Ok(Function::new(function_type, locals, body))
        })
        .collect::<Result<Vec<_>>>()?,
      exports: &module.exports,
      imports: &module.imports,
      datas: &module.datas,
      globals: &module.globals,
      tables: &module.tables,
      elements: &module.elements,
      limits: &module.limits,
      start: &module.start,

      locals: RefCell::new(Vec::new()),
      labels: RefCell::new(VecDeque::new()),
      return_type: RefCell::new([ValueTypes::Empty; 1]),
    })
  }

  fn validate_constant(&self, expr: &[Inst]) -> Result<ValueTypes> {
    let type_stack = TypeStack::new();
    let mut idx = 0;
    while idx < expr.len() {
      let x = &expr[idx];
      idx += 1;
      match x {
        Inst::I32Const(_) => type_stack.push(ValueTypes::I32),
        Inst::I64Const(_) => type_stack.push(ValueTypes::I64),
        Inst::F32Const(_) => type_stack.push(ValueTypes::F32),
        Inst::F64Const(_) => type_stack.push(ValueTypes::F64),
        Inst::GetGlobal => {
          let mut buf = [0; 4];
          for i in 0..buf.len() {
            idx += 1;
            let raw_byte = match expr[idx] {
              Inst::ExperimentalByte(b) => b,
              _ => return Err(TypeError::TypeMismatch),
            };
            buf[3 - i] = raw_byte;
          }
          let idx = Indice::from(unsafe { core::mem::transmute::<_, u32>(buf) });
          match self.globals.get(idx.to_usize()) {
            Some((GlobalType::Const(ty), _)) | Some((GlobalType::Var(ty), _)) => {
              type_stack.push(ty.clone())
            }
            _ => return Err(TypeError::ConstantExpressionRequired),
          }
        }
        Inst::End => {
          break;
        }
        _ => return Err(TypeError::ConstantExpressionRequired),
      }
    }
    if type_stack.len() > 1 {
      return Err(TypeError::TypeMismatch);
    }
    type_stack.pop_type()
  }

  fn validate_datas(&self) -> Result<()> {
    for Data { memidx, offset, .. } in self.datas.iter() {
      self
        .limits
        .get(*memidx as usize)
        .ok_or(TypeError::UnknownMemory)?;
      if ValueTypes::I32 != self.validate_constant(offset)? {
        return Err(TypeError::TypeMismatch);
      }
    }
    Ok(())
  }

  fn validate_elements(&self) -> Result<()> {
    for Element {
      table_idx,
      offset,
      init,
    } in self.elements.iter()
    {
      if self.tables.get(table_idx.to_usize()).is_none() {
        return Err(TypeError::UnknownTable(table_idx.to_u32()));
      }
      if ValueTypes::I32 != self.validate_constant(offset)? {
        return Err(TypeError::TypeMismatch);
      }
      for i in init.iter() {
        self
          .functions
          .get(i.to_usize())
          .ok_or_else(|| TypeError::UnknownFunction(i.to_u32()))?;
      }
    }
    Ok(())
  }

  fn validate_globals(&self) -> Result<()> {
    for (global_type, init) in self.globals.iter() {
      let type_stack = TypeStack::new();
      for x in init.iter() {
        match x {
          Inst::I32Const(_) => type_stack.push(ValueTypes::I32),
          Inst::I64Const(_) => type_stack.push(ValueTypes::I64),
          Inst::F32Const(_) => type_stack.push(ValueTypes::F32),
          Inst::F64Const(_) => type_stack.push(ValueTypes::F64),
          Inst::GetGlobal => return Err(TypeError::ConstantExpressionRequired),
          Inst::End => {
            break;
          }
          _ => return Err(TypeError::ConstantExpressionRequired),
        }
      }
      if type_stack.len() > 1 {
        return Err(TypeError::TypeMismatch);
      }
      let ty = type_stack.pop_type()?;
      if &ty
        != match global_type {
          GlobalType::Const(expect) | GlobalType::Var(expect) => expect,
        }
      {
        return Err(TypeError::TypeMismatch);
      }
    }
    Ok(())
  }

  fn validate_exports(&self) -> Result<()> {
    let mut names = Vec::with_capacity(self.exports.len());
    for ExternalInterface {
      descriptor, name, ..
    } in self.exports.iter()
    {
      match descriptor {
        ModuleDescriptor::ExportDescriptor(ExportDescriptor::Function(x)) => {
          self
            .functions
            .get(x.to_usize())
            .ok_or_else(|| TypeError::UnknownFunction(x.to_u32()))?;
        }
        ModuleDescriptor::ExportDescriptor(ExportDescriptor::Table(x)) => {
          self
            .tables
            .get(x.to_usize())
            .ok_or_else(|| TypeError::UnknownTable(x.to_u32()))?;
        }
        ModuleDescriptor::ExportDescriptor(ExportDescriptor::Memory(x)) => {
          self
            .limits
            .get(x.to_usize())
            .ok_or_else(|| TypeError::UnknownMemory)?;
        }
        ModuleDescriptor::ExportDescriptor(ExportDescriptor::Global(x)) => {
          self
            .globals
            .get(x.to_usize())
            .ok_or_else(|| TypeError::UnknownGlobal(x.to_u32()))?;
        }
        _ => unreachable!(),
      };
      names.push(name);
    }
    names.dedup();
    if names.len() != self.exports.len() {
      return Err(TypeError::DuplicateExportName);
    }
    Ok(())
  }

  fn validate_imports(&self) -> Result<()> {
    let mut tables = Vec::new();
    let mut memories = Vec::new();
    for ExternalInterface { descriptor, .. } in self.imports.iter() {
      match descriptor {
        ModuleDescriptor::ImportDescriptor(ImportDescriptor::Function(x)) => {
          self
            .function_types
            .get(x.to_usize())
            .ok_or_else(|| TypeError::UnknownFunction(x.to_u32()))?;
        }
        ModuleDescriptor::ImportDescriptor(ImportDescriptor::Table(ty)) => {
          if !self.tables.is_empty() {
            return Err(TypeError::MultipleTables);
          }
          tables.push(ty);
        }
        ModuleDescriptor::ImportDescriptor(ImportDescriptor::Memory(limit)) => {
          if !self.limits.is_empty() {
            return Err(TypeError::MultipleMemories);
          }
          memories.push(limit);
        }
        ModuleDescriptor::ImportDescriptor(ImportDescriptor::Global(_ty)) => {}
        _ => unreachable!(),
      };
    }
    if tables.len() > 1 {
      return Err(TypeError::MultipleTables);
    }
    if memories.len() > 1 {
      return Err(TypeError::MultipleMemories);
    }
    Ok(())
  }

  fn validate_tables(&self) -> Result<()> {
    if self.tables.len() > 1 {
      return Err(TypeError::MultipleTables);
    }
    Ok(())
  }

  fn validate_memories(&self) -> Result<()> {
    for limit in self.limits.iter() {
      match limit {
        Limit::NoUpperLimit(min) => {
          if *min > 65536 {
            return Err(TypeError::InvalidMemorySize);
          }
        }
        Limit::HasUpperLimit(min, max) => {
          if min > max || *min > 65536 || *max > 65536 {
            return Err(TypeError::InvalidMemorySize);
          }
        }
      }
    }
    if self.limits.len() > 1 {
      return Err(TypeError::MultipleMemories);
    }
    Ok(())
  }

  fn validate_start(&self) -> Result<()> {
    if let Some(idx) = self.start {
      let func = self
        .functions
        .get(*idx as usize)
        .ok_or_else(|| TypeError::UnknownFunction(*idx))?;
      let ty = func.function_type;
      if !ty.parameters().is_empty() || !ty.returns().is_empty() {
        return Err(TypeError::InvalidStartFunction);
      }
    }
    Ok(())
  }

  fn validate_function_types(&self) -> Result<()> {
    for fy in self.function_types.iter() {
      if fy.returns().len() > 1 {
        return Err(TypeError::InvalidResultArity);
      }
    }
    Ok(())
  }

  fn validate_functions(&self) -> Result<()> {
    for f in self.functions.iter() {
      self.validate_function(f)?;
    }
    Ok(())
  }

  fn validate_load(
    &self,
    cxt: &TypeStack,
    bit_width: u32,
    ty: ValueTypes,
    function: &Function,
  ) -> Result<()> {
    let align = function.pop_raw_u32()?;
    let _offset = function.pop_raw_u32()?;
    self.limits.first().ok_or(TypeError::UnknownMemory)?;
    if 2u32.pow(align) > bit_width / 8 {
      return Err(TypeError::InvalidAlignment);
    };
    cxt.pop_i32()?;
    cxt.push(ty);
    Ok(())
  }

  fn validate_store(
    &self,
    cxt: &TypeStack,
    bit_width: u32,
    expect: &ValueTypes,
    function: &Function,
  ) -> Result<()> {
    let align = function.pop_raw_u32()?;
    let _offset = function.pop_raw_u32()?;
    self.limits.first().ok_or(TypeError::UnknownMemory)?;
    if 2u32.pow(align) > bit_width / 8 {
      return Err(TypeError::InvalidAlignment);
    };
    let actual = cxt.pop_type()?;
    cxt.pop_i32()?;
    if &actual != expect {
      return Err(TypeError::TypeMismatch);
    }
    Ok(())
  }

  fn validate_unary(&self, cxt: &TypeStack) -> Result<()> {
    let t = cxt.pop_type()?;
    cxt.push(t);
    Ok(())
  }

  fn validate_test_inst(&self, cxt: &TypeStack, expect: &ValueTypes) -> Result<()> {
    let actual = cxt.pop_type()?;
    if &actual != expect {
      return Err(TypeError::TypeMismatch);
    }
    cxt.push(ValueTypes::I32);
    Ok(())
  }

  fn validate_convert(&self, cxt: &TypeStack, from: &ValueTypes, to: ValueTypes) -> Result<()> {
    let from_ty = cxt.pop_type()?;
    if &from_ty != from {
      return Err(TypeError::TypeMismatch);
    }
    cxt.push(to);
    Ok(())
  }

  fn validate_function(&self, function: &Function) -> Result<()> {
    use self::Inst::*;
    let cxt = &function.type_stack;
    let labels = &mut self.labels.borrow_mut();
    let locals = &mut self.locals.borrow_mut();
    for param in function.function_type.parameters().iter() {
      locals.push(param.clone());
    }
    for local in function.locals.iter() {
      locals.push(local.clone());
    }
    if let Some(ret) = function.function_type.returns().first() {
      self.return_type.replace([ret.clone(); 1]);
    };
    let return_type = &self.return_type.borrow();

    labels.push_front(
      [match function.function_type.returns().first() {
        Some(ty) => ty.clone(),
        None => ValueTypes::Empty,
      }; 1],
    );

    while let Some(inst) = function.pop() {
      match inst {
        Unreachable => {}
        Nop => {}
        Block => {
          let _ = function.pop_raw_u32()?; // Drop size of block.
          let expect_type = function.pop_value_type()?;
          labels.push_front([expect_type; 1]);
          cxt.push_label();
        }
        Loop => {
          let expect_type = function.pop_value_type()?;
          labels.push_front([expect_type; 1]);
          cxt.push_label();
        }
        If => {
          let _ = cxt.pop_i32()?;
          let _ = function.pop_raw_u32()?; // Drop size of if.
          let _ = function.pop_raw_u32()?; // Drop size of else.
          let expect_type = function.pop_value_type()?;
          labels.push_front([expect_type; 1]);
          cxt.push_label();
        }
        Else => {
          let expect = labels.pop_front().ok_or(TypeError::TypeMismatch)?[0].clone();
          let actual = cxt.pop_type()?;
          if expect != actual {
            return Err(TypeError::TypeMismatch);
          }
          cxt.pop_until_label()?;
          labels.push_front([expect; 1]);
        }
        End => {
          let expect = labels.pop_front().ok_or(TypeError::TypeMismatch)?[0].clone();
          match cxt.pop() {
            Some(Entry::Type(actual)) => {
              if expect != actual {
                return Err(TypeError::TypeMismatch);
              };
              cxt.pop_until_label()?;
            }
            _ => {
              if expect != ValueTypes::Empty {
                return Err(TypeError::TypeMismatch);
              }
            }
          };
        }

        Br => {
          let idx = Indice::from(function.pop_raw_u32()?);
          let expect = labels.get(idx.to_usize()).ok_or(TypeError::UnknownLabel)?[0].clone();
          let actual = cxt.pop_type()?;
          if expect != actual {
            return Err(TypeError::TypeMismatch);
          }
        }
        BrIf => {
          let idx = Indice::from(function.pop_raw_u32()?);
          let expect = labels.get(idx.to_usize()).ok_or(TypeError::UnknownLabel)?[0].clone();
          let actual = cxt.pop_type()?;
          cxt.pop_i32()?;
          if expect != actual {
            return Err(TypeError::TypeMismatch);
          }
        }
        BrTable => {
          let len = function.pop_raw_u32()?;
          let mut indices = vec![];
          for _ in 0..len {
            let idx = function.pop_raw_u32()?;
            indices.push(Indice::from(idx));
          }
          let idx = Indice::from(function.pop_raw_u32()?);
          let expect = labels.get(idx.to_usize()).ok_or(TypeError::UnknownLabel)?[0].clone();
          for i in indices.iter() {
            let actual = labels.get(i.to_usize()).ok_or(TypeError::UnknownLabel)?[0].clone();
            if expect != actual {
              return Err(TypeError::TypeMismatch);
            }
          }
          let actual = cxt.pop_type()?;
          cxt.pop_i32()?;
          if expect != actual {
            return Err(TypeError::TypeMismatch);
          }
        }
        Return => {
          let expect = return_type[0].clone();
          let actual = cxt.pop_type()?;
          if expect != actual {
            return Err(TypeError::TypeMismatch);
          }
          cxt.push(actual);
        }
        Call => {
          let idx = Indice::from(function.pop_raw_u32()?);
          let function_type = self
            .functions
            .get(idx.to_usize())
            .map(|f| f.function_type)
            .ok_or(TypeError::TypeMismatch)?;
          let mut parameters = function_type.parameters().clone();
          while let Some(ty) = parameters.pop() {
            if ty != cxt.pop_type()? {
              return Err(TypeError::TypeMismatch);
            };
          }
          for ty in function_type.returns().iter() {
            cxt.push(ty.clone());
          }
        }
        CallIndirect => {
          let idx = Indice::from(function.pop_raw_u32()?);
          self.tables.first().ok_or(TypeError::UnknownTable(0))?;
          let function_type = self
            .function_types
            .get(idx.to_usize())
            .ok_or_else(|| TypeError::UnknownFunctionType(idx.to_u32()))?;
          let mut parameters = function_type.parameters().clone();
          cxt.pop_i32()?;
          while let Some(ty) = parameters.pop() {
            if ty != cxt.pop_type()? {
              return Err(TypeError::TypeMismatch);
            };
          }
          for ty in function_type.returns().iter() {
            cxt.push(ty.clone());
          }
        }

        I32Const(_) => cxt.push(ValueTypes::I32),
        I64Const(_) => cxt.push(ValueTypes::I64),
        F32Const(_) => cxt.push(ValueTypes::F32),
        F64Const(_) => cxt.push(ValueTypes::F64),

        GetLocal => {
          let idx = Indice::from(function.pop_raw_u32()?);
          let actual = locals.get(idx.to_usize()).ok_or(TypeError::UnknownLocal)?;
          cxt.push(actual.clone());
        }
        SetLocal => {
          let expect = cxt.pop_type()?;
          let idx = Indice::from(function.pop_raw_u32()?);
          let actual = locals.get(idx.to_usize()).ok_or(TypeError::UnknownLocal)?;
          if &expect != actual {
            return Err(TypeError::TypeMismatch);
          }
        }
        TeeLocal => {
          let expect = cxt.pop_type()?;
          let idx = Indice::from(function.pop_raw_u32()?);
          let actual = locals.get(idx.to_usize()).ok_or(TypeError::UnknownLocal)?;
          if &expect != actual {
            return Err(TypeError::TypeMismatch);
          }
          cxt.push(actual.clone());
        }

        GetGlobal => {
          let idx = Indice::from(function.pop_raw_u32()?);
          let ty = self
            .globals
            .get(idx.to_usize())
            .ok_or_else(|| TypeError::UnknownGlobal(idx.to_u32()))
            .map(|(global_type, _)| match global_type {
              GlobalType::Const(ty) | GlobalType::Var(ty) => ty,
            })?;
          cxt.push(ty.clone());
        }
        SetGlobal => {
          let expect = cxt.pop_type()?;
          let idx = function.pop_raw_u32()?;
          let idx: Indice = From::from(idx);
          let ty = self
            .globals
            .get(idx.to_usize())
            .ok_or_else(|| TypeError::UnknownGlobal(idx.to_u32()))
            .and_then(|(global_type, _)| match global_type {
              GlobalType::Var(ty) => Ok(ty),
              GlobalType::Const(_) => Err(TypeError::GlobalIsImmutable),
            })?;
          if &expect != ty {
            return Err(TypeError::TypeMismatch);
          }
        }

        I32Load => self.validate_load(cxt, 32, ValueTypes::I32, function)?,
        I64Load => self.validate_load(cxt, 64, ValueTypes::I64, function)?,
        F32Load => self.validate_load(cxt, 32, ValueTypes::F32, function)?,
        F64Load => self.validate_load(cxt, 64, ValueTypes::F64, function)?,
        I32Load8Sign => self.validate_load(cxt, 8, ValueTypes::I32, function)?,
        I32Load8Unsign => self.validate_load(cxt, 8, ValueTypes::I32, function)?,
        I32Load16Sign => self.validate_load(cxt, 16, ValueTypes::I32, function)?,
        I32Load16Unsign => self.validate_load(cxt, 16, ValueTypes::I32, function)?,
        I64Load8Sign => self.validate_load(cxt, 8, ValueTypes::I64, function)?,
        I64Load8Unsign => self.validate_load(cxt, 8, ValueTypes::I64, function)?,
        I64Load16Sign => self.validate_load(cxt, 16, ValueTypes::I64, function)?,
        I64Load16Unsign => self.validate_load(cxt, 16, ValueTypes::I64, function)?,
        I64Load32Sign => self.validate_load(cxt, 32, ValueTypes::I64, function)?,
        I64Load32Unsign => self.validate_load(cxt, 32, ValueTypes::I64, function)?,

        I32Store => self.validate_store(cxt, 32, &TYPE_I32, function)?,
        I64Store => self.validate_store(cxt, 64, &TYPE_I64, function)?,
        F32Store => self.validate_store(cxt, 32, &TYPE_F32, function)?,
        F64Store => self.validate_store(cxt, 64, &TYPE_F64, function)?,
        I32Store8 => self.validate_store(cxt, 8, &TYPE_I32, function)?,
        I32Store16 => self.validate_store(cxt, 16, &TYPE_I32, function)?,
        I64Store8 => self.validate_store(cxt, 8, &TYPE_I64, function)?,
        I64Store16 => self.validate_store(cxt, 16, &TYPE_I64, function)?,
        I64Store32 => self.validate_store(cxt, 32, &TYPE_I64, function)?,

        MemorySize => {
          self.limits.first().ok_or(TypeError::TypeMismatch)?;
          cxt.push(ValueTypes::I32);
        }
        MemoryGrow => {
          self.limits.first().ok_or(TypeError::TypeMismatch)?;
          cxt.pop_i32()?;
          cxt.push(ValueTypes::I32);
        }

        I32CountLeadingZero => self.validate_unary(cxt)?,
        I32CountTrailingZero => self.validate_unary(cxt)?,
        I32CountNonZero => self.validate_unary(cxt)?,
        I64CountLeadingZero => self.validate_unary(cxt)?,
        I64CountTrailingZero => self.validate_unary(cxt)?,
        I64CountNonZero => self.validate_unary(cxt)?,

        I32Add => bin_op!(cxt),
        I32Sub => bin_op!(cxt),
        I32Mul => bin_op!(cxt),
        I32DivSign => bin_op!(cxt),
        I32DivUnsign => bin_op!(cxt),
        I32RemSign => bin_op!(cxt),
        I32RemUnsign => bin_op!(cxt),
        I32And => bin_op!(cxt),
        I32Or => bin_op!(cxt),
        I32Xor => bin_op!(cxt),
        I32ShiftLeft => bin_op!(cxt),
        I32ShiftRIghtSign => bin_op!(cxt),
        I32ShiftRightUnsign => bin_op!(cxt),
        I32RotateLeft => bin_op!(cxt),
        I32RotateRight => bin_op!(cxt),

        I64Add => bin_op!(cxt),
        I64Sub => bin_op!(cxt),
        I64Mul => bin_op!(cxt),
        I64DivSign => bin_op!(cxt),
        I64DivUnsign => bin_op!(cxt),
        I64RemSign => bin_op!(cxt),
        I64RemUnsign => bin_op!(cxt),
        I64And => bin_op!(cxt),
        I64Or => bin_op!(cxt),
        I64Xor => bin_op!(cxt),
        I64ShiftLeft => bin_op!(cxt),
        I64ShiftRightSign => bin_op!(cxt),
        I64ShiftRightUnsign => bin_op!(cxt),
        I64RotateLeft => bin_op!(cxt),
        I64RotateRight => bin_op!(cxt),

        I32EqualZero => self.validate_test_inst(cxt, &TYPE_I32)?,
        I64EqualZero => self.validate_test_inst(cxt, &TYPE_I64)?,

        Equal => rel_op!(cxt),
        NotEqual => rel_op!(cxt),
        LessThanSign => rel_op!(cxt),
        LessThanUnsign => rel_op!(cxt),
        I32GreaterThanSign => rel_op!(cxt),
        I32GreaterThanUnsign => rel_op!(cxt),
        I32LessEqualSign => rel_op!(cxt),
        I32LessEqualUnsign => rel_op!(cxt),
        I32GreaterEqualSign => rel_op!(cxt),
        I32GreaterEqualUnsign => rel_op!(cxt),

        I64Equal => rel_op!(cxt),
        I64NotEqual => rel_op!(cxt),
        I64LessThanSign => rel_op!(cxt),
        I64LessThanUnSign => rel_op!(cxt),
        I64GreaterThanSign => rel_op!(cxt),
        I64GreaterThanUnSign => rel_op!(cxt),
        I64LessEqualSign => rel_op!(cxt),
        I64LessEqualUnSign => rel_op!(cxt),
        I64GreaterEqualSign => rel_op!(cxt),
        I64GreaterEqualUnSign => rel_op!(cxt),

        F32Equal => rel_op!(cxt),
        F32NotEqual => rel_op!(cxt),
        F32LessThan => rel_op!(cxt),
        F32GreaterThan => rel_op!(cxt),
        F32LessEqual => rel_op!(cxt),
        F32GreaterEqual => rel_op!(cxt),

        F64Equal => rel_op!(cxt),
        F64NotEqual => rel_op!(cxt),
        F64LessThan => rel_op!(cxt),
        F64GreaterThan => rel_op!(cxt),
        F64LessEqual => rel_op!(cxt),
        F64GreaterEqual => rel_op!(cxt),

        F32Abs => self.validate_unary(cxt)?,
        F32Neg => self.validate_unary(cxt)?,
        F32Ceil => self.validate_unary(cxt)?,
        F32Floor => self.validate_unary(cxt)?,
        F32Trunc => self.validate_unary(cxt)?,
        F32Nearest => self.validate_unary(cxt)?,
        F32Sqrt => self.validate_unary(cxt)?,

        F32Add => bin_op!(cxt),
        F32Sub => bin_op!(cxt),
        F32Mul => bin_op!(cxt),
        F32Div => bin_op!(cxt),
        F32Min => bin_op!(cxt),
        F32Max => bin_op!(cxt),
        F32Copysign => bin_op!(cxt),

        F64Abs => self.validate_unary(cxt)?,
        F64Neg => self.validate_unary(cxt)?,
        F64Ceil => self.validate_unary(cxt)?,
        F64Floor => self.validate_unary(cxt)?,
        F64Trunc => self.validate_unary(cxt)?,
        F64Nearest => self.validate_unary(cxt)?,
        F64Sqrt => self.validate_unary(cxt)?,
        F64Add => bin_op!(cxt),
        F64Sub => bin_op!(cxt),
        F64Mul => bin_op!(cxt),
        F64Div => bin_op!(cxt),
        F64Min => bin_op!(cxt),
        F64Max => bin_op!(cxt),
        F64Copysign => bin_op!(cxt),

        Select => {
          cxt.pop_type()?;
          cxt.pop_type()?;
          let operand = cxt.pop_type()?;
          cxt.push(operand);
        }
        DropInst => {
          cxt.pop_type()?;
        }

        // To_convert_name_From
        // macro(cxt, from, to)
        I32WrapI64 => self.validate_convert(cxt, &TYPE_I64, ValueTypes::I32)?,
        I32TruncSignF32 => self.validate_convert(cxt, &TYPE_F32, ValueTypes::I32)?,
        I32TruncUnsignF32 => self.validate_convert(cxt, &TYPE_F32, ValueTypes::I32)?,
        I32TruncSignF64 => self.validate_convert(cxt, &TYPE_F64, ValueTypes::I32)?,
        I32TruncUnsignF64 => self.validate_convert(cxt, &TYPE_F64, ValueTypes::I32)?,
        I64ExtendSignI32 => self.validate_convert(cxt, &TYPE_I32, ValueTypes::I64)?,
        I64ExtendUnsignI32 => self.validate_convert(cxt, &TYPE_I32, ValueTypes::I64)?,
        I64TruncSignF32 => self.validate_convert(cxt, &TYPE_F32, ValueTypes::I64)?,
        I64TruncUnsignF32 => self.validate_convert(cxt, &TYPE_F32, ValueTypes::I64)?,
        I64TruncSignF64 => self.validate_convert(cxt, &TYPE_F64, ValueTypes::I64)?,
        I64TruncUnsignF64 => self.validate_convert(cxt, &TYPE_F64, ValueTypes::I64)?,
        F32ConvertSignI32 => self.validate_convert(cxt, &TYPE_I32, ValueTypes::F32)?,
        F32ConvertUnsignI32 => self.validate_convert(cxt, &TYPE_I32, ValueTypes::F32)?,
        F32ConvertSignI64 => self.validate_convert(cxt, &TYPE_I64, ValueTypes::F32)?,
        F32ConvertUnsignI64 => self.validate_convert(cxt, &TYPE_I64, ValueTypes::F32)?,
        F32DemoteF64 => self.validate_convert(cxt, &TYPE_F64, ValueTypes::F32)?,
        F64ConvertSignI32 => self.validate_convert(cxt, &TYPE_I32, ValueTypes::F64)?,
        F64ConvertUnsignI32 => self.validate_convert(cxt, &TYPE_I32, ValueTypes::F64)?,
        F64ConvertSignI64 => self.validate_convert(cxt, &TYPE_I64, ValueTypes::F64)?,
        F64ConvertUnsignI64 => self.validate_convert(cxt, &TYPE_I64, ValueTypes::F64)?,
        F64PromoteF32 => self.validate_convert(cxt, &TYPE_F32, ValueTypes::F64)?,
        I32ReinterpretF32 => self.validate_convert(cxt, &TYPE_F32, ValueTypes::I32)?,
        I64ReinterpretF64 => self.validate_convert(cxt, &TYPE_F64, ValueTypes::I64)?,
        F32ReinterpretI32 => self.validate_convert(cxt, &TYPE_I32, ValueTypes::F32)?,
        F64ReinterpretI64 => self.validate_convert(cxt, &TYPE_I64, ValueTypes::F64)?,

        RuntimeValue(_) | ExperimentalByte(_) => unreachable!(),
      }
    }
    Ok(())
  }

  pub fn validate(&self) -> Result<()> {
    self.validate_exports()?;
    self.validate_imports()?;
    self.validate_datas()?;
    self.validate_tables()?;
    self.validate_memories()?;
    self.validate_elements()?;
    self.validate_globals()?;
    self.validate_function_types()?;
    self.validate_functions()?;
    self.validate_start()?;
    Ok(())
  }
}
