pub struct Machine {
	pub name: String,
	pub id: String,
	pub description: String,
	pub category: String,
	pub urn: String,
	pub usage: Usage
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Usage {
	Free,
	Occupied,
	Yours
}