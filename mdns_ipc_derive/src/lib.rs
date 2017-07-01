extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
enum Attribute {
}
enum FieldAttribute {
	Const(quote::Tokens),
	HexDisplay,
}

fn attr_get_single_list_arg(attr: &syn::Attribute) -> quote::Tokens {
	match attr.value {
		syn::MetaItem::List(_, ref l) => {
			if l.len() != 1 {
				panic!("{:?} attribute requires exactly one argument", attr.name());
			}
			let arg = &l[0];
			quote!{#arg}
		},
		syn::MetaItem::NameValue(_, ref arg) => {
			quote!{#arg}
		}
		_ => panic!("{:?} argument requires one argument like: [#{}(...)]", attr.name(), attr.name()),
	}
}

struct AttributeParser<'a>(pub &'a [syn::Attribute]);
impl<'a> Iterator for AttributeParser<'a> {
	type Item = (Attribute, &'a syn::Attribute);

	fn next(&mut self) -> Option<Self::Item> {
		while !self.0.is_empty() {
			let a = &self.0[0];
			self.0 = &self.0[1..];
			if a.is_sugared_doc { continue; }
			match a.value.name() {
				"Const" => {
					panic!("Attribute {} not supported on struct when deriving IpcStruct", a.value.name())
				},
				"HexDisplay" => {
					panic!("Attribute {} not supported on struct when deriving IpcStruct", a.value.name())
				},
				_ => (),
			}
		}
		None
	}
}

struct FieldAttributeParser<'a>(pub &'a [syn::Attribute]);
impl<'a> Iterator for FieldAttributeParser<'a> {
	type Item = (FieldAttribute, &'a syn::Attribute);

	fn next(&mut self) -> Option<Self::Item> {
		while !self.0.is_empty() {
			let a = &self.0[0];
			self.0 = &self.0[1..];
			if a.is_sugared_doc { continue; }
			match a.value.name() {
				"Const" => {
					return Some((FieldAttribute::Const(attr_get_single_list_arg(a)), a));
				},
				"HexDisplay" => {
					return Some((FieldAttribute::HexDisplay, a));
				},
				_ => (),
			}
		}
		None
	}
}

#[proc_macro_derive(IpcStruct, attributes(Const,HexDisplay))]
pub fn derive_ipc_struct(input: TokenStream) -> TokenStream {
	// Construct a string representation of the type definition
	let s = input.to_string();

	// Parse the string representation
	let ast = syn::parse_derive_input(&s).unwrap();

	for _ in AttributeParser(&ast.attrs) {
	}

	let (is_tuple, fields) = match ast.body {
		syn::Body::Enum(_) => panic!("Deriving IpcStruct not supported for enum types"),
		syn::Body::Struct(syn::VariantData::Struct(s)) => (false, s),
		syn::Body::Struct(syn::VariantData::Tuple(s)) => (true, s),
		syn::Body::Struct(_) => panic!("Deriving IpcStruct not supported for unit struct types"),
	};

	if !ast.generics.ty_params.is_empty() {
		panic!("Type parameters not supported for deriving IpcStruct");
	}

	if !ast.generics.where_clause.predicates.is_empty() {
		panic!("Where clauses not supported for deriving IpcStruct");
	}

	if !ast.generics.lifetimes.is_empty() {
		panic!("Lifetimes not supported for deriving IpcStruct");
	}

	//panic!("{:?}", ast.generics);

	let name = &ast.ident;

	let mut parse_fields = quote!{};
	let mut ser_fields = quote!{};
	let mut calc_size_fields = quote!{0};
	let mut size_hint_fields = quote!{::mdns_ipc_core::SizeHint::Exact(0)};
	let mut show_fields = quote!{};

	for (field_ndx, field) in fields.iter().enumerate() {
		let mut a_const = None;
		let mut a_hex_display = false;
		for (attr, orig) in FieldAttributeParser(&field.attrs) {
			match attr {
				FieldAttribute::Const(value) => {
					if a_const.is_some() { panic!("Attribute {:?} must be given at most once for each field", orig.value.name()); }
					a_const = Some(value);
				},
				FieldAttribute::HexDisplay => {
					if a_hex_display { panic!("Attribute {:?} must be given at most once for each field", orig.value.name()); }
					a_hex_display = true;
				},
			}
		}
		let (field_prefix, field_name) = match field.ident.as_ref() {
			Some(fname) => (quote!{#fname:}, quote!{#fname}),
			None => {
				let field_ndx = syn::Lit::Int(field_ndx as u64, syn::IntTy::Unsuffixed);
				(quote!{}, quote!{#field_ndx})
			},
		};

		let field_ty = &field.ty;
		let (parse_data, ser_data, calc_size, size_hint) = match a_const {
			None => (
				quote!{
					<#field_ty as ::mdns_ipc_core::Deserialize>::deserialize(_src)
				},
				quote!{
					<#field_ty as ::mdns_ipc_core::Serialize>::serialize(&self.#field_name, _dst)
				},
				quote!{
					<#field_ty as ::mdns_ipc_core::Serialize>::serialized_size(&self.#field_name)
				},
				quote!{
					<#field_ty as ::mdns_ipc_core::Deserialize>::deserialize_size_hint()
				},
			),
			Some(ref value) => (
				quote!{
					<#field_ty as ::mdns_ipc_core::ConstCheck<_>>::deserialize(_src, #value)
				},
				quote!{
					<#field_ty as ::mdns_ipc_core::ConstCheck<_>>::serialize(_dst, #value)
				},
				quote!{
					<#field_ty as ::mdns_ipc_core::ConstCheck<_>>::serialized_size(#value)
				},
				quote!{
					<#field_ty as ::mdns_ipc_core::ConstCheck<_>>::deserialize_size_hint(#value)
				},
			),
		};

		parse_fields = quote!{#parse_fields
			#field_prefix {
				match #parse_data {
					Ok(val) => val,
					Err(e) => {
						debug!("failed parsing ipc struct field {}::{}: {:?}", stringify!(#name), stringify!(#field_name), e);
						return Err(e);
					}
				}
			},
		};

		ser_fields = quote!{#ser_fields
			match #ser_data {
				Ok(val) => val,
				Err(e) => {
					debug!("failed serializing ipc struct field {}::{}: {:?}", stringify!(#name), stringify!(#field_name), e);
					return Err(e);
				}
			}
		};

		calc_size_fields = quote!{#calc_size_fields +
			match #calc_size {
				Ok(val) => val,
				Err(e) => {
					debug!("failed calculating max size for ipc struct field {}::{}: {:?}", stringify!(#name), stringify!(#field_name), e);
					return Err(e);
				}
			}
		};
		size_hint_fields = quote!{#size_hint_fields + #size_hint};

		show_fields = match a_const {
			None => if a_hex_display {
				quote!{
					#show_fields
					.field(stringify!(#field_name), &::mdns_ipc_core::HexDisplay(&self.#field_name))
				}
			} else {
				quote!{
					#show_fields
					.field(stringify!(#field_name), &self.#field_name)
				}
			},
			Some(ref value) => quote!{
				#show_fields
				.field(stringify!(#field_name), &#value)
			}
		};
	}

	let parse_fields = if is_tuple {
		quote!{ #name(#parse_fields) }
	} else {
		quote!{ #name{#parse_fields} }
	};

	let gen = quote! {
		impl ::mdns_ipc_core::Serialize for #name {
			fn serialize(&self, _dst: &mut ::bytes::BytesMut) -> ::std::io::Result<()> {
				#ser_fields
				Ok(())
			}
			fn serialized_size(&self) -> ::std::io::Result<usize> {
				Ok(#calc_size_fields)
			}
		}

		impl ::mdns_ipc_core::Deserialize for #name {
			fn deserialize(_src: &mut ::std::io::Cursor<::bytes::Bytes>) -> ::std::io::Result<Self> {
				Ok(#parse_fields)
			}

			fn deserialize_size_hint() -> ::mdns_ipc_core::SizeHint {
				#size_hint_fields
			}
		}

		impl ::std::fmt::Debug for #name {
			fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
				f.debug_struct(stringify!(#name))
					#show_fields
					.finish()
			}
		}
	};

	// panic!(gen.to_string());

	// Return the generated impl
	gen.parse().unwrap()
}

#[proc_macro_derive(IpcStructDisplay, attributes(Const,HexDisplay))]
pub fn derive_ipc_struct_display(input: TokenStream) -> TokenStream {
	// Construct a string representation of the type definition
	let s = input.to_string();

	// Parse the string representation
	let ast = syn::parse_derive_input(&s).unwrap();

	for _ in AttributeParser(&ast.attrs) {
	}

	let fields = match ast.body {
		syn::Body::Enum(_) => panic!("Deriving IpcStruct/Display not supported for enum types"),
		syn::Body::Struct(syn::VariantData::Struct(s)) => s,
		syn::Body::Struct(syn::VariantData::Tuple(s)) => s,
		syn::Body::Struct(_) => panic!("Deriving IpcStruct/Display not supported for unit struct types"),
	};

	if !ast.generics.ty_params.is_empty() {
		panic!("Type parameters not supported for deriving IpcStruct/Display");
	}

	if !ast.generics.where_clause.predicates.is_empty() {
		panic!("Where clauses not supported for deriving IpcStruct/Display");
	}

	if !ast.generics.lifetimes.is_empty() {
		panic!("Lifetimes not supported for deriving IpcStruct/Display");
	}

	//panic!("{:?}", ast.generics);

	let name = &ast.ident;

	let mut show_fields = quote!{};
	for (field_ndx, field) in fields.iter().enumerate() {
		let mut a_const = None;
		let mut a_hex_display = false;
		for (attr, orig) in FieldAttributeParser(&field.attrs) {
			match attr {
				FieldAttribute::Const(value) => {
					if a_const.is_some() { panic!("Attribute {:?} must be given at most once for each field", orig.value.name()); }
					a_const = Some(value);
				},
				FieldAttribute::HexDisplay => {
					if a_hex_display { panic!("Attribute {:?} must be given at most once for each field", orig.value.name()); }
					a_hex_display = true;
				},
			}
		}
		let field_name = match field.ident.as_ref() {
			Some(fname) => quote!{#fname},
			None => {
				let field_ndx = syn::Lit::Int(field_ndx as u64, syn::IntTy::Unsuffixed);
				quote!{#field_ndx}
			},
		};

		show_fields = match a_const {
			None => if a_hex_display {
				quote!{
					#show_fields
					.field(stringify!(#field_name), &::mdns_ipc_core::HexDisplay(&self.#field_name))
				}
			} else {
				quote!{
					#show_fields
					.field(stringify!(#field_name), &D(&self.#field_name))
				}
			},
			Some(ref value) => quote!{
				#show_fields
				.field(stringify!(#field_name), &#value)
			}
		};
	}

	let debug_wrapper = quote! {
		#[allow(dead_code)]
		struct D<'a, T:'a>(&'a T);
		impl<'a, T: fmt::Display+'a> fmt::Debug for D<'a, T> {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				fmt::Display::fmt(&self.0, f)
			}
		}
	};

	let gen = quote! {
		impl ::std::fmt::Display for #name {
			fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
				use std::fmt;
				#debug_wrapper
				f.debug_struct(stringify!(#name))
					#show_fields
					.finish()
			}
		}
	};

	// panic!(gen.to_string());

	// Return the generated impl
	gen.parse().unwrap()
}
