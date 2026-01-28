use proc_macro::TokenStream;
use quote::quote;
use syn::{
	Attribute, Data, DeriveInput, Fields, Ident, ItemStruct, LitStr, Token, 
	parse::{Parse, ParseStream}, parse_macro_input
};

// --- [GlVertex Derive] ---

#[proc_macro_derive(GlVertex, attributes(vertex))]
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

	let field_definitions = fields.iter().map(|f| {
		let ty = &f.ty;
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

fn has_normalized_attr(attrs: &[Attribute]) -> bool {
	attrs.iter().any(|attr| {
		if attr.path().is_ident("vertex") {
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

// --- [Program Interface Attribute] ---

struct ShaderArgs {
	args: Vec<(String, String)>,
}

impl Parse for ShaderArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut args = Vec::new();
		while !input.is_empty() {
			let key: Ident = input.parse()?;
			input.parse::<Token![=]>()?;
			let path: LitStr = input.parse()?;
			args.push((key.to_string(), path.value()));
			if !input.is_empty() { input.parse::<Token![,]>()?; }
		}
		Ok(ShaderArgs { args })
	}
}

#[proc_macro_attribute]
pub fn program_interface(args: TokenStream, input: TokenStream) -> TokenStream {
	let shader_args = parse_macro_input!(args as ShaderArgs);
	let mut item_struct = parse_macro_input!(input as ItemStruct);
	let name = &item_struct.ident;

	// Adiciona o lifetime para o Renderer
	item_struct.generics.params.push(syn::parse_quote!('a));

	let mut field_info = Vec::new(); // Armazena (identificador, tipo_original)

	if let Fields::Named(ref mut fields) = item_struct.fields {
		for field in fields.named.iter_mut() {
			let f_ident = field.ident.as_ref().unwrap().clone();
			let f_ty = field.ty.clone();
			
			// Guardamos os tipos originais para os setters
			field_info.push((f_ident, f_ty));
			
			// Transformamos o campo da struct em um i32 (location)
			field.ty = syn::parse_quote!(i32);
			field.vis = syn::Visibility::Public(syn::parse_quote!(pub));
		}

		// Adiciona o programa interno (posse do ID OpenGL)
		fields.named.push(syn::parse_quote!(pub program: Program<'a>));
	}

	let shader_inits = shader_args.args.iter().map(|(kind, path)| {
		let variant = match kind.as_str() {
			"vert" => quote!(ShaderType::Vertex),
			"frag" => quote!(ShaderType::Fragment),
			"geom" => quote!(ShaderType::Geometry),
			"comp" => quote!(ShaderType::Compute),
			"tesc" => quote!(ShaderType::TesselationControl),
			"tese" => quote!(ShaderType::TesselationEvaluation),
			_ => panic!("Tipo de shader desconhecido: {}", kind),
		};
		quote! {
			(
				unsafe { std::ffi::CStr::from_ptr(concat!(include_str!(#path), "\0").as_ptr() as *const i8) },
				#variant
			)
		}
	});

	// Inicializa as localizações dos uniforms
	let field_initializers = field_info.iter().map(|(f, _)| {
		let f_str = f.to_string();
		quote! {
			let #f = unsafe {
				gl::GetUniformLocation(program.get_id(), concat!(#f_str, "\0").as_ptr() as *const i8)
			};
		}
	});

	// Gera os setters tipados usando DSA (glProgramUniform)
	let setters = field_info.iter().map(|(f, ty)| {
		let setter_name = Ident::new(&format!("set_{}", f), f.span());
		quote! {
			pub fn #setter_name(&self, value: &#ty) {
				UniformValue::set_program_uniform(value, self.program.get_id(), self.#f);
			}
		}
	});

	let field_names: Vec<_> = field_info.iter().map(|(f, _)| f).collect();

	let expanded = quote! {
		#item_struct

		impl<'a> #name<'a> {
			pub fn init(renderer: &'a Renderer) -> Result<Self, String> {
				let program = Program::new(renderer, &[
					#(#shader_inits),*
				]).ok_or_else(|| "Erro no link/compilação do programa OpenGL".to_string())?;

				#(#field_initializers)*

				Ok(Self {
					#(#field_names,)*
					program,
				})
			}

			#(#setters)*

			pub fn bind(&self) {
				self.program.bind();
			}
		}

		impl<'a> std::ops::Deref for #name<'a> {
			type Target = Program<'a>;
			fn deref(&self) -> &Self::Target {
				&self.program
			}
		}
	};

	TokenStream::from(expanded)
}
