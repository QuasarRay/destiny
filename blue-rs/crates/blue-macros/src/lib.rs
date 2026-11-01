use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::Parser, parse_macro_input, parse_quote, spanned::Spanned, Attribute, Data, DeriveInput, Fields, ImplItem,
    FnArg, ItemEnum, ItemFn, ItemImpl, ItemStruct, LitStr, Pat, ReturnType, Signature, Type,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BlueKind {
    Component,
    Resource,
}

impl BlueKind {
    fn from_str(value: &str) -> syn::Result<Self> {
        match value {
            "component" => Ok(Self::Component),
            "resource" => Ok(Self::Resource),
            other => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("unsupported Blue class kind `{other}`; expected `component` or `resource`"),
            )),
        }
    }
}

#[derive(Debug)]
struct ClassArgs {
    module: String,
    name: Option<String>,
    doc: Option<String>,
    kind: BlueKind,
    subclass: bool,
}

impl Default for ClassArgs {
    fn default() -> Self {
        Self {
            module: "blue".to_owned(),
            name: None,
            doc: None,
            kind: BlueKind::Component,
            subclass: false,
        }
    }
}

fn parse_class_args(tokens: proc_macro2::TokenStream) -> syn::Result<ClassArgs> {
    let mut args = ClassArgs::default();
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("module") {
            args.module = meta.value()?.parse::<LitStr>()?.value();
            return Ok(());
        }
        if meta.path.is_ident("name") {
            args.name = Some(meta.value()?.parse::<LitStr>()?.value());
            return Ok(());
        }
        if meta.path.is_ident("doc") {
            args.doc = Some(meta.value()?.parse::<LitStr>()?.value());
            return Ok(());
        }
        if meta.path.is_ident("kind") {
            args.kind = BlueKind::from_str(&meta.value()?.parse::<LitStr>()?.value())?;
            return Ok(());
        }
        if meta.path.is_ident("subclass") {
            args.subclass = true;
            return Ok(());
        }
        Err(meta.error("unsupported #[blue_pyclass(...)] option"))
    });
    parser.parse2(tokens)?;
    Ok(args)
}


fn descriptor_ident(ident: &syn::Ident, suffix: &str) -> syn::Ident {
    let base = ident.to_string().to_ascii_uppercase();
    format_ident!("{base}_{suffix}")
}

fn is_option_type(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    path.qself.is_none()
        && path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "Option")
}

fn has_explicit_pyo3_signature(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("pyo3")
            && attr.meta.to_token_stream().to_string().contains("signature")
    })
}

