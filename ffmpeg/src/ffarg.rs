type ArgFlags = Vec<String>;

trait FFArg {
	fn to_arg(&self) -> String;
}

macro_rules! ffarg_impl {
	($($type:ty), + $(,)?) => {
		$(
		impl FFArg for $type {
			fn to_arg(&self) -> String {
				self.to_string()
			}
		}
		)+
	}
}

#[rustfmt::skip]
ffarg_impl!(
	i8, i16, i32, i64, i128, isize,
	u8, u16, u32, u64, u128, usize,
	f32, f64,
);

impl FFArg for bool {
	fn to_arg(&self) -> String {
		match self {
			true => "1",
			false => "0",
		}
		.to_string()
	}
}

impl FFArg for ArgFlags {
	fn to_arg(&self) -> String {
		self.join("+")
	}
}
