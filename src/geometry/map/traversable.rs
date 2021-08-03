/// Can a visitor move through this map tile?
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Traversable {
    /// Obstructed tiles cannot be moved into.
    Obstructed,
    /// Free tiles can be moved through.
    Free,
    /// Halt tiles can be moved into, but not past.
    Halt,
}
