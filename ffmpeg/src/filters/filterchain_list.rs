use crate::filters::FilterChain;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::vec::IntoIter;

#[derive(Default)]
pub struct FilterChainList(Vec<FilterChain>);

impl FilterChainList {
	/// Shorter alias for `FilterChainList::default()`
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	pub fn extend(&mut self, other: impl IntoIterator<Item = FilterChain>) {
		self.0.extend(other);
	}
}

// This is to ensure FilterChainList::extend() still works with arguments of type `[FilterChain; _]` and so on.
impl IntoIterator for FilterChainList {
	type Item = FilterChain;
	type IntoIter = IntoIter<FilterChain>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl Deref for FilterChainList {
	type Target = Vec<FilterChain>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for FilterChainList {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Display for FilterChainList {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let chain_string = self
			.iter()
			.map(ToString::to_string)
			.collect::<Vec<_>>()
			.join(";");

		write!(f, "{chain_string}")?;

		Ok(())
	}
}
