pub struct Machine {
	pub name: String,
	pub usage: Usage
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Usage {
	Free,
	Occupied,
	Yours
}