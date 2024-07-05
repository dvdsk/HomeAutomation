use crate::{Device, Reading, Unit};

#[derive(Debug)]
pub struct ReadingInfo {
    pub val: f32,
    pub device: Device,
    pub resolution: f32,
    pub range: core::ops::Range<f32>,
    pub unit: Unit,
    pub description: &'static str,
}

impl ReadingInfo {
    pub fn from_same_device(&self) -> &'static [Reading] {
        self.device.affected_readings()
    }
}

pub type Id = u8;
#[derive(Debug)]
pub enum Item<'a> {
    Leaf(ReadingInfo),
    Node(&'a dyn Tree),
}

pub trait Tree: core::fmt::Debug {
    fn inner<'a>(&'a self) -> Item<'a>;
    fn leaf<'a>(&'a self) -> ReadingInfo
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
        let dbg_repr = format!("{:?}", self);
        dbg_repr
            .split_once('(')
            .map(|(name, _)| name)
            .unwrap_or("-")
            .to_string()
    }
    fn id(&self) -> Id;
}

macro_rules! all_nodes {
    ($name:ident; $variant:ident; $($var:ident),*) => {
        impl crate::reading_tree::Tree for $name {
            fn inner<'a>(&'a self) -> crate::reading_tree::Item<'a> {
                match self {
                    $(
                    $name::$var(inner) => crate::reading_tree::Item::Node(inner as &dyn crate::reading_tree::Tree)
                    ),*
                }
            }

            fn id(&self) -> crate::reading_tree::Id {
                $variant::from(self) as crate::reading_tree::Id
            }
        }
    };
}

pub(crate) use all_nodes;
