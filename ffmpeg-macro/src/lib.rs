mod helpers;

use crate::helpers::{
	add_to_string_if_needed, extract_option_inner_type, field_name, is_bool_type, is_display_type, is_option_type,
	is_vec_type, vec_display_expr,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Field, ItemStruct, LitStr, Token};

/////
// ffarg helpers
/////

/// Types that implement Display and thus can use .to_string() directly.
#[rustfmt::skip]
const DISPLAY_TYPES: &[&str] = &[
	"i8", "i16", "i32", "i64", "i128", "isize",
	"u8", "u16", "u32", "u64", "u128", "usize",
	"f32", "f64",
	"String", "&str",
];

/// Arguments for the `#[ffarg(name = "string", default = <expr>)]` field attribute.
#[derive(Default)]
struct FFArgArgs {
	name: Option<String>,
	default: Option<Expr>,
	separator: Option<String>,
	extra_flags: Vec<String>,
	omit_default: bool,
	noname: bool,
	default_from: Option<syn::Ident>,
}

impl Parse for FFArgArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut name: Option<String> = None;
		let mut default: Option<Expr> = None;
		let mut default_from: Option<syn::Ident> = None;
		let mut separator: Option<String> = None;
		let mut extra_flags: Vec<String> = Vec::new();
		let mut omit_default = false;
		let mut noname = false;

		while !input.is_empty() {
			let key: syn::Ident = input.parse()?;

			match key.to_string().as_str() {
				"omit_default" => {
					omit_default = true;
				}
				"noname" => {
					noname = true;
				}
				_ => {
					input.parse::<Token![=]>()?;

					match key.to_string().as_str() {
						"name" => {
							let lit: LitStr = input.parse()?;
							name = Some(lit.value());
						}
						"default" => {
							let expr: Expr = input.parse()?;
							// Wrap bare string literals in .to_string() so users can write
							// `#[ffarg(default = "black")]` instead of `#[ffarg(default = "black".to_string())]`
							default = Some(add_to_string_if_needed(expr));
						}
						"default_from" => {
							let ident: syn::Ident = input.parse()?;
							default_from = Some(ident);
						}
						"separator" => {
							let lit: LitStr = input.parse()?;
							separator = Some(lit.value());
						}
						"extra_flags" => {
							let content;
							syn::bracketed!(content in input);
							while !content.is_empty() {
								let lit: LitStr = content.parse()?;
								extra_flags.push(lit.value());
								if content.peek(Token![,]) {
									content.parse::<Token![,]>()?;
								}
							}
						}
						other => return Err(syn::Error::new(key.span(), format!("Unknown ffarg argument: {other}"))),
					}
				}
			}

			if input.peek(Token![,]) {
				input.parse::<Token![,]>()?;
			}
		}

		if name.is_some() && noname {
			return Err(syn::Error::new(input.span(), "name and noname are mutually exclusive"));
		}

		Ok(FFArgArgs {
			name,
			default,
			separator,
			extra_flags,
			omit_default,
			noname,
			default_from,
		})
	}
}

/// Extract `#[ffarg(...)]` arguments from a struct field, if present.
fn get_ffarg_args(field: &Field) -> syn::Result<FFArgArgs> {
	for attr in &field.attrs {
		if attr.path().is_ident("ffarg") {
			return attr.parse_args::<FFArgArgs>();
		}
	}
	Ok(FFArgArgs::default())
}

/////
// filter macro
/////

/// Arguments for the `#[filter(name = "string")]` struct attribute.
struct FilterArgs {
	name: String,
}

impl Parse for FilterArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut name = None;

		while !input.is_empty() {
			let key: syn::Ident = input.parse()?;
			input.parse::<Token![=]>()?;

			match key.to_string().as_str() {
				"name" => {
					let lit: LitStr = input.parse()?;
					name = Some(lit.value());
				}
				other => return Err(syn::Error::new(key.span(), format!("Unknown filter argument: {other}"))),
			}

			if input.peek(Token![,]) {
				input.parse::<Token![,]>()?;
			}
		}

		let name = name.ok_or_else(|| input.error("Missing required argument: name"))?;
		Ok(FilterArgs { name })
	}
}

