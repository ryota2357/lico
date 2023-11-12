use crate::{context::Context, Compilable, ContextCompilable};
use vm::code::Code;

#[derive(Default, Clone, Debug, PartialEq)]
pub(super) struct Fragment<'a> {
    code: Vec<Code<'a>>,
    forward_jump_pos: Vec<usize>,
    backward_jump_pos: Vec<usize>,
}

impl<'a> Fragment<'a> {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            forward_jump_pos: Vec::new(),
            backward_jump_pos: Vec::new(),
        }
    }

    pub fn with_compile(compilable: &impl Compilable<'a>) -> Self {
        let mut fragment = Self::new();
        compilable.compile(&mut fragment);
        fragment
    }

    pub fn with_code(code: Vec<Code<'a>>) -> Self {
        Self {
            code,
            forward_jump_pos: Vec::new(),
            backward_jump_pos: Vec::new(),
        }
    }

    pub fn with_compile_with_context(
        compilable: &impl ContextCompilable<'a>,
        context: &mut Context,
    ) -> Self {
        let mut fragment = Self::new();
        compilable.compile(&mut fragment, context);
        fragment
    }

    /// Sets the jump offset for all forward jumps from the end of the fragment.
    pub fn patch_forward_jump(&mut self, offset: isize) {
        let len = self.code.len();
        for pos in self.forward_jump_pos.iter() {
            debug_assert!(matches!(self.code[*pos], Code::Jump(0)));
            self.code[*pos] = Code::Jump((len - *pos - 1) as isize + offset);
        }
        self.forward_jump_pos.clear();
    }

    /// Sets the jump offset for all backward jumps from the beginning of the fragment.
    pub fn patch_backward_jump(&mut self, offset: isize) {
        for pos in self.backward_jump_pos.iter() {
            debug_assert!(matches!(self.code[*pos], Code::Jump(0)));
            self.code[*pos] = Code::Jump(-(*pos as isize) + offset);
        }
        self.backward_jump_pos.clear();
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.code.len()
    }

    #[inline]
    pub fn append(&mut self, code: Code<'a>) -> &mut Self {
        self.code.push(code);
        self
    }

    #[inline]
    pub fn append_many(&mut self, code: impl IntoIterator<Item = Code<'a>>) -> &mut Self {
        self.code.extend(code);
        self
    }

    #[inline]
    pub fn append_compile(&mut self, compilable: &impl Compilable<'a>) -> &mut Self {
        compilable.compile(self);
        self
    }

    #[inline]
    pub fn append_compile_with_context(
        &mut self,
        compilable: &impl ContextCompilable<'a>,
        context: &mut Context,
    ) -> &mut Self {
        compilable.compile(self, context);
        self
    }

    pub fn append_compile_many(&mut self, compilable: &[impl Compilable<'a>]) -> &mut Self {
        for c in compilable.iter() {
            self.append_compile(c);
        }
        self
    }

    pub fn append_forward_jump(&mut self) {
        self.code.push(Code::Jump(0));
        self.forward_jump_pos.push(self.code.len() - 1);
    }

    pub fn append_backward_jump(&mut self) {
        self.code.push(Code::Jump(0));
        self.backward_jump_pos.push(self.code.len() - 1);
    }

    pub fn append_fragment(&mut self, fragment: Fragment<'a>) -> &mut Self {
        let len = self.code.len();
        let Fragment {
            code,
            backward_jump_pos: forward_jump_pos,
            forward_jump_pos: backward_jump_pos,
        } = fragment;

        self.code.extend(code);
        self.backward_jump_pos
            .extend(forward_jump_pos.into_iter().map(|pos| pos + len));
        self.forward_jump_pos
            .extend(backward_jump_pos.into_iter().map(|pos| pos + len));
        self
    }

    pub fn append_fragment_many(
        &mut self,
        fragments: impl IntoIterator<Item = Fragment<'a>>,
    ) -> &mut Self {
        for fragment in fragments {
            self.append_fragment(fragment);
        }
        self
    }

    #[inline]
    pub fn into_code(self) -> Vec<Code<'a>> {
        if cfg!(debug_assertions) {
            for code in self.code.iter() {
                assert!(!matches!(code, Code::Jump(0)));
            }
        }
        self.code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_forward_jump() {
        let mut fragment1 = Fragment {
            code: vec![Code::Jump(0), Code::Jump(0), Code::Jump(0)],
            backward_jump_pos: Vec::new(),
            forward_jump_pos: vec![0, 1, 2],
        };
        let mut fragment2 = fragment1.clone();

        fragment1.patch_forward_jump(3);
        fragment2.patch_forward_jump(-2);

        assert_eq!(
            fragment1.code,
            vec![Code::Jump(5), Code::Jump(4), Code::Jump(3)]
        );
        assert_eq!(
            fragment2.code,
            vec![Code::Jump(0), Code::Jump(-1), Code::Jump(-2)]
        );
        assert_eq!(fragment1.forward_jump_pos, Vec::new());
        assert_eq!(fragment2.forward_jump_pos, Vec::new());
    }

    #[test]
    fn patch_backward_jump() {
        let mut fragment1 = Fragment {
            code: vec![Code::Jump(0), Code::Jump(0), Code::Jump(0)],
            backward_jump_pos: vec![0, 1, 2],
            forward_jump_pos: Vec::new(),
        };
        let mut fragment2 = fragment1.clone();

        fragment1.patch_backward_jump(-3);
        fragment2.patch_backward_jump(2);

        assert_eq!(
            fragment1.code,
            vec![Code::Jump(-3), Code::Jump(-4), Code::Jump(-5)]
        );
        assert_eq!(
            fragment2.code,
            vec![Code::Jump(2), Code::Jump(1), Code::Jump(0)]
        );
        assert_eq!(fragment1.backward_jump_pos, Vec::new());
        assert_eq!(fragment2.backward_jump_pos, Vec::new());
    }

    #[test]
    fn append_fragment() {
        let mut fragment = Fragment {
            code: vec![Code::Jump(0), Code::LoadNil, Code::Jump(0)],
            backward_jump_pos: vec![2],
            forward_jump_pos: vec![0],
        };
        fragment.append_fragment(Fragment {
            code: vec![Code::Jump(0), Code::UnloadTop, Code::Jump(0)],
            backward_jump_pos: vec![0],
            forward_jump_pos: vec![2],
        });

        assert_eq!(
            fragment.code,
            vec![
                Code::Jump(0),   // 0: forward jump
                Code::LoadNil,   // 1:
                Code::Jump(0),   // 2: backward jump
                Code::Jump(0),   // 3: backward jump
                Code::UnloadTop, // 4:
                Code::Jump(0),   // 5: forward jump
            ]
        );
        assert_eq!(fragment.backward_jump_pos, vec![2, 3]);
        assert_eq!(fragment.forward_jump_pos, vec![0, 5]);
    }
}
