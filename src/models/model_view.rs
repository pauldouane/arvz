pub struct ModelView<T: ?Sized> {
    content: T,
}

impl<T> ModelView<T> {}

impl<T> ModelView<T> {
    pub fn new(content: T) -> Self {
        Self { content }
    }
}
