pub type ErrorTomatoId = u8;
#[derive(Debug)]
pub enum ErrorTomatoItem<'a> {
    Leaf(String),
    Node(&'a dyn ErrorTomato),
}

pub trait ErrorTomato: core::fmt::Debug {
    fn inner<'a>(&'a self) -> ErrorTomatoItem<'a>;
    fn name(&self) -> String {
        let dbg_repr = format!("{:?}", self);
        dbg_repr
            .split_once('(')
            .map(|(name, _)| name)
            .unwrap_or("-")
            .to_string()
    }
    fn id(&self) -> ErrorTomatoId;
}

macro_rules! error_all_nodes {
    ($name:ident; $variant:ident; $($var:ident),*) => {
        impl crate::error_tomato::ErrorTomato for $name {
            fn inner<'a>(&'a self) -> crate::error_tomato::ErrorTomatoItem<'a> {
                match self {
                    $(
                    $name::$var(inner) => crate::error_tomato::ErrorTomatoItem::Node(inner as &dyn crate::error_tomato::ErrorTomato)
                    ),*
                }
            }

            fn id(&self) -> crate::error_tomato::ErrorTomatoId {
                $variant::from(self) as crate::error_tomato::ErrorTomatoId
            }
        }
    };
}

pub(crate) use error_all_nodes;