fn filter_macro(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
	let filter_args: FilterArgs = syn::parse2(args)?;

	let mut item: ItemStruct = syn::parse2(input)?;
	let struct_ident = &item.ident;
	let filter_name = &filter_args.name;

	let fields = match &item.fields {
		syn::Fields::Named(fields) => &fields.named,
		_ => {
			return Err(syn::Error::new_spanned(
				&item,
				"#[filter] only works on structs with named fields",
			));
		}
	};

	let mut display_entries = Vec::new();
	let mut default_entries = Vec::new();

	// Collect the set of field names referenced by `default_from`, these are skipped in Display.
	let mut referenced_fields: std::collections::HashSet<String> = std::collections::HashSet::new();
	for field in fields.iter() {
		let ffarg = get_ffarg_args(field)?;
		if let Some(ref source) = ffarg.default_from {
			referenced_fields.insert(source.to_string());
		}
	}

	for field in fields.iter() {
		let ident = field.ident.as_ref().unwrap();
		let ty = &field.ty;

		let ffarg = get_ffarg_args(field)?;
		let is_option = is_option_type(ty);

		// Validate that incompatible ffarg parameters are not used on Option fields.
		if is_option {
			if ffarg.omit_default {
				return Err(syn::Error::new_spanned(
					field,
					"Optional types are already omitted when None",
				));
			}
			let inner_is_vec = extract_option_inner_type(ty).is_some_and(is_vec_type);
			if ffarg.default.is_some() || (ffarg.default_from.is_some() && !inner_is_vec) {
				return Err(syn::Error::new_spanned(field, "Optional types always default to None"));
			}
		}

		// The default value for this field. Uses the ffarg `default` argument when specified, else Default::default().
		// TODO: this assumes everything implements Default. proooobably reasonable but check back after adding more filters
		let default_val = if is_option {
			quote! { None }
		} else {
			match &ffarg.default {
				Some(expr) => quote! { #expr },
				None => quote! { Default::default() },
			}
		};
		default_entries.push(quote! { #ident: #default_val });

		// Fields referenced by another field's `default_from` are skipped in Display,
		// their value is added to the referencing field's value instead.
		let skip_display = referenced_fields.contains(&ident.to_string());
		if skip_display {
			continue;
		}

		// The field name. Either pulled from the ffarg `name` argument or the field name, verbatim.
		let name = field_name(ident, &ffarg);

		// Option<T> fields: only included in Display when Some.
		if is_option {
			let inner_ty = extract_option_inner_type(ty).unwrap();
			if is_vec_type(inner_ty) {
				let default_from_extend = ffarg.default_from.as_ref().map(|source| {
					quote! { v.push(self.#source.to_string()); }
				});
				let noname = ffarg.noname;
				let vec_expr = vec_display_expr(quote! { val }, &ffarg, default_from_extend, |join| {
					if noname {
						quote! { format!("{}", #join) }
					} else {
						quote! { format!("{}={}", #name, #join) }
					}
				});
				display_entries.push(quote! {
					if let Some(val) = &self.#ident {
						#vec_expr
					} else {
						None
					}
				});
			} else {
				let value_expr = if is_display_type(inner_ty) {
					quote! { val.to_string() }
				} else if is_bool_type(inner_ty) {
					quote! { u8::from(*val).to_string() }
				} else {
					quote! { val.to_string() }
				};
				let format_expr = if ffarg.noname {
					quote! { format!("{}", #value_expr) }
				} else {
					quote! { format!("{}={}", #name, #value_expr) }
				};
				display_entries.push(quote! {
					if let Some(val) = &self.#ident {
						Some(#format_expr)
					} else {
						None
					}
				});
			}
			continue;
		}

		// The field value.
		let value_expr = if is_display_type(ty) {
			quote! { self.#ident.to_string() }
		} else if is_bool_type(ty) {
			quote! { u8::from(self.#ident).to_string() }
		} else if is_vec_type(ty) {
			// If this field has `default_from`, inject the source field's value (when non-default).
			let default_from_extend = ffarg.default_from.as_ref().map(|source| {
				quote! { v.push(self.#source.to_string()); }
			});

			let noname = ffarg.noname;
			let vec_expr = vec_display_expr(quote! { self.#ident }, &ffarg, default_from_extend, |join| {
				if noname {
					quote! { format!("{}", #join) }
				} else {
					quote! { format!("{}={}", #name, #join) }
				}
			});
			display_entries.push(vec_expr);
			continue;
		} else {
			// enums fall into this branch (along with everything else) but
			// the enums used for filters generally implement Display thanks to strum.
			// if we get here with a field of a type that doesn't implement Display,
			// this'll result in a compile-time error.

			quote! { self.#ident.to_string() }
		};

		let format_expr = if ffarg.noname {
			quote! { format!("{}", #value_expr) }
		} else {
			quote! { format!("{}={}", #name, #value_expr) }
		};

		if ffarg.omit_default {
			let default_val = match &ffarg.default {
				Some(expr) => quote! { #expr },
				None => quote! { <#ty as Default>::default() },
			};
			display_entries.push(quote! {
				if self.#ident != #default_val {
					Some(#format_expr)
				} else {
					None
				}
			});
		} else {
			display_entries.push(quote! {
				Some(#format_expr)
			});
		}
	}

	// Strip #[ffarg] attributes from every field before outputting it
	if let syn::Fields::Named(ref mut fields) = item.fields {
		for field in fields.named.iter_mut() {
			field.attrs.retain(|attr| !attr.path().is_ident("ffarg"));
		}
	}

	Ok(quote! {
		#[derive(Debug, Clone)]
		#item

		const _: () = {
			use crate::filters::FFmpegFilter;

			impl Default for #struct_ident {
				fn default() -> Self {
					Self {
						#(#default_entries,)*
					}
				}
			}

			impl FFmpegFilter for #struct_ident {
				const NAME: &str = #filter_name;
			}

			impl ::std::fmt::Display for #struct_ident {
				#[allow(clippy::float_cmp, clippy::cmp_owned)]
				fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
					let output: Vec<String> = [
						#(#display_entries,)*
					].into_iter().flatten().filter(|s| !s.is_empty()).collect();

					if output.is_empty() {
						write!(f, "{}", <Self as FFmpegFilter>::NAME)
					} else {
						write!(f, "{}={}", <Self as FFmpegFilter>::NAME, output.join(":"))
					}
				}
			}
		};
	})
}

#[proc_macro_attribute]
pub fn filter(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	match filter_macro(args.into(), input.into()) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.into_compile_error().into(),
	}
}
