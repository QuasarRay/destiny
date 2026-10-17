//! Source force ownership through existing typed rules and span transforms.
use genco::prelude::*;
use quote::ToTokens;
use serde::Deserialize;
use std::{error::Error, fs};
use syn::{spanned::Spanned, visit::Visit};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rules {
    schema: u32,
    original_commit: String,
    functions: Vec<super::state_rules::Function>,
}
fn rules() -> Result<Rules, Box<dyn Error>> {
    let r: Rules = serde_json::from_str(&fs::read_to_string(
        "verification/migration/force_rules.json",
    )?)?;
    if r.schema != 1 || r.original_commit != "114e89fa584f0e1a81eb0a679aabf41af8870072" {
        return Err("unreviewed force rules".into());
    }
    Ok(r)
}
pub fn emit() -> Result<String, Box<dyn Error>> {
    let functions = rules()?
        .functions
        .iter()
        .filter(|f| f.owner.as_deref() == Some("Forces"))
        .map(|f| f.lower().map(|t| t.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    // Generic lossless representation conversion, matching the existing angular
    // state codec. These adapters contain no physics or force evolution rule.
    let output: rust::Tokens = quote! {
     #[derive(Debug,Clone,Copy,Default,serde::Serialize,serde::Deserialize)]
     #[serde(from="ForceBits",into="ForceBits")]
     pub struct Forces {pub last_g:[f64;3],pub last_c:[f64;3]}
     #[derive(serde::Serialize,serde::Deserialize)]
     #[serde(deny_unknown_fields)]
     struct ForceBits {last_g:[u64;3],last_c:[u64;3]}
     impl From<Forces> for ForceBits {fn from(value:Forces)->Self {Self {last_g:value.last_g.map(f64::to_bits),last_c:value.last_c.map(f64::to_bits)}}}
     impl From<ForceBits> for Forces {fn from(value:ForceBits)->Self {Self {last_g:value.last_g.map(f64::from_bits),last_c:value.last_c.map(f64::from_bits)}}}
     $(for function in functions => $(function))
    };
    Ok(output.to_file_string()?)
}
fn mutable_receiver(sig: &syn::Signature) -> Result<syn::Signature, Box<dyn Error>> {
    let mut next = sig.clone();
    let Some(syn::FnArg::Receiver(receiver)) = next.inputs.first_mut() else {
        return Err("missing collision owner receiver".into());
    };
    if receiver.reference.is_none()
        || receiver.colon_token.is_some()
        || receiver.mutability.is_some()
    {
        return Err("unreviewed owner receiver".into());
    }
    *receiver = syn::parse_quote!(&mut self);
    Ok(next)
}
fn state_initializer(source: &syn::ExprStruct) -> Result<syn::ExprStruct, Box<dyn Error>> {
    let angle = source
        .fields
        .iter()
        .find(|f| f.member.to_token_stream().to_string() == "source_angles")
        .ok_or("missing source state mapping")?;
    let value = match &angle.expr {
        syn::Expr::Field(e) => {
            let mut e = e.clone();
            e.member = syn::parse_quote!(source_forces);
            quote::quote!(#e)
        }
        syn::Expr::Call(e) if e.to_token_stream().to_string() == "Default :: default ()" => {
            quote::quote!(Default::default())
        }
        _ => return Err("unreviewed state initializer".into()),
    };
    let mut next = source.clone();
    next.fields.push(syn::parse_quote!(source_forces:#value));
    Ok(next)
}

struct Pass<'a> {
    edit: super::partition::Rewrite<'a>,
    scope: String,
    stage: &'static str,
}
impl<'ast> Visit<'ast> for Pass<'_> {
    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        let old = std::mem::replace(&mut self.scope, f.sig.ident.to_string());
        if self.stage == "runtime" && self.scope == "prepare_iterative_sphere_tick" {
            let signature = mutable_receiver(&f.sig).expect("known collision owner receiver");
            self.edit
                .replace(&f.sig, quote::quote!(#signature), "mutable-collision-owner");
        }
        syn::visit::visit_impl_item_fn(self, f);
        self.scope = old;
    }
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        let old = std::mem::replace(&mut self.scope, f.sig.ident.to_string());
        if self.stage == "gradient" && self.scope == "resolve" {
            let mut sig = f.sig.clone();
            let syn::FnArg::Typed(arg) = &mut sig.inputs[0] else {
                unreachable!()
            };
            let syn::Type::Reference(ty) = &mut *arg.ty else {
                unreachable!()
            };
            ty.mutability = Some(Default::default());
            self.edit
                .replace(&f.sig, quote::quote!(#sig), "gradient-mutable-input");
        }
        syn::visit::visit_item_fn(self, f);
        self.scope = old;
    }
    fn visit_item_struct(&mut self, s: &'ast syn::ItemStruct) {
        if self.stage == "runtime"
            && ["DestinyBallMetadata", "BallSnapshot"]
                .iter()
                .any(|n| s.ident == *n)
        {
            let syn::Fields::Named(fields) = &s.fields else {
                unreachable!()
            };
            self.edit.insert(
                fields.brace_token.span.close().start(),
                quote::quote!(#[serde(default)] pub source_forces:crate::generated_forces::Forces,),
                "force-owner-field",
            );
        }
        if self.stage == "runtime" && s.ident == "PreparedMotion" {
            let syn::Fields::Named(mut fields) = s.fields.clone() else {
                unreachable!()
            };
            fields.named = fields
                .named
                .into_iter()
                .filter(|f| f.ident.as_ref().is_none_or(|n| n != "acceleration"))
                .collect();
            self.edit
                .replace(&s.fields, quote::quote!(#fields), "remove-force-plan-field");
        }
        syn::visit::visit_item_struct(self, s);
    }
    fn visit_expr_struct(&mut self, s: &'ast syn::ExprStruct) {
        if self.stage == "runtime"
            && s.path.to_token_stream().to_string() == "crate :: iterative_collision :: Body"
        {
            for f in &s.fields {
                match f.member.to_token_stream().to_string().as_str() {
                    "acceleration" => self.edit.replace(
                        &f.expr,
                        quote::quote!(ball.last_g),
                        "actual-controller-input",
                    ),
                    "collision_acceleration" => self.edit.replace(
                        &f.expr,
                        quote::quote!(ball.last_c),
                        "actual-collision-input",
                    ),
                    _ => (),
                }
            }
        }

        if self.stage == "runtime"
            && s.path
                .segments
                .last()
                .is_some_and(|n| n.ident == "DestinyBallMetadata" || n.ident == "BallSnapshot")
        {
            let next = state_initializer(s).expect("known state initializer");
            self.edit
                .replace(s, quote::quote!(#next), "force-owner-initialize");
        }
        if self.stage == "runtime"
            && s.path.is_ident("OriginalParkBall")
            && self.scope == "original_motion_input"
        {
            let at = s
                .dot2_token
                .as_ref()
                .map(|t| t.span().start())
                .unwrap_or_else(|| s.brace_token.span.close().start());
            self.edit.insert(at,quote::quote!(last_g:metadata.source_forces.controller(),last_c:metadata.source_forces.collision(),),"original-force-projection");
        }
        if self.stage == "runtime" && s.path.is_ident("PreparedMotion") {
            let mut next = s.clone();
            next.fields = next
                .fields
                .into_iter()
                .filter(|f| f.member.to_token_stream().to_string() != "acceleration")
                .collect();
            self.edit
                .replace(s, quote::quote!(#next), "remove-plan-force-initialize");
        }
        syn::visit::visit_expr_struct(self, s);
    }
    fn visit_stmt(&mut self, s: &'ast syn::Stmt) {
        let text = s.to_token_stream().to_string();
        if self.stage == "runtime"
            && self.scope == "prepare_motion_tick"
            && text.starts_with("let mut control =")
        {
            self.edit.insert(
                s.span().start(),
                quote::quote!(self.reset_source_behavior_state(id)?;),
                "live-force-reset",
            );
        }
        if self.stage == "runtime"
            && self.scope == "prepare_motion_tick"
            && text.starts_with("results . push (PreparedMotion")
        {
            self.edit.insert(
                s.span().start(),
                quote::quote!(self.store_source_controller_force(id,acceleration)?;),
                "controller-force-store",
            );
        }
        if self.stage == "runtime"
            && self.scope == "prepare_iterative_sphere_tick"
            && text.starts_with("let step =")
        {
            self.edit
                .replace(s, quote::quote!(), "remove-plan-force-binding");
        }
        if self.stage == "runtime"
            && self.scope == "prepare_iterative_sphere_tick"
            && text.starts_with("let results =")
        {
            self.edit.insert(
                s.span().end(),
                quote::quote!(self.store_source_collision_forces(&bodies)?;),
                "collision-force-store",
            );
        }
        if self.stage == "runtime" && text == "metadata . source_angles = ball . source_angles ;" {
            self.edit.insert(
                s.span().end(),
                quote::quote!(metadata.source_forces=ball.source_forces;),
                "restore-force-owner",
            );
        }
        if self.stage == "gradient"
            && self.scope == "resolve"
            && text.starts_with("publication . push (Some")
        {
            self.edit.insert(
                s.span().start(),
                quote::quote!(bodies[body_index].collision_acceleration=me.last_c;),
                "gradient-force-output",
            );
        }
        syn::visit::visit_stmt(self, s);
    }
    fn visit_expr_call(&mut self, e: &'ast syn::ExprCall) {
        if (self.stage == "runtime" || self.stage == "fixture")
            && e.func.to_token_stream().to_string() == "crate :: noniterative_collision :: resolve"
            || self.stage == "fixture"
                && e.func.to_token_stream().to_string() == "noniterative_collision :: resolve"
        {
            let syn::Expr::Reference(mut reference) = e.args[0].clone() else {
                unreachable!()
            };
            reference.mutability = Some(Default::default());
            self.edit.replace(
                &e.args[0],
                quote::quote!(#reference),
                "gradient-mutable-call",
            );
        }
        syn::visit::visit_expr_call(self, e);
    }
    fn visit_expr_for_loop(&mut self, e: &'ast syn::ExprForLoop) {
        let pat = e.pat.to_token_stream().to_string();
        let expr = e.expr.to_token_stream().to_string();
        if self.stage == "gradient" && pat == "& body" && expr == "bodies" {
            self.edit
                .replace(&*e.pat, quote::quote!(body_index), "gradient-body-index");
            self.edit.replace(
                &*e.expr,
                quote::quote!(0..bodies.len()),
                "gradient-body-range",
            );
            self.edit.insert(
                e.body.brace_token.span.open().end(),
                quote::quote!(let body=bodies[body_index];),
                "gradient-body-copy",
            );
        }
        if self.stage == "gradient" && pat == "& other" && expr == "bodies" {
            self.edit.replace(
                &*e.expr,
                quote::quote!(bodies.iter()),
                "gradient-neighbor-read",
            );
        }
        if self.stage == "fixture" && pat == "body" && expr == "& mut bodies" {
            self.edit.insert(
                e.body.brace_token.span.open().end(),
                quote::quote!(body.collision_acceleration=Vec3::ZERO;),
                "fixture-source-reset",
            );
        }
        syn::visit::visit_expr_for_loop(self, e);
    }
}
pub fn transform(source: &str, stage: &'static str) -> Result<String, Box<dyn Error>> {
    let file = syn::parse_file(source)?;
    let mut p = Pass {
        edit: super::partition::pass(source),
        scope: String::new(),
        stage,
    };
    p.visit_file(&file);
    let expected = match stage {
        "runtime" => vec![
            ("mutable-collision-owner", 1),
            ("force-owner-field", 2),
            ("remove-force-plan-field", 1),
            ("force-owner-initialize", 3),
            ("original-force-projection", 1),
            ("remove-plan-force-initialize", 2),
            ("actual-controller-input", 1),
            ("actual-collision-input", 1),
            ("live-force-reset", 1),
            ("controller-force-store", 1),
            ("remove-plan-force-binding", 1),
            ("collision-force-store", 1),
            ("restore-force-owner", 1),
            ("gradient-mutable-call", 1),
        ],
        "gradient" => vec![
            ("gradient-mutable-input", 1),
            ("gradient-body-index", 1),
            ("gradient-body-range", 1),
            ("gradient-body-copy", 1),
            ("gradient-neighbor-read", 1),
            ("gradient-force-output", 1),
        ],
        "fixture" => vec![("gradient-mutable-call", 1), ("fixture-source-reset", 1)],
        _ => return Err("unknown force stage".into()),
    };
    let mut out = p.edit.finish(&expected)?;
    if stage == "runtime" {
        let file = syn::parse_file(&out)?;
        let tests = file
            .items
            .iter()
            .find(|item| matches!(item,syn::Item::Mod(m) if m.ident=="tests"))
            .ok_or("runtime test boundary missing")?;
        let at = super::offset(&out, tests.span().start());
        let functions = rules()?
            .functions
            .iter()
            .filter(|f| f.owner.as_deref() == Some("CompatRuntime"))
            .map(|f| f.lower().map(|f| f.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        out.insert_str(at, &format!("{}\n", functions.join("\n")));
    }
    syn::parse_file(&out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owner_initializers_preserve_field_structure_without_trailing_commas() {
        for source in [
            "BallSnapshot {source_angles:Default::default()}",
            "BallSnapshot {source_angles:metadata.source_angles,}",
        ] {
            let input: syn::ExprStruct = syn::parse_str(source).unwrap();
            let out = state_initializer(&input).unwrap();
            let parsed: syn::ExprStruct = syn::parse2(quote::quote!(#out)).unwrap();
            assert_eq!(parsed.fields.len(), 2);
        }
        let unsupported: syn::ExprStruct = syn::parse_quote!(BallSnapshot {
            source_angles: other()
        });
        assert!(state_initializer(&unsupported).is_err());
    }
    #[test]
    fn mutable_owner_receiver_keeps_syn_receiver_type_consistent() {
        let sig: syn::Signature = syn::parse_quote!(fn step(&self,value:i64));
        let next = mutable_receiver(&sig).unwrap();
        let parsed: syn::Signature = syn::parse2(quote::quote!(#next)).unwrap();
        let syn::FnArg::Receiver(r) = &parsed.inputs[0] else {
            panic!("receiver missing")
        };
        assert!(r.mutability.is_some());
        assert!(r.colon_token.is_none());
        let static_sig: syn::Signature = syn::parse_quote!(fn step(value:i64));
        assert!(mutable_receiver(&static_sig).is_err());
    }
}
