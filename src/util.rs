pub trait Prepend {
    type Out<Item>;

    fn prepend<Item>(self, item: Item) -> Self::Out<Item>;
}

impl Prepend for () {
    type Out<Item> = Item;

    fn prepend<Item>(self, item: Item) -> Self::Out<Item> {
        item
    }
}

macro_rules! impl_prepend {
    ($(($P:ident, $p:ident)),*) => {
        impl<$($P),*> Prepend for ($($P,)*) {
            type Out<Item> = (Item, $($P),*);

            fn prepend<Item>(self, item: Item) -> Self::Out<Item> {
                let ($($p,)*) = self;
                (item, $($p),*)
            }
        }
    }
}

bevy::ecs::all_tuples!(impl_prepend, 1, 15, P, p);

#[cfg(feature = "nightly")]
mod nightly {
    use crate::util::Prepend;

    impl<A: NonTuple> Prepend for A {
        type Out<Item> = (Item, A);

        fn prepend<Item>(self, item: Item) -> Self::Out<Item> {
            (item, self)
        }
    }

    auto trait NonTuple {}

    impl !NonTuple for () {}

    macro_rules! negimpl_nontuple {
    ($($P:ident),*) => {
        impl<$($P),*> !NonTuple for ($($P,)*) {}
    };
}

    bevy::ecs::all_tuples!(negimpl_nontuple, 1, 15, P);
}
