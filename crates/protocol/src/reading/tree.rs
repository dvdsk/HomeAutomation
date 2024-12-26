use super::Info;

pub type Id = u8;
#[derive(Debug)]
pub enum Item<'a> {
    Leaf(Info),
    Node(&'a dyn Tree),
}

pub trait Tree: core::fmt::Debug {
    fn inner(&self) -> Item<'_>;
    fn leaf(&self) -> Info
    where
        Self: Sized,
    {
        let mut current = self as &dyn Tree;
        loop {
            match current.inner() {
                Item::Node(node) => current = node,
                Item::Leaf(leaf) => return leaf,
            }
        }
    }
    fn name(&self) -> String {
        let dbg_repr = format!("{self:?}");
        dbg_repr
            .split_once('(')
            .map_or("_", |(name, _)| name)
            .to_string()
    }
    fn branch_id(&self) -> Id;
}

macro_rules! impl_zero {
    ($name:ident; $($var:ident),*) => {
        impl crate::IsSameAs for $name {
            #[must_use]
            fn is_same_as(&self, other: &Self) -> bool {
                match (self, other) {
                    $(($name::$var(a), $name::$var(b)) => a.is_same_as(b),)*
                    (_, _) => false,
                }
            }
        }

    };
}
pub(crate) use impl_zero;

macro_rules! all_nodes {
    ($name:ident; $variant:ident; $($var:ident),*) => {
        impl crate::reading::tree::Tree for $name {
            fn inner(&self) -> crate::reading::tree::Item<'_> {
                match self {
                    $(
                    $name::$var(inner) => crate::reading::tree::Item::Node(inner as &dyn crate::reading::tree::Tree)
                    ),*
                }
            }

            fn branch_id(&self) -> crate::reading::tree::Id {
                $variant::from(self) as crate::reading::tree::Id
            }

        }
        crate::reading::tree::impl_zero!{$name; $($var),*}
    };
}
pub(crate) use all_nodes;

