use foundation::object::Object;

pub(crate) struct Stack(Vec<Object>);

impl Stack {
    pub(crate) const fn new() -> Self {
        Stack(Vec::new())
    }

    pub(crate) fn push(&mut self, value: Object) {
        self.0.push(value);
    }

    pub(crate) fn pop(&mut self) -> Object {
        self.0
            .pop()
            .expect("[BUG] Stack must have at least one value at pop.")
    }

    pub(crate) fn pop2(&mut self) -> (Object, Object) {
        let b = self.pop();
        let a = self.pop();
        (a, b)
    }

    pub(crate) fn pop3(&mut self) -> (Object, Object, Object) {
        let c = self.pop();
        let b = self.pop();
        let a = self.pop();
        (a, b, c)
    }
}

pub(crate) struct LeaveHook(Vec<Hook>);
pub(crate) struct Hook {
    pub(crate) ra: usize,
    pub(crate) post_exec: Option<Box<dyn FnOnce(Object) -> Result<Object, ()>>>,
}

impl LeaveHook {
    pub(crate) const fn new() -> Self {
        LeaveHook(Vec::new())
    }

    pub(crate) fn set(
        &mut self,
        value: usize,
        post_exec: Option<Box<dyn FnOnce(Object) -> Result<Object, ()>>>,
    ) {
        self.0.push(Hook {
            ra: value,
            post_exec,
        });
    }

    pub(crate) fn pop(&mut self) -> Option<Hook> {
        self.0.pop()
    }
}
