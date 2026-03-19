use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Field, ItemStruct, LitStr, Token, Type};

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
}

impl Parse for FFArgArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut name: Option<String> = None;
		let mut default: Option<Expr> = None;
		let mut separator: Option<String> = None;
		let mut extra_flags: Vec<String> = Vec::new();

		while !input.is_empty() {
			let key: syn::Ident = input.parse()?;
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

			if input.peek(Token![,]) {
				input.parse::<Token![,]>()?;
			}
		}

		Ok(FFArgArgs { name, default, separator, extra_flags })
	}
}

/// Adds `.to_string()` inside of the macro if `expr` is a string literal.
fn add_to_string_if_needed(expr: Expr) -> Expr {
	if matches!(&expr, Expr::Lit(lit) if matches!(&lit.lit, syn::Lit::Str(_))) {
		syn::parse2(quote! { #expr.to_string() }).unwrap()
	} else {
		expr
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

/// Returns `true` if the type is a Vec of some kind.
fn is_vec_type(ty: &Type) -> bool {
	if let Type::Path(type_path) = ty
		&& let Some(segment) = type_path.path.segments.first()
	{
		return segment.ident == "Vec";
	}
	false
}

/// Returns `true` if the type is known to implement Display.
fn is_display_type(ty: &Type) -> bool {
	if let Type::Path(type_path) = ty
		&& let Some(segment) = type_path.path.segments.first()
	{
		let name = segment.ident.to_string();
		return DISPLAY_TYPES.contains(&name.as_str());
	}
	false
}

/// Returns the display key for a field: the `name` from `#[ffarg(name = "...")]`,
/// or the field identifier with any `r#` prefix stripped.
fn field_name(ident: &syn::Ident, ffarg: &FFArgArgs) -> String {
	if let Some(ref name) = ffarg.name {
		return name.clone();
	}
	let s = ident.to_string();
	s.strip_prefix("r#").unwrap_or(&s).to_string()
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

fn filter_macro(args: TokenStream2, input: TokenStream2) -> syn::Result<TokenStream2> {
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

	for field in fields.iter() {
		let ident = field.ident.as_ref().unwrap();
		let ty = &field.ty;

		let ffarg = get_ffarg_args(field)?;

		// The field name. Either pulled from the ffarg `name` argument or the field name, verbatim.
		let name = field_name(ident, &ffarg);

		// The field value.
		let value_expr = if is_display_type(ty) {
			quote! { self.#ident.to_string() }
		} else if is_vec_type(ty) {
			let sep = ffarg.separator.clone().unwrap_or(":".to_string());
			let flags = &ffarg.extra_flags;
			if flags.is_empty() {
				quote! { self.#ident.join(#sep) }
			} else {
				quote! {
					{
						let mut v = self.#ident.clone();
						v.extend([#(#flags.to_string()),*]);
						v.dedup();
						v.join(#sep)
					}
				}
			}
		} else {
			// TODO: this assumes everything else implements Display. return an error here?
			quote! { self.#ident.to_string() }
		};

		display_entries.push(quote! {
			format!("{}={}", #name, #value_expr)
		});

		// The default value for this field. Uses the ffarg `default` argument when specified, else Default::default().
		// TODO: this assumes everything implements Default. proooobably reasonable but check back after adding more filters
		let default_val = match ffarg.default {
			Some(expr) => quote! { #expr },
			None => quote! { Default::default() },
		};
		default_entries.push(quote! { #ident: #default_val });
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
			// use crate::filters::FFArg;
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
				fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
					let output: Vec<String> = vec![
						#(#display_entries,)*
					];
					write!(f, "{}={}", <Self as FFmpegFilter>::NAME, output.join(":"))
				}
			}
		};
	})
}

#[proc_macro_attribute]
pub fn filter(args: TokenStream, input: TokenStream) -> TokenStream {
	match filter_macro(args.into(), input.into()) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.into_compile_error().into(),
	}
}
