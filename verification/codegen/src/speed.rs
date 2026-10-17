//! Preserve source maximum velocity as a thrust parameter, without Avian caps.
//! Only selected syntax spans change; surrounding source/comments are retained.
use quote::ToTokens;
use std::error::Error;
use syn::{Expr, Item, visit::Visit};

struct Pass<'a> {
    source: &'a str,
    scope: String,
    edits: Vec<(usize, usize, String)>,
    removed_caps: usize,
    network_imports: usize,
    damping_call: Option<String>,
    damping_destination: Option<usize>,
}

impl Pass<'_> {
    fn replace(&mut self, node: &impl syn::spanned::Spanned, text: String) {
        let span = node.span();
        self.edits.push((
            super::offset(self.source, span.start()),
            super::offset(self.source, span.end()),
            text,
        ));
    }

    fn damping_scope(&self) -> bool {
        [
            "configure_destiny_space_damping",
            "correct_collision_free_analytic_motion",
        ]
        .contains(&self.scope.as_str())
    }
}

impl<'ast> Visit<'ast> for Pass<'_> {
    fn visit_stmt(&mut self, statement: &'ast syn::Stmt) {
        if self.scope == "advance_one_tick_with_motion" {
            if matches!(statement,syn::Stmt::Local(local) if matches!(&local.pat,syn::Pat::Ident(name) if name.ident=="iterative"))
            {
                use syn::spanned::Spanned;
                assert!(self.damping_destination.is_none());
                self.damping_destination = Some(super::offset(self.source, statement.span().end()));
            }
            if statement.to_token_stream().to_string()
                == "self . configure_destiny_space_damping () ? ;"
            {
                assert!(self.damping_call.is_none());
                self.damping_call = Some(statement.to_token_stream().to_string());
                self.replace(statement, String::new());
                return;
            }
        }
        syn::visit::visit_stmt(self, statement);
    }
    fn visit_macro(&mut self, value: &'ast syn::Macro) {
        fn references(pass: &mut Pass<'_>, tokens: proc_macro2::TokenStream) {
            for token in tokens {
                match token {
                    proc_macro2::TokenTree::Ident(name) if name == "MaxLinearSpeed" => {
                        pass.replace(&name, "DestinyMaxVelocity".into());
                    }
                    proc_macro2::TokenTree::Group(group) => references(pass, group.stream()),
                    _ => (),
                }
            }
        }
        references(self, value.tokens.clone());
    }
    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        let prior = std::mem::replace(&mut self.scope, function.sig.ident.to_string());
        syn::visit::visit_impl_item_fn(self, function);
        self.scope = prior;
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        let prior = std::mem::replace(&mut self.scope, function.sig.ident.to_string());
        if self.scope == "speed_clamped_damped_motion_uses_the_clamped_initial_velocity" {
            self.replace(
                &function.sig.ident,
                "source_max_velocity_does_not_cap_stop_motion".into(),
            );
        }
        syn::visit::visit_item_fn(self, function);
        self.scope = prior;
    }

    fn visit_ident(&mut self, name: &'ast syn::Ident) {
        if name == "MaxLinearSpeed" {
            self.replace(name, "DestinyMaxVelocity".into());
        }
    }

    fn visit_lit_str(&mut self, value: &'ast syn::LitStr) {
        if value.value() == "missing MaxLinearSpeed" {
            self.replace(value, "\"missing DestinyMaxVelocity\"".into());
        }
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let syn::Pat::Ident(name) = &local.pat {
            if self.damping_scope() && (name.ident == "speed" || name.ident == "initial_speed") {
                self.replace(local, String::new());
                return;
            }
            if self.damping_scope()
                && (name.ident == "velocity" || name.ident == "initial_velocity")
                && let Some(token) = &name.mutability
            {
                self.replace(token, String::new());
            }
            if self.scope == "speed_clamped_damped_motion_uses_the_clamped_initial_velocity"
                && name.ident == "expected"
            {
                let value = &local
                    .init
                    .as_ref()
                    .expect("existing speed regression expectation")
                    .expr;
                // Test orchestration only: retain the old decay assertion and
                // tolerance, applying it to the original uncapped input (10).
                let replacement: Expr = syn::parse_quote!(10.0 * (#value));
                self.replace(&**value, replacement.to_token_stream().to_string());
            }
        }
        syn::visit::visit_local(self, local);
    }

    fn visit_expr_if(&mut self, branch: &'ast syn::ExprIf) {
        let condition = branch.cond.to_token_stream().to_string();
        if self.damping_scope()
            && (condition.starts_with("speed > max_speed . 0")
                || condition.starts_with("initial_speed > initial . max_linear_speed . 0"))
        {
            self.removed_caps += 1;
            self.replace(branch, String::new());
            return;
        }
        syn::visit::visit_expr_if(self, branch);
    }

    fn visit_type_tuple(&mut self, tuple: &'ast syn::TypeTuple) {
        if self.scope == "configure_destiny_space_damping"
            && tuple
                .elems
                .iter()
                .any(|field| field.to_token_stream().to_string() == "& MaxLinearSpeed")
        {
            let mut result = tuple.clone();
            result.elems = tuple
                .elems
                .iter()
                .filter(|field| field.to_token_stream().to_string() != "& MaxLinearSpeed")
                .cloned()
                .collect();
            self.replace(tuple, result.to_token_stream().to_string());
            return;
        }
        syn::visit::visit_type_tuple(self, tuple);
    }

    fn visit_pat_tuple(&mut self, tuple: &'ast syn::PatTuple) {
        if self.scope == "configure_destiny_space_damping"
            && tuple
                .elems
                .iter()
                .any(|field| field.to_token_stream().to_string() == "max_speed")
        {
            let mut result = tuple.clone();
            result.elems = tuple
                .elems
                .iter()
                .filter(|field| field.to_token_stream().to_string() != "max_speed")
                .cloned()
                .collect();
            self.replace(tuple, result.to_token_stream().to_string());
            return;
        }
        syn::visit::visit_pat_tuple(self, tuple);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        if item
            .to_token_stream()
            .to_string()
            .starts_with("use avian3d")
            && item
                .to_token_stream()
                .to_string()
                .contains("MaxLinearSpeed")
        {
            let mut result = item.clone();
            if let syn::UseTree::Path(root) = &mut result.tree
                && let syn::UseTree::Path(prelude) = &mut *root.tree
                && let syn::UseTree::Group(group) = &mut *prelude.tree
            {
                group.items=group.items.iter().filter(|tree| !matches!(tree,syn::UseTree::Name(name) if name.ident=="MaxLinearSpeed")).cloned().collect();
                self.network_imports += 1;
            }
            self.replace(
                item,
                format!(
                    "{}\nuse crate::DestinyMaxVelocity;",
                    result.to_token_stream()
                ),
            );
            return;
        }
        syn::visit::visit_item_use(self, item);
    }
}

fn run(source: &str, network: bool) -> Result<String, Box<dyn Error>> {
    let file = syn::parse_file(source)?;
    let mut pass = Pass {
        source,
        scope: String::new(),
        edits: Vec::new(),
        removed_caps: 0,
        network_imports: 0,
        damping_call: None,
        damping_destination: None,
    };
    pass.visit_file(&file);
    if network {
        if pass.network_imports != 1 || pass.removed_caps != 0 {
            return Err("network source speed registration changed".into());
        }
    } else {
        if pass.removed_caps != 2 || pass.network_imports != 0 {
            return Err("missing or ambiguous non-source velocity clamps".into());
        }
        let call: syn::Stmt = syn::parse_str(
            pass.damping_call
                .as_ref()
                .ok_or("missing Avian damping stage")?,
        )?;
        let block: syn::Stmt = syn::parse_quote!(if should_step { #call });
        let point = pass
            .damping_destination
            .ok_or("missing source collision preparation boundary")?;
        pass.edits
            .push((point, point, format!("\n{}", block.to_token_stream())));
        let mass = file
            .items
            .iter()
            .find_map(|item| match item {
                Item::Struct(value) if value.ident == "DestinyMass" => Some(value),
                _ => None,
            })
            .ok_or("missing scalar component declaration")?;
        let mut maximum = mass.clone();
        maximum.ident = syn::parse_quote!(DestinyMaxVelocity);
        maximum
            .attrs
            .retain(|attribute| !attribute.path().is_ident("doc"));
        maximum.attrs.push(syn::parse_quote!(#[doc="Destiny maximum-velocity parameter used by source thrust; this is not an Avian velocity cap. Existing API storage is f64; motion kernels narrow to original f32."]));
        use syn::spanned::Spanned;
        let point = super::offset(source, mass.span().end());
        pass.edits
            .push((point, point, format!("\n{}", maximum.to_token_stream())));
    }
    pass.edits.sort_by_key(|edit| (edit.0, edit.1));
    if pass.edits.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Err("overlapping speed transformations".into());
    }
    let mut output = source.to_owned();
    for (start, end, text) in pass.edits.into_iter().rev() {
        output.replace_range(start..end, &text);
    }
    syn::parse_file(&output)?;
    Ok(output)
}

pub fn runtime(source: &str) -> Result<String, Box<dyn Error>> {
    run(source, false)
}
pub fn network(source: &str) -> Result<String, Box<dyn Error>> {
    run(source, true)
}
