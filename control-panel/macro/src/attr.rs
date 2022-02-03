use syn::Attribute;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum CallHandlerArgumentAttribute {
    Engine,
    Responder,
}

impl CallHandlerArgumentAttribute {
    pub fn clean(attrs: &mut Vec<Attribute>) {
        attrs.retain(|a| Self::from_attr(a).is_none())
    }

    pub fn from_attrs(attrs: &[Attribute]) -> Vec<Self> {
        attrs.iter().filter_map(Self::from_attr).collect()
    }

    pub fn from_attr(attr: &Attribute) -> Option<Self> {
        attr.path
            .get_ident()
            .and_then(|i| match i.to_string().as_str() {
                "engine" => Some(Self::Engine),
                "responder" => Some(Self::Responder),
                _ => None,
            })
    }
}
