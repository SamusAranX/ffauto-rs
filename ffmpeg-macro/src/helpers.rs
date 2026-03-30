use crate::{DISPLAY_TYPES, FFArgArgs};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Type};

/// Returns `true` if the type is a HashMap of some kind.
pub(crate) fn is_hashmap_type(ty: &Type) -> bool {
	if let Type::Path(type_path) = ty
		&& let Some(segment) = type_path.path.segments.last()
	{
		return segment.ident == "HashMap";
	}
	false
}

/// Adds `.to_string()` inside of the macro if `expr` is a string literal.
pub(crate) fn add_to_string_if_needed(expr: Expr) -> Expr {
	if matches!(&expr, Expr::Lit(lit) if matches!(&lit.lit, syn::Lit::Str(_))) {
		syn::parse2(quote! { #expr.to_string() }).unwrap()
	} else {
		expr
	}
}

/// Returns `true` if the type is known to implement Display.
pub(crate) fn is_display_type(ty: &Type) -> bool {
	if let Type::Path(type_path) = ty
		&& let Some(segment) = type_path.path.segments.first()
	{
		let name = segment.ident.to_string();
		return DISPLAY_TYPES.contains(&name.as_str());
	}
	false
}

/// Returns `true` if the type is bool.
pub(crate) fn is_bool_type(ty: &Type) -> bool {
	if let Type::Path(type_path) = ty
		&& let Some(segment) = type_path.path.segments.first()
	{
		return segment.ident == "bool";
	}
	false
}

/// Returns `true` if the type is a Vec of some kind.
pub(crate) fn is_vec_type(ty: &Type) -> bool {
	if let Type::Path(type_path) = ty
		&& let Some(segment) = type_path.path.segments.first()
	{
		return segment.ident == "Vec";
	}
	false
}

/// Returns `true` if the type is an Option of some kind.
pub(crate) fn is_option_type(ty: &Type) -> bool {
	if let Type::Path(type_path) = ty
		&& let Some(segment) = type_path.path.segments.first()
	{
		return segment.ident == "Option";
	}
	false
}

/// Extracts the inner type `T` from `Option<T>`.
pub(crate) fn extract_option_inner_type(ty: &Type) -> Option<&Type> {
	if let Type::Path(type_path) = ty
		&& let Some(segment) = type_path.path.segments.first()
		&& segment.ident == "Option"
		&& let syn::PathArguments::AngleBracketed(args) = &segment.arguments
		&& let Some(syn::GenericArgument::Type(inner)) = args.args.first()
	{
		return Some(inner);
	}
	None
}

/// Returns the display key for a field: the `name` from `#[ffarg(name = "...")]`,
/// or the field identifier with any `r#` prefix stripped.
pub(crate) fn field_name(ident: &syn::Ident, ffarg: &FFArgArgs) -> String {
	if let Some(ref name) = ffarg.name {
		return name.clone();
	}
	let s = ident.to_string();
	s.strip_prefix("r#").unwrap_or(&s).to_string()
}

/// Generates a `TokenStream` for a block that builds the final `Vec<String>` from the given
/// vec access expression (applying `default_from` and `extra_flags`), then returns
/// `Option<String>`: `Some(formatted)` if non-empty, `None` if empty.
///
/// The `format_fn` closure receives the join expression and should return the final
/// format expression (e.g. with or without the field name).
pub(crate) fn vec_display_expr(
	vec_access: TokenStream,
	ffarg: &FFArgArgs,
	default_from_extend: Option<TokenStream>,
	format_fn: impl FnOnce(TokenStream) -> TokenStream,
) -> TokenStream {
	let sep = ffarg.separator.clone().unwrap_or(":".to_string());
	let flags = &ffarg.extra_flags;

	let extra_extends = if flags.is_empty() {
		quote! {}
	} else {
		quote! { v.extend([#(#flags.to_string()),*]); }
	};

	let needs_mut = default_from_extend.is_some() || !flags.is_empty();
	let build = if needs_mut {
		quote! {
			let mut v = #vec_access.clone();
			#default_from_extend
			#extra_extends
			v.dedup();
		}
	} else {
		quote! { let v = &#vec_access; }
	};

	let join_expr = quote! { v.join(#sep) };
	let format_expr = format_fn(join_expr);

	quote! {
		{
			#build
			if v.is_empty() {
				None
			} else {
				Some(#format_expr)
			}
		}
	}
}

/// Generates a `TokenStream` for a block that formats a HashMap field as a flat string.
/// Key-value pairs are joined with `map_kv_separator` (default `"="`), and the resulting
/// strings are joined with `map_item_separator` (default `","`). Returns `Option<String>`:
/// `Some(formatted)` if non-empty, `None` if the map is empty.
///
/// The `format_fn` closure receives the join expression and should return the final
/// format expression (e.g. with or without the field name).
pub(crate) fn hashmap_display_expr(
	map_access: TokenStream,
	ffarg: &FFArgArgs,
	format_fn: impl FnOnce(TokenStream) -> TokenStream,
) -> TokenStream {
	let kv_sep = ffarg.map_kv_separator.as_deref().unwrap_or("=");
	let item_sep = ffarg.map_item_separator.as_deref().unwrap_or(",");

	let join_expr = quote! { format!("'{}'", pairs.join(#item_sep)) };
	let format_expr = format_fn(join_expr);

	quote! {
		{
			let mut sorted: Vec<_> = #map_access.iter().collect();
			sorted.sort_by_key(|(k, _)| k.to_string());
			let pairs: Vec<String> = sorted.iter()
				.map(|(k, v)| format!("{}{}{}", k, #kv_sep, v))
				.collect();
			if pairs.is_empty() {
				None
			} else {
				Some(#format_expr)
			}
		}
	}
}
