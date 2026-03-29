use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Deref;

#[derive(Default)]
pub struct FilterChain {
	pub filters: Vec<Box<dyn Display + Send + Sync>>,
	pub inputs: Vec<String>,
	pub outputs: Vec<String>,
}

impl Deref for FilterChain {
	type Target = Vec<Box<dyn Display + Send + Sync>>;

	fn deref(&self) -> &Self::Target {
		&self.filters
	}
}

impl Display for FilterChain {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let filters = self
			.filters
			.iter()
			.map(ToString::to_string)
			.collect::<Vec<_>>()
			.join(",");

		let inputs = self
			.inputs
			.iter()
			.fold(String::new(), |i, _| format!("[{i}]"));
		let outputs = self
			.outputs
			.iter()
			.fold(String::new(), |o, _| format!("[{o}]"));

		if !inputs.is_empty() {
			write!(f, "{inputs}")?;
		}

		write!(f, "{filters}")?;

		if !outputs.is_empty() {
			write!(f, "{outputs}")?;
		}

		Ok(())
	}
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
