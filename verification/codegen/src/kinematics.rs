//! Source old/scratch pose ownership and phase adapters. Domain rules are
//! restricted typed IR; this pass maps existing owners and call sites.
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
fn functions(owner: &str) -> Result<String, Box<dyn Error>> {
    let r: Rules = serde_json::from_str(&fs::read_to_string(
        "verification/migration/kinematic_rules.json",
    )?)?;
    if r.schema != 1 || r.original_commit != "114e89fa584f0e1a81eb0a679aabf41af8870072" {
        return Err("unreviewed kinematic rules".into());
    }
    Ok(r.functions
        .iter()
        .filter(|f| f.owner.as_deref() == Some(owner))
        .map(|f| f.lower().map(|t| t.to_string()))
        .collect::<Result<Vec<_>, _>>()?
        .join("\n"))
}
pub fn emit() -> Result<String, Box<dyn Error>> {
    let functions = functions("Pose")?;
    // Same generic lossless binary64 representation as the existing force and
    // angular state codecs; no kinematic evolution is hidden in this template.
    let output: rust::Tokens = quote! {
        #[derive(Debug,Clone,Copy,Default,serde::Serialize,serde::Deserialize)]
        #[serde(from="PoseBits",into="PoseBits")]
        pub struct Pose {pub position:[f64;3],pub velocity:[f64;3]}
        #[derive(serde::Serialize,serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct PoseBits{position:[u64;3],velocity:[u64;3]}
        impl From<Pose> for PoseBits{fn from(p:Pose)->Self{Self{position:p.position.map(f64::to_bits),velocity:p.velocity.map(f64::to_bits)}}}
        impl From<PoseBits> for Pose{fn from(p:PoseBits)->Self{Self{position:p.position.map(f64::from_bits),velocity:p.velocity.map(f64::from_bits)}}}
        $(functions)
    };
    Ok(output.to_file_string()?)
}
struct Pass<'a> {
    edit: super::partition::Rewrite<'a>,
    scope: String,
    in_publication: bool,
}
impl<'ast> Visit<'ast> for Pass<'_> {
    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        let old = std::mem::replace(&mut self.scope, f.sig.ident.to_string());
        if self.scope == "freeze_source_dead" {
            let mut sig = f.sig.clone();
            sig.inputs
                .push(syn::parse_quote!(plan:Option<&[PreparedMotion]>));
            self.edit
                .replace(&f.sig, quote::quote!(#sig), "projection-plan-parameter");
        }
        syn::visit::visit_impl_item_fn(self, f);
        self.scope = old;
    }
    fn visit_item_struct(&mut self, s: &'ast syn::ItemStruct) {
        if s.ident == "DestinyBallMetadata" || s.ident == "BallSnapshot" {
            let syn::Fields::Named(fields) = &s.fields else {
                unreachable!()
            };
            self.edit.insert(fields.brace_token.span.close().start(),quote::quote!(#[serde(default)] pub source_old_pose:Option<crate::generated_kinematics::Pose>,),"old-pose-owner-field");
        }
        syn::visit::visit_item_struct(self, s);
    }
    fn visit_expr_struct(&mut self, s: &'ast syn::ExprStruct) {
        if s.path
            .segments
            .last()
            .is_some_and(|n| n.ident == "DestinyBallMetadata" || n.ident == "BallSnapshot")
        {
            let value = if self.scope == "add_ball_with_identity" {
                quote::quote!(Some(crate::generated_kinematics::Pose::created(
                    position.to_array(),
                    velocity.to_array(),
                    self.app
                        .world()
                        .resource::<Time<Fixed>>()
                        .timestep()
                        .as_secs_f64()
                )))
            } else if self.scope == "ball_snapshot" {
                quote::quote!(metadata.source_old_pose)
            } else if self.scope == "dispatch_compat" {
                quote::quote!(Some(Default::default()))
            } else {
                panic!("unreviewed source pose initializer scope {}", self.scope)
            };
            let mut next = s.clone();
            next.fields.push(syn::parse_quote!(source_old_pose:#value));
            self.edit
                .replace(s, quote::quote!(#next), "old-pose-initialize");
        }
        if s.path.is_ident("OriginalParkBall") && self.scope == "original_motion_input" {
            let at = s
                .dot2_token
                .as_ref()
                .map(|t| t.span().start())
                .unwrap_or_else(|| s.brace_token.span.close().start());
            self.edit.insert(at,quote::quote!(old_position:self.source_old_pose(id)?.position_vector(),old_velocity:self.source_old_pose(id)?.velocity_vector(),),"old-pose-projection");
        }
        syn::visit::visit_expr_struct(self, s);
    }
    fn visit_expr_for_loop(&mut self, e: &'ast syn::ExprForLoop) {
        let old = self.in_publication;
        if self.scope == "advance_one_tick_with_motion"
            && e.pat
                .to_token_stream()
                .to_string()
                .starts_with("& PreparedMotion")
        {
            self.in_publication = true;
        }
        syn::visit::visit_expr_for_loop(self, e);
        self.in_publication = old;
    }
    fn visit_expr_if(&mut self, e: &'ast syn::ExprIf) {
        if self.scope == "advance_one_tick_with_motion"
            && e.cond.to_token_stream().to_string() == "should_step"
            && e.then_branch.stmts.first().is_some_and(|s| {
                s.to_token_stream().to_string() == "self . configure_destiny_space_damping () ? ;"
            })
        {
            self.edit.replace(
                &e.cond,
                quote::quote!(should_step && iterative.is_none()),
                "source-skip-damping-preparation",
            );
        }
        if self.in_publication
            && e.cond
                .to_token_stream()
                .to_string()
                .starts_with("! self . source_cached_dead")
        {
            let old = &e.then_branch;
            self.edit.replace(
                old,
                quote::quote!({if iterative.is_some(){self.publish_source_pose(id)?;}else #old}),
                "source-pose-publication",
            );
            return;
        }
        syn::visit::visit_expr_if(self, e);
    }
    fn visit_stmt(&mut self, s: &'ast syn::Stmt) {
        let text = s.to_token_stream().to_string();
        if self.scope == "freeze_source_dead" && text.starts_with("let entities =") {
            self.edit.replace(
                s,
                quote::quote!(let entities=self.source_projection_entities(plan);),
                "source-motion-projection",
            );
            return;
        }
        if self.scope == "advance_one_tick_with_motion" && text == "self . freeze_source_dead () ;"
        {
            self.edit.replace(s,quote::quote!(self.stage_source_motion(iterative.as_deref())?;self.freeze_source_dead(iterative.as_deref());),"source-motion-staging");
            return;
        }
        if self.scope == "set_position" && text == "self . set_position_value (id , position) ? ;" {
            self.edit.insert(
                s.span().end(),
                quote::quote!(self.write_source_old_position(id,position)?;),
                "command-both-position",
            );
        }
        if self.scope == "set_velocity"
            && text == "self . set_velocity_value (ball_id , velocity) ? ;"
        {
            self.edit.insert(
                s.span().end(),
                quote::quote!(self.write_source_old_velocity(ball_id,velocity)?;),
                "command-both-velocity",
            );
        }
        if text == "metadata . source_contacts = ball . source_contacts . clone () ;" {
            self.edit.insert(
                s.span().end(),
                quote::quote!(metadata.source_old_pose=ball.source_old_pose;),
                "restore-old-pose",
            );
        }
        syn::visit::visit_stmt(self, s);
    }
    fn visit_expr_method_call(&mut self, e: &'ast syn::ExprMethodCall) {
        if self.scope == "advance_one_tick_with_motion"
            && e.method == "correct_collision_free_analytic_motion"
        {
            self.edit.replace(e,quote::quote!((if iterative.is_some(){Ok(())}else{self.capture_source_previous(checkpoint)?;#e})),"source-single-integrator");
            return;
        }
        syn::visit::visit_expr_method_call(self, e);
    }
}
pub fn transform(source: &str) -> Result<String, Box<dyn Error>> {
    let file = syn::parse_file(source)?;
    let mut p = Pass {
        edit: super::partition::pass(source),
        scope: String::new(),
        in_publication: false,
    };
    p.visit_file(&file);
    let mut out = p.edit.finish(&[
        ("projection-plan-parameter", 1),
        ("source-motion-projection", 1),
        ("old-pose-owner-field", 2),
        ("old-pose-initialize", 3),
        ("old-pose-projection", 1),
        ("source-pose-publication", 1),
        ("source-motion-staging", 1),
        ("command-both-position", 1),
        ("command-both-velocity", 1),
        ("restore-old-pose", 1),
        ("source-single-integrator", 1),
        ("source-skip-damping-preparation", 1),
    ])?;
    let file = syn::parse_file(&out)?;
    let test = file
        .items
        .iter()
        .find(|i| matches!(i,syn::Item::Mod(m) if m.ident=="tests"))
        .ok_or("runtime test boundary missing")?;
    let at = super::offset(&out, test.span().start());
    out.insert_str(at, &format!("{}\n", functions("CompatRuntime")?));
    out.insert_str(at, "#[cfg(test)]\n#[path=\"../tests/support/kinematic_phases.rs\"]\nmod source_kinematic_tests;\n");
    syn::parse_file(&out)?;
    Ok(out)
}

/// Deliberate verification mutant: allow Avian to advance source-planned
/// bodies. Only a disposable output is consumed; live generation never calls it.
pub fn mutate_projection(source: &str) -> Result<String, Box<dyn Error>> {
    let file = syn::parse_file(source)?;
    let mut edit = super::partition::pass(source);
    for item in &file.items {
        if let syn::Item::Impl(owner) = item {
            for item in &owner.items {
                if let syn::ImplItem::Fn(f) = item
                    && f.sig.ident == "source_projection_entities"
                {
                    let mut next = f.clone();
                    for input in &mut next.sig.inputs {
                        if let syn::FnArg::Typed(input) = input
                            && let syn::Pat::Ident(name) = &mut *input.pat
                            && name.ident == "plan"
                        {
                            name.ident = syn::parse_quote!(_plan);
                        }
                    }
                    let before = next.block.stmts.len();
                    next.block
                        .stmts
                        .retain(|s| !matches!(s, syn::Stmt::Expr(syn::Expr::If(_), _)));
                    if before - next.block.stmts.len() != 1 {
                        return Err("projection mutant lost its exact branch".into());
                    }
                    edit.replace(f, quote::quote!(#next), "projection-mutation");
                }
            }
        }
    }
    edit.finish(&[("projection-mutation", 1)])
}

/// Deliberate verification mutant: retain f64 instead of source AsVector3's
/// f32 narrowing. The three actual generated cast sites must be identified.
pub fn mutate_acceleration(source: &str) -> Result<String, Box<dyn Error>> {
    struct Casts<'a> {
        edit: super::partition::Rewrite<'a>,
        selected: bool,
    }
    impl<'ast> Visit<'ast> for Casts<'_> {
        fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
            let old = self.selected;
            self.selected = f.sig.ident == "acceleration";
            syn::visit::visit_impl_item_fn(self, f);
            self.selected = old;
        }
        fn visit_expr_cast(&mut self, e: &'ast syn::ExprCast) {
            fn bare(mut e: &syn::Expr) -> &syn::Expr {
                while let syn::Expr::Paren(p) = e {
                    e = &p.expr;
                }
                e
            }
            if self.selected
                && matches!(&*e.ty,syn::Type::Path(t) if t.path.is_ident("f64"))
                && let syn::Expr::Cast(inner) = bare(&e.expr)
                && matches!(&*inner.ty,syn::Type::Path(t) if t.path.is_ident("f32"))
            {
                let value = &inner.expr;
                self.edit
                    .replace(e, quote::quote!(#value), "acceleration-mutation");
                return;
            }
            syn::visit::visit_expr_cast(self, e);
        }
    }
    let file = syn::parse_file(source)?;
    let mut p = Casts {
        edit: super::partition::pass(source),
        selected: false,
    };
    p.visit_file(&file);
    p.edit.finish(&[("acceleration-mutation", 3)])
}
