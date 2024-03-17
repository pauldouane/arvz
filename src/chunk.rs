#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Chunk {
    Main,
    ContextInformations,
    Input,
    Table,
    StatusBar,
}
