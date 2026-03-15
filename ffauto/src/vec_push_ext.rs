pub(crate) trait PushStrExt {
	fn add<S: Into<String>>(&mut self, s: S);
	fn add_two<S: Into<String>, T: Into<String>>(&mut self, s1: S, s2: T);
}

impl PushStrExt for Vec<String> {
	fn add<S: Into<String>>(&mut self, s: S) {
		self.push(s.into());
	}

	/// allows for easier assembly of command argument lists
	fn add_two<S: Into<String>, T: Into<String>>(&mut self, s1: S, s2: T) {
		self.push(s1.into());
		self.push(s2.into());
	}
}
