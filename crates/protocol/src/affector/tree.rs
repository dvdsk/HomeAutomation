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
        let split = dbg_repr.find('(').unwrap_or(dbg_repr.len());
        dbg_repr.split_at(split).0.to_string()
    }
    fn branch_id(&self) -> Id;
}

macro_rules! impl_zero {
    ($name:ident; $($var:ident),*) => {
        impl crate::IsSameAs for $name {
            #[allow(unreachable_patterns)]
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
        impl crate::affector::tree::Tree for $name {
            fn inner(&self) -> crate::affector::tree::Item<'_> {
                match self {
                    $(
                    $name::$var(inner) => crate::affector::tree::Item::Node(inner as &dyn crate::affector::tree::Tree)
                    ),*
                }
            }

            fn branch_id(&self) -> crate::affector::tree::Id {
                $variant::from(self) as crate::affector::tree::Id
            }

        }
        crate::affector::tree::impl_zero!{$name; $($var),*}
    };
}
pub(crate) use all_nodes;
