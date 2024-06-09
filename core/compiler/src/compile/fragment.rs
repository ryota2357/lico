use super::*;
use smallvec::SmallVec;

#[derive(Default, Clone, Debug, PartialEq)]
pub(crate) struct Fragment {
    code: Vec<ICodeSource>,
    forward_jump_pos: SmallVec<u32, 4>,
    backward_jump_pos: SmallVec<u32, 4>,
}

impl Fragment {
    pub(crate) const fn new() -> Self {
        Fragment {
            code: Vec::new(),
            forward_jump_pos: SmallVec::new(),
            backward_jump_pos: SmallVec::new(),
        }
    }

    pub(crate) fn with_compile<'node, 'src: 'node>(
        compilable: &'node impl Compilable<'node, 'src>,
        ctx: &mut Context<'src>,
    ) -> Self {
        let mut fragment = Self::new();
        compilable.compile(&mut fragment, ctx);
        fragment
    }

    /// Sets the jump offset for all forward jumps from the end of the fragment.
    pub(crate) fn patch_forward_jump(&mut self, offset: isize) {
        let len = self.code.len();
        for pos in self.forward_jump_pos.iter() {
            let pos = *pos as usize;
            debug_assert_eq!(self.code[pos], ICodeSource::Tombstone);
            self.code[pos] = ICodeSource::Jump((len - pos - 1) as isize + offset);
        }
        self.forward_jump_pos.clear();
    }

    /// Sets the jump offset for all backward jumps from the beginning of the fragment.
    pub(crate) fn patch_backward_jump(&mut self, offset: isize) {
        for pos in self.backward_jump_pos.iter() {
            let pos = *pos as usize;
            debug_assert_eq!(self.code[pos], ICodeSource::Tombstone);
            self.code[pos] = ICodeSource::Jump(-(pos as isize) + offset);
        }
        self.backward_jump_pos.clear();
    }

    pub(crate) fn len(&self) -> usize {
        self.code.len()
    }

    pub(crate) fn append(&mut self, code: ICodeSource) -> &mut Self {
        self.code.push(code);
        self
    }

    pub(crate) fn append_many<I>(&mut self, code: I) -> &mut Self
    where
        I: IntoIterator<Item = ICodeSource>,
    {
        self.code.extend(code);
        self
    }

    pub(crate) fn append_compile<'node, 'src: 'node>(
        &mut self,
        compilable: &'node impl Compilable<'node, 'src>,
        ctx: &mut Context<'src>,
    ) -> &mut Self {
        compilable.compile(self, ctx);
        self
    }

    pub(crate) fn append_forward_jump(&mut self) {
        self.code.push(ICodeSource::Tombstone);
        let pos = self.code.len() - 1;
        self.forward_jump_pos.push(pos as u32);
    }

    pub(crate) fn append_backward_jump(&mut self) {
        self.code.push(ICodeSource::Tombstone);
        let pos = self.code.len() - 1;
        self.backward_jump_pos.push(pos as u32);
    }

    pub(crate) fn append_fragment(&mut self, fragment: Fragment) -> &mut Self {
        let len = self.code.len() as u32;
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

    pub(crate) fn finish(self) -> Vec<ICodeSource> {
        assert!(
            self.forward_jump_pos.is_empty(),
            "[BUG] Remaining unpatched forward jumps."
        );
        assert!(
            self.backward_jump_pos.is_empty(),
            "[BUG] Remaining unpatched backward jumps."
        );
        assert!(
            self.code.len() <= u32::MAX as usize,
            "Unexpectedly large IL fragment."
        );
        self.code
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use smallvec::smallvec;

    use super::ICodeSource::*;

    #[test]
    fn patch_forward_jump() {
        let mut fragment1 = Fragment {
            code: vec![Tombstone, Tombstone, Tombstone],
            backward_jump_pos: SmallVec::new(),
            forward_jump_pos: smallvec![0, 1, 2],
        };
        let mut fragment2 = fragment1.clone();

        fragment1.patch_forward_jump(3);
        fragment2.patch_forward_jump(-2);

        assert_eq!(
            fragment1.code,
            [Jump(5), Jump(4), Jump(3)].into_iter().collect::<Vec<_>>()
        );
        assert_eq!(
            fragment2.code,
            [Jump(0), Jump(-1), Jump(-2)]
                .into_iter()
                .collect::<Vec<_>>()
        );
        assert_eq!(fragment1.forward_jump_pos, SmallVec::<_, 4>::new());
        assert_eq!(fragment2.forward_jump_pos, SmallVec::<_, 4>::new());
    }

    #[test]
    fn patch_backward_jump() {
        let mut fragment1 = Fragment {
            code: vec![Tombstone, Tombstone, Tombstone],
            backward_jump_pos: smallvec![0, 1, 2],
            forward_jump_pos: SmallVec::new(),
        };
        let mut fragment2 = fragment1.clone();

        fragment1.patch_backward_jump(-3);
        fragment2.patch_backward_jump(2);

        assert_eq!(
            fragment1.code,
            [Jump(-3), Jump(-4), Jump(-5)]
                .into_iter()
                .collect::<Vec<_>>()
        );
        assert_eq!(
            fragment2.code,
            [Jump(2), Jump(1), Jump(0)].into_iter().collect::<Vec<_>>()
        );
        assert_eq!(fragment1.backward_jump_pos, SmallVec::<_, 4>::new());
        assert_eq!(fragment2.backward_jump_pos, SmallVec::<_, 4>::new());
    }

    #[test]
    fn append_fragment() {
        let mut fragment = Fragment {
            code: vec![Tombstone, Unload, Tombstone],
            backward_jump_pos: smallvec![2],
            forward_jump_pos: smallvec![0],
        };
        fragment.append_fragment(Fragment {
            code: vec![Tombstone, Leave, Tombstone],
            backward_jump_pos: smallvec![0],
            forward_jump_pos: smallvec![2],
        });

        assert_eq!(
            fragment.code,
            vec![
                Tombstone, // 0: forward jump
                Unload,    // 1:
                Tombstone, // 2: backward jump
                Tombstone, // 3: backward jump
                Leave,     // 4:
                Tombstone, // 5: forward jump
            ]
        );
        assert_eq!(fragment.backward_jump_pos, SmallVec::<_, 4>::from([2, 3]));
        assert_eq!(fragment.forward_jump_pos, SmallVec::<_, 4>::from([0, 5]));
    }
}
