use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, AttributeArgs, ItemFn, NestedMeta, Meta, Lit, Pat, FnArg, Type,
};
use proc_macro_crate::{crate_name, FoundCrate};

/// Resolve host crate path (equivalent to `$crate`)
fn host_crate() -> proc_macro2::TokenStream {
    match crate_name("mini_langchain") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(::#ident)
        }
        Err(_) => quote!(::mini_langchain),
    }
}

#[proc_macro_attribute]
pub fn tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AttributeArgs);
    let input_fn = parse_macro_input!(item as ItemFn);

    let mut name_override = None;
    let mut description = None;
    let mut params_meta = Vec::<(String, String)>::new();

    for nested in args {
        match nested {
            NestedMeta::Meta(Meta::NameValue(nv)) => {
                if let Some(ident) = nv.path.get_ident() {
                    if let Lit::Str(s) = nv.lit {
                        match ident.to_string().as_str() {
                            "name" => name_override = Some(s.value()),
                            "description" => description = Some(s.value()),
                            _ => {}
                        }
                    }
                }
            }
            NestedMeta::Meta(Meta::List(list)) if list.path.is_ident("params") => {
                for nm in list.nested {
                    if let NestedMeta::Meta(Meta::NameValue(nv)) = nm {
                        if let (Some(ident), Lit::Str(s)) =
                            (nv.path.get_ident(), &nv.lit)
                        {
                            params_meta.push((ident.to_string(), s.value()));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let description = match description {
        Some(d) => d,
        None => {
            return syn::Error::new_spanned(
                &input_fn.sig.ident,
                "tool requires `description = \"...\"`",
            )
            .to_compile_error()
            .into();
        }
    };

    let fn_ident = input_fn.sig.ident.clone();
    let fn_name = fn_ident.to_string();
    let tool_name = name_override.unwrap_or(fn_name.clone());

    let mut fields = Vec::new();
    let mut param_names = Vec::new();

    for arg in &input_fn.sig.inputs {
        match arg {
            FnArg::Typed(pt) => {
                if let Pat::Ident(pi) = &*pt.pat {
                    fields.push((pi.ident.clone(), (*pt.ty).clone()));
                    param_names.push(pi.ident.to_string());
                } else {
                    return syn::Error::new_spanned(
                        &pt.pat,
                        "only simple identifiers are supported",
                    )
                    .to_compile_error()
                    .into();
                }
            }
            FnArg::Receiver(_) => {
                return syn::Error::new_spanned(
                    arg,
                    "methods with self are not supported",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    for (k, _) in &params_meta {
        if !param_names.contains(k) {
            return syn::Error::new_spanned(
                &input_fn.sig.ident,
                format!("param '{}' not found in function signature", k),
            )
            .to_compile_error()
            .into();
        }
    }

    let params_struct_ident =
        syn::Ident::new(&format!("{}Params", pascal_case(&fn_name)), fn_ident.span());
    let tool_struct_ident =
        syn::Ident::new(&format!("{}Tool", pascal_case(&fn_name)), fn_ident.span());

    let host = host_crate();

    let field_defs = fields.iter().map(|(id, ty)| {
        quote!(pub #id: #ty)
    });

    let args_entries = fields.iter().map(|(ident, ty)| {
        let desc = params_meta
            .iter()
            .find(|(k, _)| k == &ident.to_string())
            .map(|(_, v)| v.clone())
            .unwrap_or_default();

        if desc.is_empty() {
            return syn::Error::new_spanned(
                ident,
                format!("missing description for param '{}'", ident),
            )
            .to_compile_error();
        }

        let arg_type = infer_json_type(ty);
        let name_lit = syn::LitStr::new(&ident.to_string(), ident.span());
        let desc_lit = syn::LitStr::new(&desc, ident.span());

        quote! {
            #host::tools::traits::ArgSchema {
                name: #name_lit.into(),
                arg_type: #arg_type.into(),
                description: #desc_lit.into(),
                required: true,
            }
        }
    });

    let call_args = fields.iter().map(|(id, _)| quote!(params.#id));
    let is_async = input_fn.sig.asyncness.is_some();

    let run_body = if is_async {
        quote! {
            let params: #params_struct_ident =
                #host::serde_json::from_value(input)
                    .map_err(|e| #host::tools::error::ToolError::ParamsNotMatched(e.to_string()))?;
            Ok(#fn_ident(#(#call_args),*).await)
        }
    } else {
        quote! {
            let params: #params_struct_ident =
                #host::serde_json::from_value(input)
                    .map_err(|e| #host::tools::error::ToolError::ParamsNotMatched(e.to_string()))?;
            Ok(#fn_ident(#(#call_args),*))
        }
    };

    let expanded = quote! {
        #input_fn

        #[derive(#host::serde::Deserialize)]
        pub struct #params_struct_ident {
            #(#field_defs,)*
        }

        pub struct #tool_struct_ident;

        #[#host::async_trait::async_trait]
        impl #host::tools::traits::Tool for #tool_struct_ident {
            fn name(&self) -> &str { #tool_name }
            fn description(&self) -> &str { #description }
            fn args(&self) -> Vec<#host::tools::traits::ArgSchema> {
                vec![#(#args_entries),*]
            }
            async fn run(
                &self,
                input: #host::serde_json::Value,
            ) -> Result<String, #host::tools::error::ToolError> {
                #run_body
            }
        }
    };

    TokenStream::from(expanded)
}

fn pascal_case(s: &str) -> String {
    s.split('_')
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn infer_json_type(ty: &Type) -> &'static str {
    match ty {
        Type::Path(p) => {
            let ident = p.path.segments.last().unwrap().ident.to_string();
            match ident.as_str() {
                "String" => "string",
                "bool" => "boolean",
                "i8" | "i16" | "i32" | "i64" |
                "u8" | "u16" | "u32" | "u64" |
                "usize" | "isize" => "integer",
                "f32" | "f64" => "number",
                "Vec" => "array",
                _ => "object",
            }
        }
        _ => "object",
    }
}