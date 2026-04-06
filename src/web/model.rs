pub struct Machine {
	pub name: String,
	pub usage: Usage,
	
	pub urn: String,
	pub category: String,
	pub description: String,
	pub id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Usage {
	Free,
	Occupied,
	Yours
}