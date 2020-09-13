use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use std::mem::size_of;

type CommaList<T> = syn::punctuated::Punctuated<T, syn::Token![,]>;

trait Vertex {
    fn desc<'desc>() -> wgpu::VertexBufferDescriptor<'desc>;
}

struct AttrData {
    size_in_bytes: usize,
    format: String,
}

impl AttrData {
    fn new(size_in_bytes: usize, format: &str) -> Self {
        Self { size_in_bytes, format: format.to_string() }
    }
}


#[proc_macro_derive(Vertex)]
pub fn vertex_macro_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    let name = &ast.ident;

    let fields = match &ast.data {
        syn::Data::Struct(st) => {
            &st.fields
        }
        syn::Data::Enum(_) => {
            panic!("Enums are not supported for Vertex derives at this time")
        }
        syn::Data::Union(_) => {
            panic!("Unions are not supported for Vertex derives at this time")
        }
    };

    let fields = match &fields {
        syn::Fields::Named(f) => &f.named,
        syn::Fields::Unnamed(f) => &f.unnamed,
        syn::Fields::Unit => panic!("Unit structs are not supported"),
    };

    let mut shader_location = 0u32;
    let mut offset = 0u64;
    let attrs = fields.iter().map(|f| {
        let ty = &f.ty;
        let AttrData { size_in_bytes, format } = get_attr_data(&ty);
        let format = syn::Ident::new(&format, ty.span());
        let attr = quote! {
            ::wgpu::VertexAttributeDescriptor {
                offset: #offset,
                format: ::wgpu::VertexFormat::#format,
                shader_location: #shader_location,
            }
        };

        shader_location += 1;
        offset += size_in_bytes as u64;

        attr
    }).collect::<CommaList<_>>();

    let vertex_impl = quote! {
        impl Vertex for #name {
            fn desc<'desc>() -> ::wgpu::VertexBufferDescriptor<'desc> {
                wgpu::VertexBufferDescriptor {
                    stride: ::std::mem::size_of::<Self>() as ::wgpu::BufferAddress,
                    step_mode: ::wgpu::InputStepMode::Vertex,
                    attributes: &[
                        #attrs
                    ],
                }
            }
        }
    };
    vertex_impl.into()
}

fn get_attr_data(ty: &syn::Type) -> AttrData {
    match ty {
        syn::Type::Path(p) => {
            let segments = &p.path.segments;
            let last = match segments.last() {
                Some(seg) => seg,
                None => panic!("Invalid path! {}", quote!{#p}),
            };
            let ident = &last.ident;
            match ident.to_string().as_ref() {
                "f32" => AttrData::new(size_of::<f32>(), "Float"),
                "u32" => AttrData::new(size_of::<u32>(), "Uint"),
                "i32" => AttrData::new(size_of::<i32>(), "Int"),
                _ => panic!("Unsupported type! {}", quote!{#ty}),
            }
        }
        syn::Type::Array(a) => {
            let AttrData { size_in_bytes, format } = get_attr_data(&a.elem);
            let len: usize = match &a.len {
                syn::Expr::Lit(syn::ExprLit { lit, .. }) => match lit {
                    syn::Lit::Int(li) => {
                        li.base10_parse().unwrap()
                    }
                    _ => panic!("Only integer literals are supported!"),
                },
                _ => panic!("Only integer literals are supported!"),
            };

            assert!(len > 0 && len <= 4);

            AttrData { size_in_bytes: size_in_bytes * len, format: format!("{}{}", format, len) }
        }
        _ => panic!("Unsupported type! {}", quote!{#ty}),
    }
}