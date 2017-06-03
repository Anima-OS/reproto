pub use super::errors::*;
pub use super::models::*;

pub trait ForContext {
    type Output;

    fn for_context(&self, context: &str) -> Self::Output;
}

// TODO: borrow content
impl ForContext for Vec<RpToken<Code>> {
    type Output = Vec<RpToken<Code>>;

    fn for_context(&self, context: &str) -> Self::Output {
        self.iter().filter(|c| c.inner.context == context).map(|c| c.clone()).collect()
    }
}
