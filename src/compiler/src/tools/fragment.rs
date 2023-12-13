use super::*;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Fragment {
    icode: Vec<ICode>,
    forward_jump_pos: Vec<usize>,
    backward_jump_pos: Vec<usize>,
}

impl<'node, 'src: 'node> Fragment {
    pub fn new() -> Self {
        Self {
            icode: Vec::new(),
            forward_jump_pos: Vec::new(),
            backward_jump_pos: Vec::new(),
        }
    }

    pub fn with_compile(
        compilable: &'node impl Compilable<'node, 'src>,
        context: &mut Context<'src>,
    ) -> Result<Self> {
        let mut fragment = Self::new();
        compilable.compile(&mut fragment, context)?;
        Ok(fragment)
    }

    pub fn with_code(code: Vec<ICode>) -> Self {
        Self {
            icode: code,
            forward_jump_pos: Vec::new(),
            backward_jump_pos: Vec::new(),
        }
    }

    /// Sets the jump offset for all forward jumps from the end of the fragment.
    pub fn patch_forward_jump(&mut self, offset: isize) {
        let len = self.icode.len();
        for pos in self.forward_jump_pos.iter() {
            debug_assert!(matches!(self.icode[*pos], ICode::Placeholder));
            self.icode[*pos] = ICode::Jump((len - *pos - 1) as isize + offset);
        }
        self.forward_jump_pos.clear();
    }

    /// Sets the jump offset for all backward jumps from the beginning of the fragment.
    pub fn patch_backward_jump(&mut self, offset: isize) {
        for pos in self.backward_jump_pos.iter() {
            debug_assert!(matches!(self.icode[*pos], ICode::Placeholder));
            self.icode[*pos] = ICode::Jump(-(*pos as isize) + offset);
        }
        self.backward_jump_pos.clear();
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.icode.len()
    }

    #[inline]
    pub fn append(&mut self, code: ICode) -> &mut Self {
        self.icode.push(code);
        self
    }

    #[inline]
    pub fn append_many(&mut self, code: impl IntoIterator<Item = ICode>) -> &mut Self {
        self.icode.extend(code);
        self
    }

    #[inline]
    pub fn append_compile(
        &mut self,
        compilable: &'node impl Compilable<'node, 'src>,
        context: &mut Context<'src>,
    ) -> Result<&mut Self>
    where
        'src: 'node,
    {
        compilable.compile(self, context)?;
        Ok(self)
    }

    pub fn append_compile_many(
        &mut self,
        compilable: impl IntoIterator<Item = &'node (impl Compilable<'node, 'src> + 'node)>,
        context: &mut Context<'src>,
    ) -> Result<&mut Self>
    where
        'src: 'node,
    {
        for c in compilable.into_iter() {
            self.append_compile(c, context)?;
        }
        Ok(self)
    }

    pub fn append_forward_jump(&mut self) {
        self.icode.push(ICode::Placeholder);
        self.forward_jump_pos.push(self.icode.len() - 1);
    }

    pub fn append_backward_jump(&mut self) {
        self.icode.push(ICode::Placeholder);
        self.backward_jump_pos.push(self.icode.len() - 1);
    }

    pub fn append_fragment(&mut self, fragment: Fragment) -> &mut Self {
        let len = self.icode.len();
        let Fragment {
            icode: code,
            backward_jump_pos: forward_jump_pos,
            forward_jump_pos: backward_jump_pos,
        } = fragment;

        self.icode.extend(code);
        self.backward_jump_pos
            .extend(forward_jump_pos.into_iter().map(|pos| pos + len));
        self.forward_jump_pos
            .extend(backward_jump_pos.into_iter().map(|pos| pos + len));
        self
    }

    pub fn append_fragment_many(
        &mut self,
        fragments: impl IntoIterator<Item = Fragment>,
    ) -> &mut Self {
        for fragment in fragments {
            self.append_fragment(fragment);
        }
        self
    }

    #[inline]
    pub fn last(&self) -> Option<&ICode> {
        self.icode.last()
    }

    #[inline]
    pub fn into_code(self) -> Vec<vm::code::Code> {
        use std::rc::Rc;
        use vm::code::{Code, LocalId};

        #[allow(unused_variables)]
        self.icode
            .into_iter()
            .map(|icode| match icode {
                ICode::LoadInt(x) => Code::LoadInt(x),
                ICode::LoadFloat(x) => Code::LoadFloat(x),
                ICode::LoadBool(x) => Code::LoadBool(x),
                ICode::LoadString(x) => Code::LoadString(Rc::new(x)),
                ICode::LoadNil => Code::LoadNil,
                ICode::LoadLocal(id) => Code::LoadLocal(LocalId(*id)),
                ICode::UnloadTop => Code::UnloadTop,
                ICode::SetLocal(id) => Code::SetLocal(LocalId(*id)),
                ICode::MakeLocal => Code::MakeLocal,
                ICode::MakeArray(len) => Code::MakeArray(len),
                ICode::MakeNamed => Code::MakeNamed,
                ICode::MakeTable(len) => Code::MakeTable(len),
                ICode::DropLocal(count) => Code::DropLocal(count),
                ICode::Jump(x) => Code::Jump(x),
                ICode::JumpIfTrue(x) => Code::JumpIfTrue(x),
                ICode::JumpIfFalse(x) => Code::JumpIfFalse(x),
                ICode::CallMethod(name, arg_count, span) => Code::CallMethod(name, arg_count),
                ICode::Call(arg_count, span) => Code::Call(arg_count),
                ICode::SetItem(span) => Code::SetItem,
                ICode::GetItem(span) => Code::GetItem,
                ICode::Add(span) => Code::Add,
                ICode::Sub(span) => Code::Sub,
                ICode::Mul(span) => Code::Mul,
                ICode::Div(span) => Code::Div,
                ICode::Mod(span) => Code::Mod,
                ICode::Pow(span) => Code::Pow,
                ICode::Unm(span) => Code::Unm,
                ICode::Eq(span) => Code::Eq,
                ICode::NotEq(span) => Code::NotEq,
                ICode::Less(span) => Code::Less,
                ICode::LessEq(span) => Code::LessEq,
                ICode::Greater(span) => Code::Greater,
                ICode::GreaterEq(span) => Code::GreaterEq,
                ICode::Concat(span) => Code::Concat,
                ICode::Builtin(instr, arg_count) => Code::Builtin(instr, arg_count),
                ICode::BeginFuncCreation => Code::BeginFuncCreation,
                ICode::AddCapture(id) => Code::AddCapture(LocalId(*id)),
                ICode::AddArgument(x) => Code::AddArgument(x),
                ICode::EndFuncCreation => Code::EndFuncCreation,
                ICode::Placeholder => panic!("Placeholder should not be in the final code."),
                ICode::Nop => Code::Nop,
                ICode::Return => Code::Return,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_forward_jump() {
        let mut fragment1 = Fragment {
            icode: vec![ICode::Placeholder, ICode::Placeholder, ICode::Placeholder],
            backward_jump_pos: Vec::new(),
            forward_jump_pos: vec![0, 1, 2],
        };
        let mut fragment2 = fragment1.clone();

        fragment1.patch_forward_jump(3);
        fragment2.patch_forward_jump(-2);

        assert_eq!(
            fragment1.icode,
            vec![ICode::Jump(5), ICode::Jump(4), ICode::Jump(3)]
        );
        assert_eq!(
            fragment2.icode,
            vec![ICode::Jump(0), ICode::Jump(-1), ICode::Jump(-2)]
        );
        assert_eq!(fragment1.forward_jump_pos, Vec::new());
        assert_eq!(fragment2.forward_jump_pos, Vec::new());
    }

    #[test]
    fn patch_backward_jump() {
        let mut fragment1 = Fragment {
            icode: vec![ICode::Placeholder, ICode::Placeholder, ICode::Placeholder],
            backward_jump_pos: vec![0, 1, 2],
            forward_jump_pos: Vec::new(),
        };
        let mut fragment2 = fragment1.clone();

        fragment1.patch_backward_jump(-3);
        fragment2.patch_backward_jump(2);

        assert_eq!(
            fragment1.icode,
            vec![ICode::Jump(-3), ICode::Jump(-4), ICode::Jump(-5)]
        );
        assert_eq!(
            fragment2.icode,
            vec![ICode::Jump(2), ICode::Jump(1), ICode::Jump(0)]
        );
        assert_eq!(fragment1.backward_jump_pos, Vec::new());
        assert_eq!(fragment2.backward_jump_pos, Vec::new());
    }

    #[test]
    fn append_fragment() {
        let mut fragment = Fragment {
            icode: vec![ICode::Placeholder, ICode::Nop, ICode::Placeholder],
            backward_jump_pos: vec![2],
            forward_jump_pos: vec![0],
        };
        fragment.append_fragment(Fragment {
            icode: vec![ICode::Placeholder, ICode::UnloadTop, ICode::Placeholder],
            backward_jump_pos: vec![0],
            forward_jump_pos: vec![2],
        });

        assert_eq!(
            fragment.icode,
            vec![
                ICode::Placeholder, // 0: forward jump
                ICode::Nop,         // 1:
                ICode::Placeholder, // 2: backward jump
                ICode::Placeholder, // 3: backward jump
                ICode::UnloadTop,   // 4:
                ICode::Placeholder, // 5: forward jump
            ]
        );
        assert_eq!(fragment.backward_jump_pos, vec![2, 3]);
        assert_eq!(fragment.forward_jump_pos, vec![0, 5]);
    }
}
