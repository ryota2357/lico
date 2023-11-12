#[derive(Clone, Debug, PartialEq)]
pub(super) struct Context {
    block_local_count: Vec<usize>,
    loop_local_count: Vec<usize>,
    allow_loop_jump: usize,
}

impl Context {
    pub fn new() -> Self {
        Self {
            block_local_count: Vec::new(),
            loop_local_count: Vec::new(),
            allow_loop_jump: 0,
        }
    }

    pub fn inc_local_count(&mut self) {
        if let Some(count) = self.block_local_count.last_mut() {
            *count += 1;
        }
        if let Some(count) = self.loop_local_count.last_mut() {
            *count += 1;
        }
    }

    #[inline]
    pub fn start_block(&mut self) {
        self.block_local_count.push(0);
    }

    #[inline]
    pub fn start_loop_section(&mut self) {
        self.loop_local_count.push(0);
        self.allow_loop_jump += 1;
    }

    pub fn end_block(&mut self) {
        debug_assert!(
            !self.block_local_count.is_empty(),
            "Block local count is empty"
        );
        let block_cnt = self.block_local_count.pop().unwrap();

        if let Some(count) = self.loop_local_count.last_mut() {
            debug_assert!(
                *count >= block_cnt,
                "Loop local count is less than block local count"
            );
            *count -= block_cnt;
        }
    }

    #[inline]
    pub fn end_loop_section(&mut self) {
        debug_assert!(
            !self.loop_local_count.is_empty() && self.allow_loop_jump > 0,
            "Loop local count is empty"
        );
        self.loop_local_count.pop();
        self.allow_loop_jump -= 1;
    }

    /// Returns the number of locals in the current loop section.
    /// Returns `None` if there is no current loop section.
    #[inline]
    pub fn get_loop_local_count(&self) -> Option<usize> {
        self.loop_local_count.last().copied()
    }

    #[inline]
    pub fn get_block_local_count(&self) -> Result<usize, ()> {
        self.block_local_count.last().copied().ok_or(())
    }
}
