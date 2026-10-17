//! Contact-state representation adapters and guarded call-site transformations.
//! Executable source rules are typed data, lowered by the shared state IR.
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
fn functions(stage: &str) -> Result<String, Box<dyn Error>> {
    let r: Rules = serde_json::from_str(&fs::read_to_string(
        "verification/migration/contact_rules.json",
    )?)?;
    if r.schema != 1 || r.original_commit != "114e89fa584f0e1a81eb0a679aabf41af8870072" {
        return Err("unreviewed contact rules".into());
    }
    let owner = match stage {
        "runtime" => Some("CompatRuntime"),
        "frame" => Some("Frame"),
        "pass" => Some("CollisionPass"),
        _ => None,
    };
    let selected = match stage {
        "iterative" => "source_location_contacts",
        "gradient" => "append_source_contacts",
        _ => "",
    };
    Ok(r.functions
        .iter()
        .filter(|f| f.owner.as_deref() == owner && (owner.is_some() || f.name == selected))
        .map(|f| f.lower().map(|f| f.to_string()))
        .collect::<Result<Vec<_>, _>>()?
        .join("\n"))
}
pub fn emit() -> Result<String, Box<dyn Error>> {
    let functions = functions("pass")?;
    // Typed environmental messages and operation-owned cursor representation.
    // Execution rules and all domain checks are in the restricted semantic IR.
    let output: rust::Tokens = quote! {
        #[derive(Debug,Clone,Copy)]
        pub struct CollisionEvent {pub ball_id:i64,pub other_id:i64,pub acceleration:[f64;3]}
        #[derive(Debug,Clone,Copy)]
        pub enum CollisionNotice {Contact(CollisionEvent),HostError{ball_id:i64}}
        #[derive(Debug)]
        pub struct CollisionPass {pub(crate) world:bevy::ecs::world::WorldId,pub(crate) entity:bevy::prelude::Entity,pub(crate) owner:i64,pub(crate) generation:u64,pub(crate) limit:usize,pub(crate) index:usize,pub(crate) failures:usize}
        $(functions)
    };
    Ok(output.to_file_string()?)
}
struct Pass<'a> {
    edit: super::partition::Rewrite<'a>,
    scope: String,
    stage: &'static str,
}
impl<'ast> Visit<'ast> for Pass<'_> {
    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        let old = std::mem::replace(&mut self.scope, f.sig.ident.to_string());
        syn::visit::visit_impl_item_fn(self, f);
        self.scope = old;
    }
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        let old = std::mem::replace(&mut self.scope, f.sig.ident.to_string());
        if ["iterative", "gradient"].contains(&self.stage) && self.scope == "resolve" {
            let mut sig = f.sig.clone();
            sig.inputs.push(syn::parse_quote!(source_contact_effects:&mut crate::iterative_collision::ContactTrace));
            self.edit
                .replace(&f.sig, quote::quote!(#sig), "contact-output-parameter");
            if self.stage == "iterative" {
                let at = f
                    .block
                    .stmts
                    .iter()
                    .find(|s| {
                        s.to_token_stream()
                            .to_string()
                            .starts_with("# [cfg (kani)]")
                    })
                    .expect("source cover boundary");
                self.edit.insert(
                    at.span().start(),
                    quote::quote!(for index in 0..solver.bodies.len() {
                        source_contact_effects.push((
                            solver.bodies[index].id,
                            source_location_contacts(
                                solver.properties[index].as_ref(),
                                solver.statics,
                            ),
                        ));
                    }),
                    "location-output-transport",
                );
            }
        }
        syn::visit::visit_item_fn(self, f);
        self.scope = old;
    }
    fn visit_item_struct(&mut self, s: &'ast syn::ItemStruct) {
        let field = if self.stage == "runtime"
            && (s.ident == "DestinyBallMetadata" || s.ident == "BallSnapshot")
        {
            Some((
                quote::quote!(#[serde(default)] pub source_contacts:Vec<i64>,),
                "contact-owner-field",
            ))
        } else if self.stage == "frame" && s.ident == "Frame" {
            Some((
                quote::quote!(pub contact_locations:crate::iterative_collision::ContactTrace,),
                "location-projection-field",
            ))
        } else {
            None
        };
        if let Some((tokens, label)) = field {
            let syn::Fields::Named(fields) = &s.fields else {
                unreachable!()
            };
            self.edit
                .insert(fields.brace_token.span.close().start(), tokens, label);
        }
        syn::visit::visit_item_struct(self, s);
    }
    fn visit_expr_struct(&mut self, s: &'ast syn::ExprStruct) {
        if self.stage == "runtime"
            && s.path
                .segments
                .last()
                .is_some_and(|n| n.ident == "DestinyBallMetadata" || n.ident == "BallSnapshot")
        {
            let force = s
                .fields
                .iter()
                .find(|f| f.member.to_token_stream().to_string() == "source_forces")
                .expect("existing source state");
            let value = match &force.expr {
                syn::Expr::Field(f) => {
                    let mut e = f.clone();
                    e.member = syn::parse_quote!(source_contacts);
                    quote::quote!(#e.clone())
                }
                syn::Expr::Call(f)
                    if f.to_token_stream().to_string() == "Default :: default ()" =>
                {
                    quote::quote!(Default::default())
                }
                _ => panic!("unreviewed contact initializer"),
            };
            let mut next = s.clone();
            next.fields.push(syn::parse_quote!(source_contacts:#value));
            self.edit
                .replace(s, quote::quote!(#next), "contact-owner-initialize");
        }
        syn::visit::visit_expr_struct(self, s);
    }
    fn visit_stmt(&mut self, s: &'ast syn::Stmt) {
        let text = s.to_token_stream().to_string();
        if self.stage == "runtime" && self.scope == "prepare_iterative_sphere_tick" {
            if text.starts_with("let results =") {
                self.edit.insert(
                    s.span().start(),
                    quote::quote!(let mut source_contact_effects=Vec::new();),
                    "contact-output-binding",
                );
            }
            if text == "self . store_source_collision_forces (& bodies) ? ;" {
                self.edit.insert(
                    s.span().end(),
                    quote::quote!(self.store_source_contacts(source_contact_effects)?;),
                    "contact-output-store",
                );
            }
        }
        if self.stage == "runtime" && text == "metadata . source_forces = ball . source_forces ;" {
            self.edit.insert(
                s.span().end(),
                quote::quote!(metadata.source_contacts=ball.source_contacts.clone();),
                "contact-owner-restore",
            );
        }
        if self.stage == "runtime"
            && self.scope == "update_source_angles"
            && text == "let entity = self . entity (id) ? ;"
        {
            self.edit
                .replace(s, quote::quote!(), "remove-avian-contact-binding");
        }
        if self.stage == "gradient" && self.scope == "resolve" {
            if text.starts_with("let mut me =") {
                self.edit.insert(
                    s.span().end(),
                    quote::quote!(let mut source_contacts=Vec::new();),
                    "gradient-contact-binding",
                );
            }
            if text.starts_with("sphere_potential_with_environment (")
                || text.starts_with("match geometry . shape")
            {
                self.edit.insert(
                    s.span().end(),
                    quote::quote!(append_source_contacts(&mut source_contacts,&me);),
                    "gradient-contact-collect",
                );
            }
            if text.starts_with("bodies [body_index] . collision_acceleration =") {
                self.edit.insert(
                    s.span().start(),
                    quote::quote!(source_contact_effects.push((body.id,source_contacts));),
                    "gradient-contact-output",
                );
            }
        }
        syn::visit::visit_stmt(self, s);
    }
    fn visit_field_value(&mut self, f: &'ast syn::FieldValue) {
        if self.stage == "runtime"
            && self.scope == "update_source_angles"
            && f.member.to_token_stream().to_string() == "has_collisions"
        {
            self.edit.replace(
                &f.expr,
                quote::quote!(!self.metadata(id)?.source_contacts.is_empty()),
                "source-angle-contact-input",
            );
        }
        syn::visit::visit_field_value(self, f);
    }
    fn visit_expr_call(&mut self, e: &'ast syn::ExprCall) {
        let target = e.func.to_token_stream().to_string();
        if ["runtime", "iterative-fixture", "gradient-fixture"].contains(&self.stage)
            && [
                "crate :: iterative_collision :: resolve",
                "crate :: noniterative_collision :: resolve",
                "iterative_collision :: resolve",
                "noniterative_collision :: resolve",
                "production :: resolve",
            ]
            .contains(&target.as_str())
        {
            let mut next = e.clone();
            next.args.push(if self.stage == "runtime" {
                syn::parse_quote!(&mut source_contact_effects)
            } else {
                syn::parse_quote!(&mut Vec::new())
            });
            self.edit
                .replace(e, quote::quote!(#next), "contact-output-call");
        }
        syn::visit::visit_expr_call(self, e);
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
            ("contact-owner-field", 2),
            ("contact-owner-initialize", 3),
            ("contact-owner-restore", 1),
            ("remove-avian-contact-binding", 1),
            ("source-angle-contact-input", 1),
            ("contact-output-binding", 1),
            ("contact-output-store", 1),
            ("contact-output-call", 2),
        ],
        "gradient" => vec![
            ("contact-output-parameter", 1),
            ("gradient-contact-binding", 1),
            ("gradient-contact-collect", 2),
            ("gradient-contact-output", 1),
        ],
        "iterative" => vec![
            ("contact-output-parameter", 1),
            ("location-output-transport", 1),
        ],
        "frame" => vec![("location-projection-field", 1)],
        "iterative-fixture" => vec![("contact-output-call", 3)],
        "gradient-fixture" => vec![("contact-output-call", 1)],
        _ => return Err("unknown contact stage".into()),
    };
    let mut out = p.edit.finish(&expected)?;
    if !stage.ends_with("fixture") {
        let file = syn::parse_file(&out)?;
        let at = file
            .items
            .iter()
            .find(|i| matches!(i,syn::Item::Mod(m) if m.ident=="tests"))
            .map(|i| super::offset(&out, i.span().start()))
            .unwrap_or(out.len());
        let declarations = if stage == "iterative" {
            "pub(crate) type ContactTrace=Vec<(i64,Vec<i64>)>;\n"
        } else {
            ""
        };
        out.insert_str(at, &format!("\n{declarations}{}\n", functions(stage)?));
    }
    syn::parse_file(&out)?;
    Ok(out)
}
