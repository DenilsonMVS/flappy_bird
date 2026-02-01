use std::{collections::HashMap, env, fs, path::Path};

use heck::AsPascalCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
	Attribute, Data, DeriveInput, Fields, Ident, ItemStruct, LitInt, LitStr, Token, parse::{Parse, ParseStream}, parse_macro_input
};


#[proc_macro_derive(GlVertex, attributes(vertex, normalized))]
pub fn static_vertex_layout_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let divisor = get_struct_divisor(&input.attrs);

    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(f) => f.named,
            _ => panic!("Apenas structs com campos nomeados."),
        },
        _ => panic!("Apenas structs são suportadas."),
    };

    let field_definitions = fields.iter().map(|f| {
		let field_name = &f.ident;
		let ty = &f.ty;
		let is_normalized = has_normalized_attr(&f.attrs);
		
		quote! {
			FieldType::new::<#ty>(
				#is_normalized,
				core::mem::offset_of!(#name, #field_name) as u32
			)
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

            fn get_stride() -> i32 {
                std::mem::size_of::<Self>() as i32
            }

            fn get_divisor() -> u32 {
                #divisor
            }
        }
    };

    TokenStream::from(expanded)
}

fn get_struct_divisor(attrs: &[Attribute]) -> u32 {
    for attr in attrs {
        if attr.path().is_ident("vertex") {
            let mut divisor = 0;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("divisor") {
                    let value: LitInt = meta.value()?.parse()?;
                    divisor = value.base10_parse::<u32>()?;
                }
                Ok(())
            });
            return divisor;
        }
    }
    0
}

fn has_normalized_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("normalized") {
            return true;
        }

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

	item_struct.generics.params.push(syn::parse_quote!('a));

	let mut field_info = Vec::new();

	if let Fields::Named(ref mut fields) = item_struct.fields {
		for field in fields.named.iter_mut() {
			let f_ident = field.ident.as_ref().unwrap().clone();
			let f_ty = field.ty.clone();
			
			field_info.push((f_ident, f_ty));
			
			field.ty = syn::parse_quote!(i32);
			field.vis = syn::Visibility::Public(syn::parse_quote!(pub));
		}

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

	let field_initializers = field_info.iter().map(|(f, _)| {
		let f_str = f.to_string();
		quote! {
			let #f = unsafe {
				gl::GetUniformLocation(program.get_id(), concat!(#f_str, "\0").as_ptr() as *const i8)
			};
		}
	});

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
			pub fn init(renderer: &'a Renderer) -> anyhow::Result<Self> {
				let program = Program::new(renderer, &[
					#(#shader_inits),*
				])?;

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


#[proc_macro_attribute]
pub fn atlas_bundle(attr: TokenStream, item: TokenStream) -> TokenStream {
    let path_lit = parse_macro_input!(attr as LitStr);
    let path_str = path_lit.value();
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;
    let vis = &input.vis;

    let root = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let full_path = Path::new(&root).join(&path_str);
    let content = fs::read_to_string(&full_path).unwrap();
    let raw_json: HashMap<String, serde_json::Value> = serde_json::from_str(&content).unwrap();

    let mut keys: Vec<_> = raw_json.keys().cloned().collect();
    keys.sort();

    let enum_name = format_ident!("{}Frame", struct_name);
    let fields_struct_name = format_ident!("{}Fields", struct_name);
    
    let field_idents: Vec<_> = keys.iter().map(|k| format_ident!("{}", k)).collect();
    let variant_idents: Vec<_> = keys.iter().map(|k| format_ident!("{}", AsPascalCase(k).to_string())).collect();

    let expanded = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum #enum_name {
            #(#variant_idents),*
        }

        #[derive(serde::Deserialize)]
        #vis struct #fields_struct_name {
            #(pub #field_idents: FrameInfo),*
        }

        #vis struct #struct_name {
            pub dimensions: nalgebra_glm::U32Vec2,
            pub frames: #fields_struct_name,
        }

        impl TypedAtlas for #struct_name {
            type Frame = #enum_name;

            fn new(bytes: &[u8]) -> anyhow::Result<Self> {
                let f: #fields_struct_name = serde_json::from_slice(bytes)?;
                
                let mut max_w = 0;
                let mut max_h = 0;
                
                #(
                    max_w = max_w.max(f.#field_idents.x + f.#field_idents.width);
                    max_h = max_h.max(f.#field_idents.y + f.#field_idents.height);
                )*

                Ok(Self {
                    dimensions: nalgebra_glm::U32Vec2::new(max_w, max_h),
                    frames: f,
                })
            }

            fn get_info(&self, frame: Self::Frame) -> (UvInfo, nalgebra_glm::Vec2) {
                let f = match frame {
                    #(#enum_name::#variant_idents => &self.frames.#field_idents),*
                };
                
                (f.to_uv(&self.dimensions), nalgebra_glm::vec2(f.width as f32, f.height as f32))
            }

            fn dimensions(&self) -> nalgebra_glm::U32Vec2 {
                self.dimensions
            }
        }
    };

    TokenStream::from(expanded)
}
