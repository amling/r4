extern crate proc_macro2;
extern crate proc_macro;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::Attribute;
use syn::Data;
use syn::DeriveInput;
use syn::Fields;
use syn::Ident;
use syn::Index;
use syn::Lit;
use syn::LitStr;
use syn::Meta;
use syn::export::Span;

#[proc_macro_derive(Validates, attributes(ValidatesName))]
pub fn derive_validates(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let ctor_args;
    let struct_args;
    let clone_args;
    match &ast.data {
        Data::Struct(d) => match &d.fields {
            Fields::Named(d) => {
                let ctor_fields: Vec<_> = d.named.iter().map(|f| {
                    let name = f.ident.as_ref().unwrap();
                    let mangle = compute_mangle_expr(&f.attrs, name.to_string());
                    return quote! {
                        #name: ::validates::Validates::validate(self.#name) #mangle ?,
                    };
                }).collect();
                ctor_args = quote! { { #( #ctor_fields )* } };
                let struct_fields: Vec<_> = d.named.iter().map(|f| {
                    let vis = &f.vis;
                    let name = f.ident.as_ref().unwrap();
                    let ty = &f.ty;
                    return quote! {
                        #vis #name: <#ty as ::validates::Validates>::Target,
                    };
                }).collect();
                struct_args = quote! { { #( #struct_fields )* } };
                let clone_fields: Vec<_> = d.named.iter().map(|f| {
                    let name = f.ident.as_ref().unwrap();
                    return quote! {
                        #name: ::std::clone::Clone::clone(&self.#name),
                    };
                }).collect();
                clone_args = quote! { { #( #clone_fields )* } };
            },
            Fields::Unnamed(d) => {
                let ctor_fields: Vec<_> = d.unnamed.iter().enumerate().map(|(name, f)| {
                    let mangle = compute_mangle_expr(&f.attrs, format!("#{}", name));
                    let name = Index::from(name);
                    return quote! {
                        ::validates::Validates::validate(self.#name) #mangle ?,
                    };
                }).collect();
                ctor_args = quote! { ( #( #ctor_fields )* ) };
                let struct_fields: Vec<_> = d.unnamed.iter().enumerate().map(|(_name, f)| {
                    let vis = &f.vis;
                    let ty = &f.ty;
                    return quote! {
                        #vis <#ty as ::validates::Validates>::Target,
                    };
                }).collect();
                struct_args = quote! { ( #( #struct_fields )* ); };
                let clone_fields: Vec<_> = d.unnamed.iter().enumerate().map(|(name, _f)| {
                    let name = Index::from(name);
                    return quote! {
                        ::std::clone::Clone::clone(&self.#name),
                    };
                }).collect();
                clone_args = quote! { ( #( #clone_fields )* ) };
            },
            Fields::Unit => {
                ctor_args = quote! { () };
                struct_args = quote! { () };
                clone_args = quote! { () };
            },
        },
        _ => panic!("#[derive(Validates)] on something unexpected"),
    };

    let vis = &ast.vis;
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let ident_validated = Ident::new(&format!("{}Validated", ident), Span::call_site());
    let gen = quote! {
        impl #impl_generics ::validates::Validates for #ident #ty_generics #where_clause {
            type Target = #ident_validated #ty_generics;

            fn validate(self) -> ::validates::ValidationResult<Self::Target> {
                return Result::Ok(#ident_validated #ctor_args);
            }
        }

        #vis struct #ident_validated #impl_generics #struct_args #where_clause

        impl #impl_generics Clone for #ident_validated #ty_generics #where_clause {
            fn clone(&self) -> Self {
                return #ident_validated #clone_args;
            }
        }
    };

    return TokenStream::from(gen);
}

fn compute_mangle_expr(attrs: &Vec<Attribute>, default_name: String) -> proc_macro2::TokenStream {
    let user_name = attrs.iter().filter_map(|a| {
        let a = a.interpret_meta()?;
        if a.name() != "ValidatesName" {
            return None;
        }
        match a {
            Meta::NameValue(ref nv) => {
                match nv.lit {
                    Lit::Str(ref s) => {
                        return Some(s.value());
                    }
                    _ => {
                        panic!("Unexpected ValidatesName attribute: {:?}", a);
                    }
                }
            }
            _ => {
                panic!("Unexpected ValidatesName attribute: {:?}", a);
            }
        }
    }).next().unwrap_or(default_name);
    if user_name.is_empty() {
        return quote! { };
    }

    let prefix = LitStr::new(&format!("While validating {}", user_name), Span::call_site());
    return quote! { .map_err(|e| e.label(#prefix)) };
}
