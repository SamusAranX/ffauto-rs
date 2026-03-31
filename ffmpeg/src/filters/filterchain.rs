use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Write;
use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct FilterChain {
	pub filters: Vec<Box<dyn Display + Send + Sync>>,
	pub inputs: Vec<String>,
	pub outputs: Vec<String>,
}

impl FilterChain {
	/// Shorter alias for `FilterList::default()`
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	#[must_use]
	pub fn with_inputs(inputs: Vec<String>) -> Self {
		Self { inputs, ..Default::default() }
	}

	#[must_use]
	pub fn with_outputs(outputs: Vec<String>) -> Self {
		Self { outputs, ..Default::default() }
	}

	#[must_use]
	pub fn with_inputs_and_outputs(inputs: Vec<String>, outputs: Vec<String>) -> Self {
		Self { inputs, outputs, ..Default::default() }
	}

	pub fn push<T: Display + Send + Sync + 'static>(&mut self, value: T) {
		self.filters.push(Box::new(value));
	}

	pub fn extend(&mut self, other: FilterChain) {
		self.filters.extend(other.filters);
	}
}

impl Deref for FilterChain {
	type Target = Vec<Box<dyn Display + Send + Sync>>;

	fn deref(&self) -> &Self::Target {
		&self.filters
	}
}

impl DerefMut for FilterChain {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.filters
	}
}

#[allow(clippy::print_in_format_impl)]
impl Display for FilterChain {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let filters = self
			.filters
			.iter()
			.map(ToString::to_string)
			.collect::<Vec<_>>()
			.join(",");

		#[cfg(debug_assertions)]
		eprintln!("FilterChain I/O: {:?} → {:?}", self.inputs, self.outputs);

		if !self.inputs.is_empty() {
			let inputs = self.inputs.iter().fold(String::new(), |mut acc, input| {
				let _ = write!(acc, "[{input}]");
				acc
			});
			write!(f, "{inputs}")?;
		}

		write!(f, "{filters}")?;

		if !self.outputs.is_empty() {
			let outputs = self.outputs.iter().fold(String::new(), |mut acc, output| {
				let _ = write!(acc, "[{output}]");
				acc
			});
			write!(f, "{outputs}")?;
		}

		Ok(())
	}
}
