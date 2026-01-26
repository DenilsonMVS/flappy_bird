use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Attribute};

#[proc_macro_derive(GlVertex, attributes(vertex))] // 👈 Precisamos registrar o nome do atributo
pub fn static_vertex_layout_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident;

	let fields = match input.data {
		Data::Struct(s) => match s.fields {
			Fields::Named(f) => f.named,
			_ => panic!("Apenas structs com campos nomeados."),
		},
		_ => panic!("Apenas structs são suportadas."),
	};

	// 1. Mapeamos cada campo para gerar a chamada FieldType::new
	let field_definitions = fields.iter().map(|f| {
		let ty = &f.ty;
		// 2. Verificamos se o atributo #[vertex(normalized)] existe neste campo
		let is_normalized = has_normalized_attr(&f.attrs);
		
		quote! {
			FieldType::new::<#ty>(#is_normalized)
		}
	});

	let expanded = quote! {
		impl StaticVertexLayout for #name {
			fn get_fields() -> &'static [FieldType] {
				const FIELDS: &[FieldType] = &[
					#(#field_definitions),*
				];
				FIELDS
			}
		}
	};

	TokenStream::from(expanded)
}

// Função auxiliar para escanear os atributos do campo
fn has_normalized_attr(attrs: &[Attribute]) -> bool {
	attrs.iter().any(|attr| {
		if attr.path().is_ident("vertex") {
			// Verifica se dentro de vertex() existe "normalized"
			let mut is_norm = false;
			let _ = attr.parse_nested_meta(|meta| {
				if meta.path.is_ident("normalized") {
					is_norm = true;
				}
				Ok(())
			});
			return is_norm;
		}
		false
	})
}
