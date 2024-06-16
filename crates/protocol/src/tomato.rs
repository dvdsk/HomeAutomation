#[derive(Debug)]
pub struct TomatoLeaf {
    pub val: f32,
    pub device: &'static str,
    pub from_same_device: &'static [Reading],
}

pub type TomatoId = u8;
#[derive(Debug)]
pub enum TomatoItem<'a> {
    Leaf(TomatoLeaf),
    Node(&'a dyn Tomato),
}

pub trait Tomato: core::fmt::Debug {
    fn inner<'a>(&'a self) -> TomatoItem<'a>;
    fn leaf<'a>(&'a self) -> TomatoLeaf
    where
        Self: Sized,
    {
        let mut current = self as &dyn Tomato;
        loop {
            match current.inner() {
                TomatoItem::Node(node) => current = node,
                TomatoItem::Leaf(leaf) => return leaf,
            }
        }
    }
    fn name(&self) -> String {
        let dbg_repr = format!("{:?}", self);
        dbg_repr
            .split_once('(')
            .map(|(name, _)| name)
            .unwrap_or("-")
            .to_string()
    }
    fn id(&self) -> TomatoId;
}

macro_rules! all_nodes {
    ($name:ident; $variant:ident; $($var:ident),*) => {
        impl crate::tomato::Tomato for $name {
            fn inner<'a>(&'a self) -> crate::tomato::TomatoItem<'a> {
                match self {
                    $(
                    $name::$var(inner) => crate::tomato::TomatoItem::Node(inner as &dyn crate::tomato::Tomato)
                    ),*
                }
            }

            fn id(&self) -> crate::tomato::TomatoId {
                $variant::from(self) as crate::tomato::TomatoId
            }
        }
    };
}

pub(crate) use all_nodes;

use crate::Reading;
