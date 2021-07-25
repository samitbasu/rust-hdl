#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct Manufacturer {
    pub manufacturer: String,
    pub part_number: String,
}
