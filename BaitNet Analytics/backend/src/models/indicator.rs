use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndicatorKind {
    Ipv4,
    Ipv6,
    Domain,
    Url,
    Email,
    Phone,
    Bitcoin,
    Ethereum,
}

impl Display for IndicatorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Ipv4 => "ipv4",
            Self::Ipv6 => "ipv6",
            Self::Domain => "domain",
            Self::Url => "url",
            Self::Email => "email",
            Self::Phone => "phone",
            Self::Bitcoin => "bitcoin",
            Self::Ethereum => "ethereum",
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Indicator {
    pub id: Option<Uuid>,
    pub kind: IndicatorKind,
    pub value: String,
}

impl Indicator {
    pub fn new(kind: IndicatorKind, value: String) -> Self {
        Self { id: None, kind, value }
    }
}