fn inferred_optional_signature(sig: &Signature, attrs: &[Attribute]) -> Option<Attribute> {
    if has_explicit_pyo3_signature(attrs) {
        return None;
    }

    let mut parameters = Vec::new();
    let mut saw_optional = false;

    for input in &sig.inputs {
        let FnArg::Typed(argument) = input else {
            continue;
        };
        let Pat::Ident(pattern) = argument.pat.as_ref() else {
            return None;
        };
        let ident = &pattern.ident;
        if is_option_type(argument.ty.as_ref()) {
            saw_optional = true;
            parameters.push(quote!(#ident = None));
        } else {
            if saw_optional {
                // Required-after-optional needs an explicit user-authored signature.
                return None;
            }
            parameters.push(quote!(#ident));
        }
    }

    saw_optional.then(|| parse_quote!(#[pyo3(signature = (#(#parameters),*))]))
}

fn doc_string(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| match &attr.meta {
            syn::Meta::NameValue(value) => match &value.value {
                syn::Expr::Lit(expr) => match &expr.lit {
                    syn::Lit::Str(lit) => Some(lit.value().trim().to_owned()),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Default)]
struct FieldFlags {
    read: bool,
    write: bool,
    persist: bool,
    skip: bool,
}

fn field_flags(attrs: &[Attribute]) -> syn::Result<FieldFlags> {
    let mut flags = FieldFlags::default();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("blue")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("read") {
                flags.read = true;
            } else if meta.path.is_ident("write") {
                flags.write = true;
            } else if meta.path.is_ident("persist") {
                flags.persist = true;
            } else if meta.path.is_ident("skip") {
                flags.skip = true;
            }
            Ok(())
        })?;
    }
    Ok(flags)
}

fn derive_args(input: &DeriveInput) -> syn::Result<ClassArgs> {
    let mut args = ClassArgs::default();
    for attr in input.attrs.iter().filter(|attr| attr.path().is_ident("blue")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("module") {
                args.module = meta.value()?.parse::<LitStr>()?.value();
            } else if meta.path.is_ident("name") {
                args.name = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("doc") {
                args.doc = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("kind") {
                args.kind = BlueKind::from_str(&meta.value()?.parse::<LitStr>()?.value())?;
            } else if meta.path.is_ident("subclass") {
                args.subclass = true;
            }
            Ok(())
        })?;
    }
    Ok(args)
}

#[proc_macro_derive(BlueClass, attributes(blue))]
pub fn derive_blue_class(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_blue_class(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand_blue_class(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &input.ident;
    let args = derive_args(input)?;
    let module = LitStr::new(&args.module, ident.span());
    let name_value = args.name.unwrap_or_else(|| ident.to_string());
    let name = LitStr::new(&name_value, ident.span());
    let doc_value = args.doc.unwrap_or_else(|| doc_string(&input.attrs));
    let doc = LitStr::new(&doc_value, ident.span());
    let class_kind = match args.kind {
        BlueKind::Component => quote!(::blueexposure::BlueClassKind::Component),
        BlueKind::Resource => quote!(::blueexposure::BlueClassKind::Resource),
    };

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .filter_map(|field| field.ident.as_ref().map(|name| (name, field)))
                .map(|(field_ident, field)| {
                    let flags = field_flags(&field.attrs)?;
                    if flags.skip {
                        return Ok(None);
                    }
                    let field_name = LitStr::new(&field_ident.to_string(), field_ident.span());
                    let read = flags.read;
                    let write = flags.write;
                    let persist = flags.persist;
                    Ok(Some(quote! {
                        ::blueexposure::BlueFieldInfo::new(#field_name, #read, #write, #persist)
                    }))
                })
                .collect::<syn::Result<Vec<_>>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            Fields::Unnamed(fields) => fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let flags = field_flags(&field.attrs)?;
                    if flags.skip {
                        return Ok(None);
                    }
                    let field_name = LitStr::new(&format!("_{index}"), field.span());
                    let read = flags.read;
                    let write = flags.write;
                    let persist = flags.persist;
                    Ok(Some(quote! {
                        ::blueexposure::BlueFieldInfo::new(#field_name, #read, #write, #persist)
                    }))
                })
                .collect::<syn::Result<Vec<_>>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            Fields::Unit => Vec::new(),
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "BlueClass can only be derived for structs",
            ));
        }
    };

    Ok(quote! {
        impl ::blueexposure::BlueClass for #ident {
            const BLUE_MODULE: &'static str = #module;
            const BLUE_NAME: &'static str = #name;

            fn blue_class_info() -> ::blueexposure::BlueClassInfo {
                ::blueexposure::BlueClassInfo::new(
                    #module,
                    #name,
                    #doc,
                    #class_kind,
                    vec![#(#fields),*],
                )
            }
        }
    })
}

#[proc_macro_attribute]
pub fn blue_pyclass(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = match parse_class_args(attr.into()) {
        Ok(args) => args,
        Err(error) => return error.into_compile_error().into(),
    };
    let mut item = parse_macro_input!(item as ItemStruct);

    let name_value = args.name.clone().unwrap_or_else(|| item.ident.to_string());
    let module = LitStr::new(&args.module, item.ident.span());
    let name = LitStr::new(&name_value, item.ident.span());
    let doc_value = args.doc.clone().unwrap_or_else(|| doc_string(&item.attrs));
    let doc = LitStr::new(&doc_value, item.ident.span());
    let kind_lit = LitStr::new(
        match args.kind {
            BlueKind::Component => "component",
            BlueKind::Resource => "resource",
        },
        item.ident.span(),
    );

    for field in item.fields.iter_mut() {
        match field_flags(&field.attrs) {
            Ok(flags) if flags.read && flags.write => field.attrs.push(parse_quote!(#[pyo3(get, set)])),
            Ok(flags) if flags.read => field.attrs.push(parse_quote!(#[pyo3(get)])),
            Ok(flags) if flags.write => field.attrs.push(parse_quote!(#[pyo3(set)])),
            Ok(_) => {}
            Err(error) => return error.into_compile_error().into(),
        }
    }

    let pyclass_attr = if args.subclass {
        quote!(#[pyo3::pyclass(module = #module, name = #name, subclass)])
    } else {
        quote!(#[pyo3::pyclass(module = #module, name = #name)])
    };
    let bevy_derives = match args.kind {
        BlueKind::Component => quote! {
            #[derive(bevy::prelude::Component, bevy::prelude::Reflect, blueexposure::BlueClass)]
            #[reflect(Component)]
        },
        BlueKind::Resource => quote! {
            #[derive(bevy::prelude::Resource, bevy::prelude::Reflect, blueexposure::BlueClass)]
            #[reflect(Resource)]
        },
    };

    quote! {
        #pyclass_attr
        #bevy_derives
        #[blue(module = #module, name = #name, doc = #doc, kind = #kind_lit)]
        #item
    }
    .into()
}


#[proc_macro_attribute]
pub fn blue_function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = match parse_class_args(attr.into()) {
        Ok(args) => args,
        Err(error) => return error.into_compile_error().into(),
    };
    let mut function = parse_macro_input!(item as ItemFn);
    if let Some(signature) = inferred_optional_signature(&function.sig, &function.attrs) {
        function.attrs.push(signature);
    }
    let ident = &function.sig.ident;
    let descriptor = descriptor_ident(ident, "BLUE_FUNCTION");
    let register = format_ident!("__blue_register_{ident}");
    let module = LitStr::new(&args.module, ident.span());
    let name_value = args.name.unwrap_or_else(|| ident.to_string());
    let name = LitStr::new(&name_value, ident.span());
    let doc_value = args.doc.unwrap_or_else(|| doc_string(&function.attrs));
    let doc = LitStr::new(&doc_value, ident.span());
    let vis = &function.vis;

    quote! {
        #[pyo3::pyfunction]
        #[pyo3(name = #name)]
        #function

        fn #register(module: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
            use pyo3::types::PyModuleMethods as _;
            module.add_function(pyo3::wrap_pyfunction!(#ident, module)?)?;
            Ok(())
        }

        #vis const #descriptor: ::blueexposure::BlueFunctionDescriptor =
            ::blueexposure::BlueFunctionDescriptor::new(#module, #name, #doc, #register);
    }
    .into()
}

#[proc_macro_attribute]
pub fn blue_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = match parse_class_args(attr.into()) {
        Ok(args) => args,
        Err(error) => return error.into_compile_error().into(),
    };
    let item = parse_macro_input!(item as ItemEnum);
    for variant in &item.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return syn::Error::new_spanned(
                variant,
                "#[blue_enum] currently supports C-like unit variants only",
            )
            .into_compile_error()
            .into();
        }
    }

    let ident = &item.ident;
    let descriptor = descriptor_ident(ident, "BLUE_ENUM");
    let py_register = format_ident!("__blue_register_py_enum_{ident}");
    let bevy_register = format_ident!("__blue_register_bevy_enum_{ident}");
    let module = LitStr::new(&args.module, ident.span());
    let name_value = args.name.unwrap_or_else(|| ident.to_string());
    let name = LitStr::new(&name_value, ident.span());
    let doc_value = args.doc.unwrap_or_else(|| doc_string(&item.attrs));
    let doc = LitStr::new(&doc_value, ident.span());
    let vis = &item.vis;
    let variants = item
        .variants
        .iter()
        .map(|variant| LitStr::new(&variant.ident.to_string(), variant.ident.span()))
        .collect::<Vec<_>>();

    quote! {
        #[pyo3::pyclass(module = #module, name = #name, eq, eq_int)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, bevy::prelude::Reflect)]
        #item

        fn #py_register(module: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
            use pyo3::types::PyModuleMethods as _;
            module.add_class::<#ident>()
        }

        fn #bevy_register(app: &mut bevy::prelude::App) {
            app.register_type::<#ident>();
        }

        #vis const #descriptor: ::blueexposure::BlueEnumDescriptor =
            ::blueexposure::BlueEnumDescriptor::new(
                #module,
                #name,
                #doc,
                &[#(#variants),*],
                #py_register,
                #bevy_register,
            );
    }
    .into()
}

#[proc_macro_attribute]
pub fn blue_methods(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(item as ItemImpl);

    for impl_item in &mut item.items {
        let ImplItem::Fn(function) = impl_item else {
            continue;
        };
        let mut retained = Vec::with_capacity(function.attrs.len());
        let mut generated = Vec::<Attribute>::new();

        for attr in function.attrs.drain(..) {
            if !attr.path().is_ident("blue") {
                retained.push(attr);
                continue;
            }

            let result = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("new") {
                    generated.push(parse_quote!(#[new]));
                } else if meta.path.is_ident("getter") {
                    generated.push(parse_quote!(#[getter]));
                } else if meta.path.is_ident("setter") {
                    generated.push(parse_quote!(#[setter]));
                } else if meta.path.is_ident("classattr") {
                    generated.push(parse_quote!(#[classattr]));
                } else if meta.path.is_ident("static") {
                    generated.push(parse_quote!(#[staticmethod]));
                } else if meta.path.is_ident("name") {
                    let name = meta.value()?.parse::<LitStr>()?;
                    generated.push(parse_quote!(#[pyo3(name = #name)]));
                }
                Ok(())
            });
            if let Err(error) = result {
                return error.into_compile_error().into();
            }
        }

        retained.extend(generated);
        if let Some(signature) = inferred_optional_signature(&function.sig, &retained) {
            retained.push(signature);
        }
        function.attrs = retained;
    }

    quote! {
        #[pyo3::pymethods]
        #item
    }
    .into()
}

#[proc_macro_attribute]
pub fn blue_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(item as ItemFn);
    if function.sig.asyncness.is_some() {
        return syn::Error::new_spanned(&function.sig, "#[blue_test] does not support async tests")
            .into_compile_error()
            .into();
    }
    if !function.sig.inputs.is_empty() {
        return syn::Error::new_spanned(&function.sig, "#[blue_test] tests must take no arguments")
            .into_compile_error()
            .into();
    }
    if !matches!(function.sig.output, ReturnType::Default) {
        return syn::Error::new_spanned(&function.sig, "#[blue_test] tests must return ()")
            .into_compile_error()
            .into();
    }

    function.attrs.retain(|attr| !attr.path().is_ident("test"));

    let original_name = function.sig.ident.clone();
    let body = function.block;
    // The semantic body is the single source of truth. Kani wrappers are emitted by
    // blue-codegen with Genco so verification scaffolding is generated, not hand maintained.
    let attrs = function.attrs;
    let vis = function.vis;
    let sig = function.sig;
    let body_name = format_ident!("__blue_test_body_{}", original_name);
    let mut test_sig = sig;
    test_sig.ident = original_name;

    quote! {
        #[cfg(any(test, kani))]
        #(#attrs)*
        pub(crate) fn #body_name() #body

        #[cfg(test)]
        #[test]
        #vis #test_sig {
            #body_name()
        }
    }
    .into()
}
