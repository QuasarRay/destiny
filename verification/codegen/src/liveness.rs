//! Source-backed liveness rules and narrow scheduling/data projections.
//! Reuses the existing typed rule lowerer and span editor.
use genco::prelude::*;
use quote::ToTokens;
use serde::Deserialize;
use std::{error::Error, fs};
use syn::{Item, spanned::Spanned, visit::Visit};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rules {
    schema: u32,
    original_commit: String,
    functions: Vec<super::state_rules::Function>,
}
fn rules() -> Result<Rules, Box<dyn Error>> {
    let value: Rules = serde_json::from_str(&fs::read_to_string(
        "verification/migration/liveness_rules.json",
    )?)?;
    if value.schema != 1 || value.original_commit != "114e89fa584f0e1a81eb0a679aabf41af8870072" {
        return Err("unreviewed liveness source/rules".into());
    }
    Ok(value)
}
pub fn emit() -> Result<String, Box<dyn Error>> {
    let functions = rules()?
        .functions
        .iter()
        .filter(|function| function.owner.is_none())
        .map(|function| function.lower().map(|value| value.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    // Generic ECS representation projection. Component replacement/restoration
    // knows no Destiny rule; the source-derived frame selects its entities.
    let output: rust::Tokens = quote! {
        use bevy::prelude::{Component,Entity,Resource,World};
        use avian3d::prelude::RigidBody;
        use std::collections::BTreeSet;
        #[derive(Debug,Clone,Default,serde::Serialize,serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct LivenessState {
            pub registered:BTreeSet<i64>,
            pub keep_alive:Vec<(i64,BTreeSet<i64>)>,
            pub interactives:Vec<(i64,i64)>,
        }
        #[derive(Resource,Default)]
        pub(crate) struct Frame {
            pub cached_dead:BTreeSet<Entity>,
            pub projected:Vec<(Entity,RigidBody)>,
        }
        pub(crate) fn project_components<T:Component+Copy>(world:&mut World,entities:&BTreeSet<Entity>,replacement:T)->Vec<(Entity,T)> {
            let mut saved=Vec::new();
            for entity in entities {
                if let Some(value)=world.get::<T>(*entity).copied() {
                    saved.push((*entity,value));
                    world.entity_mut(*entity).insert(replacement);
                }
            }
            saved
        }
        pub(crate) fn restore_components<T:Component>(world:&mut World,saved:Vec<(Entity,T)>) {
            for (entity,value) in saved {
                if let Ok(mut target)=world.get_entity_mut(entity) {target.insert(value);}
            }
        }
        $(for function in functions => $(function))
        #[cfg(test)]
        #[path = "../tests/support/liveness_tracking.rs"]
        mod tests;
    };
    Ok(output.to_file_string()?)
}

struct Pass<'a> {
    edit: super::partition::Rewrite<'a>,
    scope: String,
    stage: &'static str,
}
impl<'ast> Visit<'ast> for Pass<'_> {
    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        let old = std::mem::replace(&mut self.scope, function.sig.ident.to_string());
        if self.stage == "runtime" && self.scope == "configured_pair" {
            // The retained collision-hook test assigns labels directly. Supply
            // live source memberships so it still observes collision filtering.
            let result = function.block.stmts.last().expect("test returns runtime");
            self.edit.insert(
                result.span().start(),
                quote::quote!(
                    runtime.add_keep_alive_member(1, left_bubble);
                    runtime.add_keep_alive_member(2, right_bubble);
                ),
                "collision-test-live-membership",
            );
        }
        syn::visit::visit_item_fn(self, function);
        self.scope = old;
    }
    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        let old = std::mem::replace(&mut self.scope, function.sig.ident.to_string());
        if self.stage == "bubbles" && self.scope == "transition" {
            self.edit.insert(
                function.block.brace_token.span.close().start(),
                quote::quote!(self.update_keep_alive(ball.id,ball.old,ball.new);),
                "keeper-transition",
            );
        }
        if self.stage == "runtime" && self.scope == "prepare_motion_tick" {
            self.edit.insert(function.block.brace_token.span.open().end(),quote::quote!(self.app.world_mut().resource_mut::<crate::generated_liveness::Frame>().cached_dead.clear();),"frame-clear");
        }
        if self.stage == "runtime" && self.scope == "set_interactive" {
            self.edit.insert(
                function.block.brace_token.span.open().end(),
                quote::quote!(let was_interactive=self.metadata(id)?.is_interactive;),
                "interactive-prior-flag",
            );
        }
        syn::visit::visit_impl_item_fn(self, function);
        self.scope = old;
    }
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        let extra = if self.stage == "bubbles" && item.ident == "Tracking" {
            Some((
                quote::quote!(#[serde(default)] keep_alive_registry:std::collections::BTreeSet<i64>,#[serde(default)] keep_alive_groups:Vec<(i64,std::collections::BTreeSet<i64>)>,),
                "keeper-fields",
            ))
        } else if self.stage == "runtime" && item.ident == "NotificationPass" {
            Some((
                quote::quote!(clear_dead_members:bool,),
                "proximity-pass-field",
            ))
        } else if self.stage == "iterative" && item.ident == "Body" {
            Some((quote::quote!(pub cached_dead:bool,), "collision-dead-field"))
        } else {
            None
        };
        if let Some((extra, kind)) = extra {
            let syn::Fields::Named(fields) = &item.fields else {
                unreachable!()
            };
            assert!(fields.named.trailing_punct());
            self.edit
                .insert(fields.brace_token.span.close().start(), extra, kind);
        }
        syn::visit::visit_item_struct(self, item);
    }
    fn visit_expr_struct(&mut self, value: &'ast syn::ExprStruct) {
        if self.stage == "runtime" && value.path.is_ident("NotificationPass") {
            self.edit.insert(
                value.brace_token.span.close().start(),
                quote::quote!(clear_dead_members:false,),
                "proximity-pass-initialize",
            );
        }
        if self.stage == "runtime"
            && value.path.to_token_stream().to_string() == "crate :: iterative_collision :: Body"
        {
            self.edit.insert(
                value.brace_token.span.close().start(),
                quote::quote!(cached_dead:self.source_cached_dead(id),),
                "collision-frame-input",
            );
        }
        if self.stage == "iterative" && value.path.is_ident("Body") && value.rest.is_none() {
            self.edit.insert(
                value.brace_token.span.close().start(),
                quote::quote!(cached_dead:false,),
                "collision-test-input",
            );
        }
        if self.stage == "fixture"
            && value
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "Body")
            && value.rest.is_none()
        {
            self.edit.insert(
                value.brace_token.span.close().start(),
                quote::quote!(cached_dead:false,),
                "client-fixture-input",
            );
        }
        syn::visit::visit_expr_struct(self, value);
    }
    fn visit_stmt(&mut self, statement: &'ast syn::Stmt) {
        let text = statement.to_token_stream().to_string();
        if self.stage == "runtime"
            && self.scope == "new_with_app"
            && text.starts_with("app . insert_resource (DestinySpaceFriction")
        {
            self.edit.insert(
                statement.span().end(),
                quote::quote!(app.init_resource::<crate::generated_liveness::Frame>();),
                "frame-initialize",
            );
        }
        if self.stage == "runtime"
            && self.scope == "advance_one_tick_with_motion"
            && text == "self . app . update () ;"
        {
            self.edit.insert(
                statement.span().start(),
                quote::quote!(if should_step {
                    self.freeze_source_dead();
                }),
                "frame-project",
            );
            self.edit.insert(
                statement.span().end(),
                quote::quote!(if should_step {
                    self.thaw_source_dead();
                }),
                "frame-restore",
            );
        }
        if self.stage == "runtime"
            && self.scope == "finish_callback_tick"
            && text.starts_with("self . evolution_phase =")
        {
            self.edit.insert(statement.span().start(),quote::quote!(self.app.world_mut().resource_mut::<crate::generated_liveness::Frame>().cached_dead.clear();),"frame-finish");
        }
        if self.stage == "runtime"
            && self.scope == "next_tick_notification"
            && text == "pass . owner = None ;"
        {
            // Cleanup is idempotent. Preserve the entity guard so a replaced
            // owner cannot receive the removed object's final membership clear.
            self.edit.insert(
                statement.span().start(),
                quote::quote!(if pass.clear_dead_members {
                    if let Some((owner, entity)) = pass.owner
                        && self.balls.get(&owner) == Some(&entity)
                    {
                        self.clear_source_dead_sensor(owner)?;
                    }
                    pass.clear_dead_members = false;
                }),
                "dead-proximity-clear",
            );
        }
        syn::visit::visit_stmt(self, statement);
    }
    fn visit_expr_for_loop(&mut self, value: &'ast syn::ExprForLoop) {
        if self.stage == "runtime"
            && self.scope == "prepare_motion_tick"
            && value.pat.to_token_stream().to_string() == "id"
        {
            let first = value.body.stmts.first().expect("source free/pending guard");
            self.edit.insert(
                first.span().end(),
                quote::quote!(if self.cache_source_dead(id)? {
                    continue;
                }),
                "cache-control-dead",
            );
        }
        syn::visit::visit_expr_for_loop(self, value);
    }
    fn visit_expr_if(&mut self, value: &'ast syn::ExprIf) {
        let text = value.cond.to_token_stream().to_string();
        if self.stage == "runtime"
            && self.scope == "set_interactive"
            && text.starts_with("let Some (tracking)")
        {
            let cond = &value.cond;
            self.edit.replace(
                &value.cond,
                quote::quote!(was_interactive!=enabled && #cond),
                "interactive-change-only",
            );
        }
        if self.stage == "runtime"
            && self.scope == "update_all_source_angles"
            && text.contains("self . metadata (id) ? . is_free")
        {
            let cond = &value.cond;
            self.edit.replace(
                &value.cond,
                quote::quote!( !self.source_cached_dead(id) && (#cond)),
                "skip-dead-angles",
            );
        }
        if self.stage == "runtime"
            && self.scope == "sync_source_partition_after_motion"
            && text == "self . balls . contains_key (& id)"
        {
            let cond = &value.cond;
            self.edit.replace(
                &value.cond,
                quote::quote!(!self.source_cached_dead(id) && (#cond)),
                "skip-dead-partition",
            );
        }
        if self.stage == "runtime"
            && self.scope == "correct_collision_free_analytic_motion"
            && text.contains("initial . pending_removal . is_some")
        {
            let cond = &value.cond;
            self.edit.replace(&value.cond,quote::quote!((#cond)||world.resource::<crate::generated_liveness::Frame>().cached_dead.contains(&initial.entity)),"skip-dead-correction");
        }
        if self.stage == "runtime"
            && self.scope == "advance_one_tick_with_motion"
            && text.starts_with("self . balls . contains_key (& id)")
        {
            let cond = &value.cond;
            self.edit.replace(
                &value.cond,
                quote::quote!(!self.source_cached_dead(id) && (#cond)),
                "skip-dead-publication",
            );
        }
        if self.stage == "runtime"
            && self.scope == "next_tick_notification"
            && text.starts_with("self . balls . get (& owner)")
        {
            self.edit.insert(
                value.span().end(),
                quote::quote!(if pass.stage == 0 {
                    let (run, clear) = self.prepare_source_proximity(owner)?;
                    if !run {
                        pass.owner = None;
                        continue;
                    }
                    pass.clear_dead_members = clear;
                }),
                "fresh-proximity-gate",
            );
        }
        if self.stage == "iterative"
            && self.scope == "resolve"
            && (text == "body . is_moribund || ! body . is_massive"
                || text == "! body . is_free || body . is_moribund")
        {
            let cond = &value.cond;
            self.edit.replace(
                &value.cond,
                quote::quote!((#cond)||body.cached_dead),
                "skip-dead-iterative",
            );
        }
        if self.stage == "noniterative"
            && self.scope == "resolve"
            && text == "! body . is_free || body . is_moribund"
        {
            let cond = &value.cond;
            self.edit.replace(
                &value.cond,
                quote::quote!((#cond)||body.cached_dead),
                "skip-dead-gradient",
            );
        }
        syn::visit::visit_expr_if(self, value);
    }
    fn visit_expr_method_call(&mut self, value: &'ast syn::ExprMethodCall) {
        if self.stage == "runtime"
            && self.scope == "prepare_mushroom_tick"
            && value.method == "then_some"
        {
            let receiver = &value.receiver;
            self.edit.replace(
                &**receiver,
                quote::quote!(((#receiver)&&!self.source_cached_dead(*id))),
                "skip-dead-mushroom",
            );
        }
        syn::visit::visit_expr_method_call(self, value);
    }
}

pub fn transform(source: &str, stage: &'static str) -> Result<String, Box<dyn Error>> {
    let file = syn::parse_file(source)?;
    let mut pass = Pass {
        edit: super::partition::pass(source),
        scope: String::new(),
        stage,
    };
    pass.visit_file(&file);
    let expected = match stage {
        "bubbles" => vec![("keeper-fields", 1), ("keeper-transition", 1)],
        "iterative" => vec![
            ("collision-dead-field", 1),
            ("collision-test-input", 0),
            ("skip-dead-iterative", 2),
        ],
        "noniterative" => vec![("skip-dead-gradient", 1)],
        "fixture" => vec![("client-fixture-input", 1)],
        "runtime" => vec![
            ("collision-test-live-membership", 1),
            ("frame-clear", 1),
            ("interactive-prior-flag", 1),
            ("proximity-pass-field", 1),
            ("proximity-pass-initialize", 1),
            ("collision-frame-input", 1),
            ("frame-initialize", 1),
            ("frame-project", 1),
            ("frame-restore", 1),
            ("frame-finish", 1),
            ("dead-proximity-clear", 2),
            ("cache-control-dead", 1),
            ("interactive-change-only", 1),
            ("skip-dead-angles", 1),
            ("skip-dead-partition", 1),
            ("skip-dead-correction", 1),
            ("skip-dead-publication", 1),
            ("fresh-proximity-gate", 1),
            ("skip-dead-mushroom", 1),
        ],
        _ => return Err("unknown liveness projection stage".into()),
    };
    let mut output = pass.edit.finish(&expected)?;
    if stage == "bubbles" {
        for function in rules()?
            .functions
            .iter()
            .filter(|function| function.owner.as_deref() == Some("Tracking"))
        {
            output.push_str(&format!("\n{}\n", function.lower()?));
        }
    }
    if stage == "runtime" {
        let bridge: Item = syn::parse_quote! {
            impl CompatRuntime {
                pub fn register_keep_alive(&mut self,id:i64,enabled:bool) {self.original_bubbles.get_or_insert_with(Default::default).register_keep_alive(id,enabled);}
                pub fn add_keep_alive_member(&mut self,id:i64,bubble:i64) {self.original_bubbles.get_or_insert_with(Default::default).keep_alive_add(id,bubble);}
                pub fn remove_keep_alive_member(&mut self,id:i64,bubble:i64) {self.original_bubbles.get_or_insert_with(Default::default).keep_alive_remove(id,bubble);}
                pub fn set_keep_alive_members(&mut self,bubble:i64,members:std::collections::BTreeSet<i64>) {self.original_bubbles.get_or_insert_with(Default::default).set_keep_alive_members(bubble,members);}
                pub fn set_interactive_members(&mut self,bubble:i64,members:std::collections::BTreeSet<i64>) {self.original_bubbles.get_or_insert_with(Default::default).set_interactive_members(bubble,members);}
                pub fn liveness_state(&self)->crate::generated_liveness::LivenessState {self.original_bubbles.as_ref().map(crate::bubbles::Tracking::liveness_state).unwrap_or_default()}
                pub fn is_in_dead_bubble(&self,id:i64)->Result<bool,CompatError> {
                    let bubble=self.metadata(id)?.new_bubble_id;
                    let mode=self.motion.get(&id).cloned().unwrap_or_default().mode;
                    let interactive=self.original_bubbles.as_ref().map_or(0,|tracking|tracking.interactive_count(self.park.is_master,bubble));
                    let keep_alive=self.original_bubbles.as_ref().map_or(0,|tracking|tracking.keep_alive_count(bubble));
                    Ok(crate::generated_liveness::is_dead(self.park.is_master,u32::from(mode_number(mode)),interactive,keep_alive))
                }
                fn cache_source_dead(&mut self,id:i64)->Result<bool,CompatError> {
                    let dead=self.is_in_dead_bubble(id)?;
                    if dead {let entity=self.entity(id)?;self.app.world_mut().resource_mut::<crate::generated_liveness::Frame>().cached_dead.insert(entity);}
                    Ok(dead)
                }
                fn source_cached_dead(&self,id:i64)->bool {self.balls.get(&id).is_some_and(|entity|self.app.world().resource::<crate::generated_liveness::Frame>().cached_dead.contains(entity))}
                fn freeze_source_dead(&mut self) {
                    let entities=self.app.world().resource::<crate::generated_liveness::Frame>().cached_dead.clone();
                    let saved=crate::generated_liveness::project_components(self.app.world_mut(),&entities,RigidBody::Static);
                    self.app.world_mut().resource_mut::<crate::generated_liveness::Frame>().projected=saved;
                }
                fn thaw_source_dead(&mut self) {
                    let saved=std::mem::take(&mut self.app.world_mut().resource_mut::<crate::generated_liveness::Frame>().projected);
                    crate::generated_liveness::restore_components(self.app.world_mut(),saved);
                }
            }
        };
        let file = syn::parse_file(&output)?;
        let tests = file
            .items
            .iter()
            .find(|item| matches!(item,Item::Mod(item) if item.ident=="tests"))
            .ok_or("runtime test module missing")?;
        let at = super::offset(&output, tests.span().start());
        let functions = rules()?
            .functions
            .iter()
            .filter(|function| function.owner.as_deref() == Some("CompatRuntime"))
            .map(|function| function.lower().map(|value| value.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        output.insert_str(
            at,
            &format!("{}\n{}\n", bridge.to_token_stream(), functions.join("\n")),
        );
    }
    syn::parse_file(&output)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn concrete_fixture_adaptation_requires_one_body_and_preserves_assertions() {
        let source = "fn body() -> production::Body {production::Body { id: 1, }} fn check() {assert_eq!(1, 1);}";
        let result = transform(source, "fixture").unwrap();
        assert!(result.contains("cached_dead : false"));
        assert!(result.ends_with("fn check() {assert_eq!(1, 1);}"));
        assert!(transform("fn body() {}", "fixture").is_err());
        assert!(transform("fn body() {Body {id:1,}; Body {id:2,};}", "fixture").is_err());
    }
}
