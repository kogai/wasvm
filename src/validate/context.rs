use super::error::{Result, TypeError};
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
use decode::Section;
use function::FunctionType;
use inst::{Indice, Inst};
// use module::{FUNCTION_DESCRIPTOR, GLOBAL_DESCRIPTOR, MEMORY_DESCRIPTOR, TABLE_DESCRIPTOR};
// use trap::Trap;
// use value::Values;
use value_type::ValueTypes;

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

  fn last(&self) -> Option<Entry> {
    self.0.borrow().last().cloned()
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
}

pub struct Context<'a> {
  function_types: &'a Vec<FunctionType>,
  functions: Vec<Function<'a>>,
  //  exports: ExternalInterfaces,
  //  codes: Vec<Result<(Vec<Inst>, Vec<ValueTypes>)>>,
  //  datas: Vec<Data>,
  //  limits: Vec<Limit>,
  //  tables: Vec<TableType>,
  //  globals: Vec<(GlobalType, Vec<Inst>)>,
  //  elements: Vec<Element>,
  //  customs: Vec<(String, Vec<u8>)>,
  //  imports: ExternalInterfaces,
  //  start: Option<u32>,
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
      locals: RefCell::new(Vec::new()),
      labels: RefCell::new(VecDeque::new()),
      return_type: RefCell::new([ValueTypes::Empty; 1]),
    })
  }

  fn validate_function_types(&self) -> Result<()> {
    for fy in self.function_types.iter() {
      if fy.returns().len() > 1 {
        return Err(TypeError::TypeMismatch);
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

  fn validate_function(&self, function: &Function) -> Result<()> {
    use self::Inst::*;
    let cxt = &function.type_stack;
    let labels = &mut self.labels.borrow_mut();
    let locals = &mut self.locals.borrow_mut();
    let return_type = &mut self.return_type.borrow_mut();

    labels.push_front(
      [match function.function_type.returns().first() {
        Some(ty) => ty.clone(),
        None => ValueTypes::Empty,
      }; 1],
    );

    while let Some(inst) = function.pop() {
      // println!("{:?}", inst);
      match inst {
        Unreachable => unimplemented!(),
        Nop => unimplemented!(),
        Block(_) => {
          let expect_type = function.pop_value_type()?;
          labels.push_front([expect_type; 1]);
          cxt.push_label();
        }
        Loop(_) => {
          let expect_type = function.pop_value_type()?;
          labels.push_front([expect_type; 1]);
          cxt.push_label();
        }
        If(_, _) => {
          let _ = cxt.pop_i32()?;
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
          let actual = cxt.pop_type()?;
          if expect != actual {
            return Err(TypeError::TypeMismatch);
          }
          cxt.pop_until_label()?;
          // println!("{:?}", labels);
        }

        Br(idx) => {
          let expect = labels.get(*idx as usize).ok_or(TypeError::TypeMismatch)?[0].clone();
          let actual = cxt.pop_type()?;
          if expect != actual {
            return Err(TypeError::TypeMismatch);
          }
        }
        BrIf(idx) => {
          let expect = labels.get(*idx as usize).ok_or(TypeError::TypeMismatch)?[0].clone();
          let actual = cxt.pop_type()?;
          cxt.pop_i32()?;
          if expect != actual {
            return Err(TypeError::TypeMismatch);
          }
        }
        BrTable(indices, idx) => {
          let expect = labels.get(*idx as usize).ok_or(TypeError::TypeMismatch)?[0].clone();
          for i in indices.iter() {
            let actual = labels.get(*i as usize).ok_or(TypeError::TypeMismatch)?[0].clone();
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
        }
        Call(_) => unimplemented!(),
        CallIndirect(_) => unimplemented!(),

        I32Const(_) => cxt.push(ValueTypes::I32),
        I64Const(_) => cxt.push(ValueTypes::I64),
        F32Const(_) => cxt.push(ValueTypes::F32),
        F64Const(_) => cxt.push(ValueTypes::F64),

        GetLocal(idx) => {
          let actual = locals.get(*idx as usize).ok_or(TypeError::TypeMismatch)?;
          cxt.push(actual.clone());
        }
        SetLocal(idx) => {
          let expect = cxt.pop_type()?;
          let actual = locals.get(*idx as usize).ok_or(TypeError::TypeMismatch)?;
          if &expect != actual {
            return Err(TypeError::TypeMismatch);
          }
        }
        TeeLocal(_) => unimplemented!(),
        GetGlobal(_) => unimplemented!(),
        SetGlobal(_) => unimplemented!(),

        I32Load(_, _) => unimplemented!(),
        I64Load(_, _) => unimplemented!(),
        F32Load(_, _) => unimplemented!(),
        F64Load(_, _) => unimplemented!(),
        I32Load8Sign(_, _) => unimplemented!(),
        I32Load8Unsign(_, _) => unimplemented!(),
        I32Load16Sign(_, _) => unimplemented!(),
        I32Load16Unsign(_, _) => unimplemented!(),
        I64Load8Sign(_, _) => unimplemented!(),
        I64Load8Unsign(_, _) => unimplemented!(),
        I64Load16Sign(_, _) => unimplemented!(),
        I64Load16Unsign(_, _) => unimplemented!(),
        I64Load32Sign(_, _) => unimplemented!(),
        I64Load32Unsign(_, _) => unimplemented!(),
        I32Store(_, _) => unimplemented!(),
        I64Store(_, _) => unimplemented!(),
        F32Store(_, _) => unimplemented!(),
        F64Store(_, _) => unimplemented!(),
        I32Store8(_, _) => unimplemented!(),
        I32Store16(_, _) => unimplemented!(),
        I64Store8(_, _) => unimplemented!(),
        I64Store16(_, _) => unimplemented!(),
        I64Store32(_, _) => unimplemented!(),
        MemorySize => unimplemented!(),
        MemoryGrow => unimplemented!(),

        I32CountLeadingZero => unimplemented!(),
        I32CountTrailingZero => unimplemented!(),
        I32CountNonZero => unimplemented!(),
        I32Add => bin_op!(cxt),
        I32Sub => bin_op!(cxt),
        I32Mul => bin_op!(cxt),
        I32DivSign => unimplemented!(),
        I32DivUnsign => unimplemented!(),
        I32RemSign => unimplemented!(),
        I32RemUnsign => unimplemented!(),
        I32And => unimplemented!(),
        I32Or => unimplemented!(),
        I32Xor => unimplemented!(),
        I32ShiftLeft => unimplemented!(),
        I32ShiftRIghtSign => unimplemented!(),
        I32ShiftRightUnsign => unimplemented!(),
        I32RotateLeft => unimplemented!(),
        I32RotateRight => unimplemented!(),

        I64CountLeadingZero => unimplemented!(),
        I64CountTrailingZero => unimplemented!(),
        I64CountNonZero => unimplemented!(),
        I64Add => unimplemented!(),
        I64Sub => unimplemented!(),
        I64Mul => unimplemented!(),
        I64DivSign => unimplemented!(),
        I64DivUnsign => unimplemented!(),
        I64RemSign => unimplemented!(),
        I64RemUnsign => unimplemented!(),
        I64And => unimplemented!(),
        I64Or => unimplemented!(),
        I64Xor => unimplemented!(),
        I64ShiftLeft => unimplemented!(),
        I64ShiftRightSign => unimplemented!(),
        I64ShiftRightUnsign => unimplemented!(),
        I64RotateLeft => unimplemented!(),
        I64RotateRight => unimplemented!(),

        I32EqualZero => {
          let ty = cxt.pop_i32()?;
          cxt.push(ty);
        }
        Equal => unimplemented!(),
        NotEqual => unimplemented!(),
        LessThanSign => unimplemented!(),
        LessThanUnsign => unimplemented!(),
        I32GreaterThanSign => unimplemented!(),
        I32GreaterThanUnsign => unimplemented!(),
        I32LessEqualSign => unimplemented!(),
        I32LessEqualUnsign => unimplemented!(),
        I32GreaterEqualSign => unimplemented!(),
        I32GreaterEqualUnsign => unimplemented!(),

        I64EqualZero => unimplemented!(),
        I64Equal => unimplemented!(),
        I64NotEqual => unimplemented!(),
        I64LessThanSign => unimplemented!(),
        I64LessThanUnSign => unimplemented!(),
        I64GreaterThanSign => unimplemented!(),
        I64GreaterThanUnSign => unimplemented!(),
        I64LessEqualSign => unimplemented!(),
        I64LessEqualUnSign => unimplemented!(),
        I64GreaterEqualSign => unimplemented!(),
        I64GreaterEqualUnSign => unimplemented!(),

        F32Equal => unimplemented!(),
        F32NotEqual => unimplemented!(),
        F32LessThan => unimplemented!(),
        F32GreaterThan => unimplemented!(),
        F32LessEqual => unimplemented!(),
        F32GreaterEqual => unimplemented!(),
        F64Equal => unimplemented!(),
        F64NotEqual => unimplemented!(),
        F64LessThan => unimplemented!(),
        F64GreaterThan => unimplemented!(),
        F64LessEqual => unimplemented!(),
        F64GreaterEqual => unimplemented!(),

        F32Abs => unimplemented!(),
        F32Neg => unimplemented!(),
        F32Ceil => unimplemented!(),
        F32Floor => unimplemented!(),
        F32Trunc => unimplemented!(),
        F32Nearest => unimplemented!(),
        F32Sqrt => unimplemented!(),
        F32Add => unimplemented!(),
        F32Sub => unimplemented!(),
        F32Mul => unimplemented!(),
        F32Div => unimplemented!(),
        F32Min => unimplemented!(),
        F32Max => unimplemented!(),
        F32Copysign => unimplemented!(),

        F64Abs => unimplemented!(),
        F64Neg => unimplemented!(),
        F64Ceil => unimplemented!(),
        F64Floor => unimplemented!(),
        F64Trunc => unimplemented!(),
        F64Nearest => unimplemented!(),
        F64Sqrt => unimplemented!(),
        F64Add => unimplemented!(),
        F64Sub => unimplemented!(),
        F64Mul => unimplemented!(),
        F64Div => unimplemented!(),
        F64Min => unimplemented!(),
        F64Max => unimplemented!(),
        F64Copysign => unimplemented!(),

        Select => unimplemented!(),
        DropInst => {
          cxt.pop().ok_or(TypeError::TypeMismatch)?;
        }
        I32WrapI64 => unimplemented!(),

        I32TruncSignF32 => unimplemented!(),
        I32TruncUnsignF32 => unimplemented!(),
        I32TruncSignF64 => unimplemented!(),
        I32TruncUnsignF64 => unimplemented!(),
        I64ExtendSignI32 => unimplemented!(),
        I64ExtendUnsignI32 => unimplemented!(),
        I64TruncSignF32 => unimplemented!(),
        I64TruncUnsignF32 => unimplemented!(),
        I64TruncSignF64 => unimplemented!(),
        I64TruncUnsignF64 => unimplemented!(),
        F32ConvertSignI32 => unimplemented!(),
        F32ConvertUnsignI32 => unimplemented!(),
        F32ConvertSignI64 => unimplemented!(),
        F32ConvertUnsignI64 => unimplemented!(),
        F32DemoteF64 => unimplemented!(),
        F64ConvertSignI32 => unimplemented!(),
        F64ConvertUnsignI32 => unimplemented!(),
        F64ConvertSignI64 => unimplemented!(),
        F64ConvertUnsignI64 => unimplemented!(),
        F64PromoteF32 => unimplemented!(),
        I32ReinterpretF32 => unimplemented!(),
        I64ReinterpretF64 => unimplemented!(),
        F32ReinterpretI32 => unimplemented!(),
        F64ReinterpretI64 => unimplemented!(),

        RuntimeValue(_) => unimplemented!(),
      }
    }
    // Ok([ValueTypes::Empty])
    Ok(())
  }

  pub fn validate(&self) -> Result<()> {
    // let grouped_imports = self.module.imports.group_by_kind();
    // let imports_function = grouped_imports.get(&FUNCTION_DESCRIPTOR)?;
    // let imports_table = grouped_imports.get(&TABLE_DESCRIPTOR)?;
    // let imports_memory = grouped_imports.get(&MEMORY_DESCRIPTOR)?;
    // let imports_global = grouped_imports.get(&GLOBAL_DESCRIPTOR)?;
    self.validate_function_types()?;
    self.validate_functions()?;

    // let global_instances =
    //   GlobalInstances::new_with_external(globals, &exports, &imports_global, &external_modules)?;

    // unimplemented!(
    //   "Type system(Also called as `validation`) not implemented yet.\n{:#?}",
    //   self.module
    // );
    Ok(())
  }
}
