use tokio::sync::RwLock;

pub mod resource;
pub mod supporter;

pub type Resource  = Entity<resource ::State, resource ::Configuration>;
pub type Supporter = Entity<supporter::State, supporter::Configuration>;

pub struct Entity<State, Configuration> {
	pub state: RwLock<State>,
	pub configuration: Configuration
}