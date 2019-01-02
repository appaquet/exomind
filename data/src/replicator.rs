use exocore_common::range::Range;

struct Replicator<O, K, I>
where
    O: Ord + Copy + Eq,
    K: Ord + Copy + Eq,
    I: Serializable,
{
    phantom: std::marker::PhantomData<(O, K, I)>,
}

impl<O, K, I> Replicator<O, K, I>
where
    O: Ord + Copy + Eq,
    K: Ord + Copy + Eq,
    I: Serializable,
{
}

trait Container<O, K, I>
where
    O: Ord + Copy + Eq,
    K: Ord + Copy + Eq,
    I: Serializable,
{
    fn complete_range(&self) -> Range<O> {
        unimplemented!()
    }

    fn elements_iter(&self) -> &dyn Iterator<Item = I>;
}

struct Slice<O, K, I>
where
    O: Ord + Copy + Eq,
    K: Ord + Copy + Eq,
    I: Serializable,
{
    range: Range<O>,
    phantom: std::marker::PhantomData<(O, K, I)>,
}

struct Message {}

trait Serializable {}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestItem {
        order: u32,
        key: String,
    }

    //    impl Container<u32> for TestItem {
    //
    //    }

}
