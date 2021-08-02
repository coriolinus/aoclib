use crate::geometry::Point;

/// A* State
// https://doc.rust-lang.org/std/collections/binary_heap/#examples
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) struct AStarNode {
    pub cost: u32,
    pub position: Point,
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for AStarNode {
    fn cmp(&self, other: &AStarNode) -> std::cmp::Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &AStarNode) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
