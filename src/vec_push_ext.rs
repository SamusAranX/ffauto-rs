pub(crate) trait PushStrExt {
	fn push_str(&mut self, s: &str);
}

impl PushStrExt for Vec<String> {
	fn push_str(&mut self, s: &str) {
		self.push(s.to_owned());
	}
}