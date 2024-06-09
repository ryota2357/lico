use super::Object;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RustFunction(fn(&[Object]) -> Result<Object, String>);

impl RustFunction {
    pub const fn new(f: fn(&[Object]) -> Result<Object, String>) -> Self {
        RustFunction(f)
    }

    // エラーハンドリング周りの処理をやりたい
    // self.0 の Result を Stringじゃなくしたり、callee情報を持たせたりしたい
    pub fn call(&self, args: &[Object]) -> Result<Object, String> {
        (self.0)(args)
    }
}
