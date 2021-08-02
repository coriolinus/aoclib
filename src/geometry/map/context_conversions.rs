use crate::geometry::Point;

/// Safe fast value-to-value conversion which consumes the input value and references some context.
///
/// This trait should be implemented in preference to [`ContextInto`][ContextInto].
pub trait ContextFrom<T> {
    type Context;

    fn ctx_from(t: T, position: Point, context: &Self::Context) -> Self;
}

impl<A, B> ContextFrom<A> for B
where
    B: From<A>,
{
    type Context = ();

    fn ctx_from(a: A, _position: Point, _context: &()) -> B {
        B::from(a)
    }
}

/// Safe fast value-to-value conversion which consumes the input value and references some context.
///
/// This differs from [`Into`][std::convert::Into] in that it requires a context.
/// Also, because of a blanket implementation, it cannot be manually implemented for a given `T`
/// for any type which also implements `Into<T>`.
pub trait ContextInto<T> {
    type Context;

    fn ctx_into(self, position: Point, context: &Self::Context) -> T;
}

impl<A, B> ContextInto<B> for A
where
    B: ContextFrom<A>,
{
    type Context = <B as ContextFrom<A>>::Context;

    fn ctx_into(self, position: Point, context: &Self::Context) -> B {
        B::ctx_from(self, position, context)
    }
}
