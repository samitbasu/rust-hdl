use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Suppliers {
    DigiKey,
    Mouser,
}

#[derive(Clone, Debug)]
pub struct Supplier {
    pub supplier: Suppliers,
    pub part_number: String,
    pub url: url::Url,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manufacturer {
    pub name: String,
    pub part_number: String,
}
