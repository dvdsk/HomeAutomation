use core::any::Any;

use super::Info;

pub type Id = u8;
#[derive(Debug)]
pub enum Item<'a> {
    Leaf(Info),
    Node(&'a dyn Tree),
}

pub enum ItemMut<'a> {
    Leaf(&'a mut dyn Any),
    Node(&'a mut dyn Tree),
}

pub trait Tree: core::fmt::Debug {
    #[must_use]
    fn inner(&self) -> Item<'_>;
    fn inner_mut(&mut self) -> ItemMut<'_>;

    fn info(&self) -> Info
    where
        Self: Sized,
    {
        let mut current = self as &dyn Tree;
        loop {
            match current.inner() {
                Item::Node(node) => current = node,
                Item::Leaf(info) => return info,
            }
        }
    }
    fn value_mut(&mut self) -> &mut (dyn Any + 'static)
    where
        Self: Sized,
    {
        let mut current = self as &mut dyn Tree;
        loop {
            match current.inner_mut() {
                ItemMut::Node(node) => current = node,
                ItemMut::Leaf(value) => return value,
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
    #[must_use]
    fn branch_id(&self) -> Id;
}

macro_rules! field_as_any {
    ($self:ident, $($field_name:ident),+) => {
        match $self {
            $(
                Reading::$field_name(inner) => inner as &mut dyn core::any::Any,
            )+
        }
    };
}
pub(crate) use field_as_any;

macro_rules! impl_zero {
    ($name:ident; $($var:ident),*) => {
        impl $crate::IsSameAs for $name {
            fn is_same_as(&self, other: &Self) -> bool {
                #[allow(unused_imports)]
                use $crate::IsSameAs;
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
            fn inner_mut(&mut self) -> crate::reading::tree::ItemMut<'_> {
                match self {
                    $(
                    $name::$var(inner) => crate::reading::tree::ItemMut::Node(inner as &mut dyn crate::reading::tree::Tree)
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
