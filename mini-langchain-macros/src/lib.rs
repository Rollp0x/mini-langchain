use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, AttributeArgs, ItemFn, NestedMeta, Meta, Lit, Pat, FnArg};
use proc_macro_crate::{crate_name, FoundCrate};

/// Minimal attribute macro #[tool(...)]
/// Supports: name (optional), description (required), params(...)
/// params syntax: params( city = "desc", units = "desc" )
#[proc_macro_attribute]
pub fn tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    // parse the attribute arguments
    let args = parse_macro_input!(attr as AttributeArgs);
    let input_fn = parse_macro_input!(item as ItemFn);
    // collect name and description and params map
    let mut name_override: Option<String> = None;
    let mut description: Option<String> = None;
    let mut params_meta: Vec<(String, String)> = Vec::new();

    for nested in args.into_iter() {
        match nested {
            NestedMeta::Meta(Meta::NameValue(nv)) => {
                let ident = nv.path.get_ident().map(|i| i.to_string());
                if let Some(key) = ident {
                    match nv.lit {
                        Lit::Str(s) => {
                            if key == "name" {
                                name_override = Some(s.value());
                            } else if key == "description" {
                                description = Some(s.value());
                            }
                        }
                        _ => {}
                    }
                }
            }
            NestedMeta::Meta(Meta::List(list)) => {
                if list.path.is_ident("params") {
                    for nm in list.nested.iter() {
                        match nm {
                            NestedMeta::Meta(Meta::NameValue(nv)) => {
                                if let Some(ident) = nv.path.get_ident() {
                                    if let Lit::Str(s) = &nv.lit {
                                        params_meta.push((ident.to_string(), s.value()));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // ensure description exists
    if description.is_none() {
        return syn::Error::new_spanned(&input_fn.sig.ident, "tool attribute requires a 'description' = \"...\"")
            .to_compile_error()
            .into();
    }

    let fn_name = input_fn.sig.ident.to_string();
    let tool_name = name_override.unwrap_or(fn_name.clone());
    let description = description.unwrap();

    // build Params struct fields from function signature
    let mut fields = Vec::new();
    let mut param_names = Vec::new();
    for input in input_fn.sig.inputs.iter() {
        match input {
            FnArg::Typed(pt) => {
                // pattern must be an identifier
                if let Pat::Ident(pi) = &*pt.pat {
                    let ident = pi.ident.clone();
                    let ty = &*pt.ty;
                    fields.push((ident.clone(), ty.clone()));
                    param_names.push(ident.to_string());
                } else {
                    return syn::Error::new_spanned(&pt.pat, "unsupported pattern in function parameters; use simple identifiers")
                        .to_compile_error()
                        .into();
                }
            }
            FnArg::Receiver(_) => {
                return syn::Error::new_spanned(input, "methods with self are not supported; use free functions")
                    .to_compile_error()
                    .into();
            }
        }
    }

    // check params_meta keys match function params
    for (k, _v) in params_meta.iter() {
        if !param_names.contains(k) {
            return syn::Error::new_spanned(&input_fn.sig.ident, format!("params list contains '{}' but function has no parameter with that name", k))
                .to_compile_error()
                .into();
        }
    }

    // create Params struct identifier
    let params_struct_ident = syn::Ident::new(&format!("{}Params", pascal_case(&fn_name)), input_fn.sig.ident.span());
    let tool_struct_ident = syn::Ident::new(&format!("{}Tool", pascal_case(&fn_name)), input_fn.sig.ident.span());

    // generate fields tokens
    let field_defs: Vec<proc_macro2::TokenStream> = fields.iter().map(|(ident, ty)| {
        quote! { pub #ident: #ty }
    }).collect();

    // build ArgSchema vector tokens
    // figure out how the host crate refers to our library (it might be `crate` when
    // expanding inside the library itself, or an external name when used from examples
    // or other crates). Use `proc_macro_crate` to discover the correct root.
    let host_crate_root = match crate_name("mini-langchain") {
        Ok(FoundCrate::Itself) => quote! { crate },
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote! { ::#ident }
        }
        Err(_) => quote! { ::mini_langchain },
    };

    let mut args_entries = Vec::new();
    for (ident, _ty) in fields.iter() {
        // find description for this param
        let mut desc = String::new();
        for (k, v) in params_meta.iter() {
            if k == &ident.to_string() { desc = v.clone(); break; }
        }
        if desc.is_empty() {
            // emit error: description missing
            return syn::Error::new_spanned(&input_fn.sig.ident, format!("missing description for parameter '{}' in tool attribute params(...)", ident))
                .to_compile_error()
                .into();
        }
        let name_lit = syn::LitStr::new(&ident.to_string(), ident.span());
        let desc_lit = syn::LitStr::new(&desc, ident.span());
        args_entries.push(quote! {
            #host_crate_root ::tools::traits::ArgSchema {
                name: #name_lit.to_string(),
                arg_type: "string".to_string(),
                description: #desc_lit.to_string(),
                required: true,
            }
        });
    }

    // prepare calling the original function: collect param idents
    let call_args: Vec<proc_macro2::TokenStream> = fields.iter().map(|(ident, _)| {
        quote! { params.#ident }
    }).collect();

    // determine if the original function is async
    let is_async = input_fn.sig.asyncness.is_some();

    // generate output tokens
    let fn_tokens = input_fn.to_token_stream();
    let fn_ident = input_fn.sig.ident.clone();
    let params_struct_ident2 = params_struct_ident.clone();
    let tool_struct_ident2 = tool_struct_ident.clone();
    let tool_name_lit = syn::LitStr::new(&tool_name, input_fn.sig.ident.span());
    let description_lit = syn::LitStr::new(&description, input_fn.sig.ident.span());

    let run_body = if is_async {
        quote! {
            let params: #params_struct_ident2 = serde_json::from_value(input)
                .map_err(|e| crate::tools::error::ToolError::ParamsNotMatched(e.to_string()))?;
            let out = #fn_ident( #(#call_args),* ).await;
            Ok(out)
        }
    } else {
        quote! {
            let params: #params_struct_ident2 = serde_json::from_value(input)
                .map_err(|e| crate::tools::error::ToolError::ParamsNotMatched(e.to_string()))?;
            let out = #fn_ident( #(#call_args),* );
            Ok(out)
        }
    };

    // figure out how the host crate refers to our library (it might be `crate` when
    // expanding inside the library itself, or an external name when used from examples
    // or other crates). Use `proc_macro_crate` to discover the correct root.
    let host_crate_root = match crate_name("mini-langchain") {
        Ok(FoundCrate::Itself) => quote! { crate },
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote! { ::#ident }
        }
        Err(_) => quote! { ::mini_langchain },
    };

    let expanded = quote! {
        #fn_tokens

        #[derive(serde::Deserialize)]
        pub struct #params_struct_ident2 {
            #(#field_defs,)*
        }

        pub struct #tool_struct_ident2;
        #[async_trait::async_trait]
        impl #host_crate_root ::tools::traits::Tool for #tool_struct_ident2 {
            fn name(&self) -> &str { #tool_name_lit }
            fn description(&self) -> &str { #description_lit }
            fn args(&self) -> Vec<#host_crate_root ::tools::traits::ArgSchema> {
                vec![ #(#args_entries),* ]
            }

            async fn run(&self, input: serde_json::Value) -> Result<String, #host_crate_root ::tools::error::ToolError> {
                #run_body
            }
        }
    };

    TokenStream::from(expanded)
}

fn pascal_case(s: &str) -> String {
    s.split('_').map(|part| {
        let mut c = part.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str()
        }
    }).collect::<Vec<_>>().join("")
}
