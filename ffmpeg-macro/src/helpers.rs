use crate::{DISPLAY_TYPES, FFArgArgs};
use quote::quote;
use syn::{Expr, Type};

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

/// Returns the display key for a field: the `name` from `#[ffarg(name = "...")]`,
/// or the field identifier with any `r#` prefix stripped.
pub(crate) fn field_name(ident: &syn::Ident, ffarg: &FFArgArgs) -> String {
	if let Some(ref name) = ffarg.name {
		return name.clone();
	}
	let s = ident.to_string();
	s.strip_prefix("r#").unwrap_or(&s).to_string()
}
