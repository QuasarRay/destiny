//! Source partition numeric reuse and generated ownership integration.
use genco::prelude::*;
use quote::ToTokens;
use serde::Deserialize;
use std::{collections::BTreeMap, error::Error, fs};
use syn::{Expr, Item, spanned::Spanned, visit::Visit, visit_mut::VisitMut};

pub fn specification(source: &str) -> Result<String, Box<dyn Error>> {
    let file = syn::parse_file(source)?;
    let mut output = source.to_owned();
    let mut edits = Vec::new();
    let mut found = 0;
    for item in file.items {
        if let Item::Impl(item) = &item {
            for item in &item.items {
                if let syn::ImplItem::Fn(function) = item
                    && function.sig.ident == "offset"
                {
                    let visibility = &function.vis;
                    edits.push((
                        super::offset(source, visibility.span().start()),
                        super::offset(source, visibility.span().end()),
                        "pub".to_owned(),
                    ));
                    found += 1;
                }
            }
        }
        let Item::Fn(mut function) = item else {
            continue;
        };
        if ["box_width", "box_low", "box_snapshot"]
            .iter()
            .any(|name| function.sig.ident == name)
        {
            let visibility = &function.vis;
            edits.push((
                super::offset(source, visibility.span().start()),
                super::offset(source, visibility.span().end()),
                "pub".to_owned(),
            ));
            found += 1;
        }
        if function.sig.ident == "point_cell_keys" {
            let original_end = super::offset(source, function.span().end());
            let body = &function.block;
            let wrapper: syn::Block = syn::parse_quote!({
                point_cell_keys_with_modifiers(position, radius, inflated, allow_large_branch)
                    .map(|(keys, _)| keys)
            });
            edits.push((
                super::offset(source, body.span().start()),
                super::offset(source, body.span().end()),
                wrapper.to_token_stream().to_string(),
            ));
            function.sig.ident = syn::parse_quote!(point_cell_keys_with_modifiers);
            function.vis = syn::parse_quote!(pub);
            function.sig.output = syn::parse_quote!(-> Option<(SphericalCellKeys,[i64;3])>);
            let Some(syn::Stmt::Expr(Expr::Call(call), _)) = function.block.stmts.last_mut() else {
                return Err("source footprint return changed".into());
            };
            if call.func.to_token_stream().to_string() != "Some"
                || call.args.to_token_stream().to_string() != "result"
            {
                return Err("source footprint return shape changed".into());
            }
            call.args[0] = syn::parse_quote!((result, [mx, my, mz]));
            edits.push((
                original_end,
                original_end,
                format!("\n{}\n", function.to_token_stream()),
            ));
            found += 1;
        }
    }
    if found != 5 {
        return Err("partition numeric source selection changed".into());
    }
    edits.sort_by_key(|edit| (edit.0, edit.1));
    for (start, end, replacement) in edits.into_iter().rev() {
        output.replace_range(start..end, &replacement);
    }
    syn::parse_file(&output)?;
    Ok(output)
}

pub fn specification_library(source: &str) -> Result<String, Box<dyn Error>> {
    syn::parse_file(source)?;
    let exports: syn::Item = syn::parse_quote! {
        pub use original_partition::{point_cell_keys_with_modifiers as original_partition_placement,box_snapshot as original_partition_box_snapshot,box_low as original_partition_box_low,box_width as original_partition_box_width};
    };
    Ok(format!("{source}\n{}\n", exports.to_token_stream()))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rules {
    schema: u32,
    original_commit: String,
    functions: Vec<super::state_rules::Function>,
}

pub fn emit() -> Result<String, Box<dyn Error>> {
    let rules: Rules = serde_json::from_str(&fs::read_to_string(
        "verification/migration/partition_rules.json",
    )?)?;
    if rules.schema != 1 || rules.original_commit != "114e89fa584f0e1a81eb0a679aabf41af8870072" {
        return Err("unreviewed partition rules".into());
    }
    let file = syn::parse_file(&fs::read_to_string(
        "verification/spec/src/original_park.rs",
    )?)?;
    let mut inflated = None;
    for item in file.items {
        let Item::Impl(item) = item else { continue };
        for item in item.items {
            if let syn::ImplItem::Fn(function) = item
                && function.sig.ident == "inflated_radius"
            {
                inflated = Some(function);
            }
        }
    }
    let mut inflated = inflated.ok_or("missing reused inflated radius")?;
    inflated.vis = syn::parse_quote!(pub);
    inflated.sig =
        syn::parse_quote!(fn inflated_radius(b:&OriginalParkBall,friction:f64,dt:f64)->f64);
    struct Environment {
        friction: usize,
        duration: usize,
    }
    impl VisitMut for Environment {
        fn visit_expr_mut(&mut self, expression: &mut Expr) {
            if expression.to_token_stream().to_string() == "self . friction" {
                *expression = syn::parse_quote!(friction);
                self.friction += 1;
                return;
            }
            if let Expr::Binary(binary) = expression
                && matches!(binary.op, syn::BinOp::Add(_))
                && binary.left.to_token_stream().to_string() == "f64 :: from (b . radius)"
            {
                let right = &binary.right;
                *binary.right = syn::parse_quote!(dt*(#right));
                self.duration += 1;
            }
            syn::visit_mut::visit_expr_mut(self, expression);
        }
    }
    let mut environment = Environment {
        friction: 0,
        duration: 0,
    };
    environment.visit_block_mut(&mut inflated.block);
    if environment.friction != 3 || environment.duration != 1 {
        return Err("inflated radius environment selection changed".into());
    }
    let inflated = inflated.to_token_stream().to_string();
    let functions = rules
        .functions
        .iter()
        .map(|function| function.lower().map(|value| value.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let output: rust::Tokens = quote! {
        use destiny_original_spec::{OriginalParkBall,Vec3,OriginalPartitionCellKey as CellKey,original_partition_box_snapshot as box_snapshot};
        use serde::{Serialize,Deserialize};
        // Derived physics-schedule input, refreshed from the sole ParkMetadata
        // owner before App::update. It is not snapshot/authoritative state.
        #[derive(bevy::prelude::Resource,Default)]
        pub(crate) struct CollisionContext {pub is_master:bool}
        #[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct Address {pub key:[i64;3],pub level:u32}
        #[derive(Debug,Clone,Serialize,Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct BoxRecord {pub address:Address,pub bubble:i64,pub object:Option<serde_json::Value>}
        #[derive(Debug,Clone,Serialize,Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct Placement {pub id:i64,pub primary:Address,pub modifiers:[i64;3],pub owned:Vec<Address>}
        #[derive(Debug,Default,Clone,Serialize,Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct Partition {pub boxes:Vec<BoxRecord>,pub placements:Vec<Placement>,pub next_bubble:i64,#[serde(default)] pub undefined_coordinates:Vec<i64>}
        #[derive(Debug,Clone,Copy)]
        pub struct PreparedInsertion {pub label:Option<i64>,pub transition:Option<i64>}
        $(inflated)
        $(for function in functions => $(function))
    };
    Ok(format!(
        "// @generated: source partition state rules and unchanged numeric kernels.\n// Bindings: verification/migration/partition_source_evidence.json; partition_rules.json.\n{}",
        output.to_file_string()?
    ))
}

pub(crate) struct Rewrite<'a> {
    source: &'a str,
    scope: String,
    arm: String,
    edits: Vec<(usize, usize, String)>,
    counts: BTreeMap<&'static str, usize>,
}
impl Rewrite<'_> {
    pub(crate) fn replace(
        &mut self,
        node: &impl Spanned,
        tokens: impl ToTokens,
        kind: &'static str,
    ) {
        self.edits.push((
            super::offset(self.source, node.span().start()),
            super::offset(self.source, node.span().end()),
            tokens.to_token_stream().to_string(),
        ));
        *self.counts.entry(kind).or_default() += 1;
    }
    pub(crate) fn insert(
        &mut self,
        point: proc_macro2::LineColumn,
        tokens: impl ToTokens,
        kind: &'static str,
    ) {
        let point = super::offset(self.source, point);
        self.edits
            .push((point, point, tokens.to_token_stream().to_string()));
        *self.counts.entry(kind).or_default() += 1;
    }
    pub(crate) fn finish(mut self, expected: &[(&str, usize)]) -> Result<String, Box<dyn Error>> {
        for (kind, count) in expected {
            if self.counts.get(kind).copied().unwrap_or(0) != *count {
                return Err(format!("partition rewrite {kind} changed: {:?}", self.counts).into());
            }
        }
        if self
            .counts
            .keys()
            .any(|kind| !expected.iter().any(|(name, _)| name == kind))
        {
            return Err(format!("unexpected partition rewrites: {:?}", self.counts).into());
        }
        self.edits.sort_by_key(|edit| (edit.0, edit.1));
        if self.edits.windows(2).any(|pair| pair[0].1 > pair[1].0) {
            return Err("overlapping partition rewrites".into());
        }
        let mut output = self.source.to_owned();
        for (start, end, replacement) in self.edits.into_iter().rev() {
            output.replace_range(start..end, &replacement);
        }
        if let Err(error) = syn::parse_file(&output) {
            let diagnostic = ".aegis/destiny/migration/partition-rewrite-invalid.rs";
            fs::write(diagnostic, &output)?;
            return Err(format!(
                "invalid partition rewrite at {:?}: {error}; see {diagnostic}",
                error.span().start()
            )
            .into());
        }
        Ok(output)
    }
}

impl<'ast> Visit<'ast> for Rewrite<'_> {
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        if let syn::UseTree::Path(path) = &item.tree
            && path.ident == "destiny_original_spec"
            && let syn::UseTree::Group(group) = &*path.tree
            && group.items.iter().any(|entry| matches!(entry, syn::UseTree::Rename(rename) if rename.ident == "OriginalPartitionCellKey" && rename.rename == "Key"))
        {
            let mut replacement = item.clone();
            let syn::UseTree::Path(path) = &mut replacement.tree else { unreachable!() };
            let syn::UseTree::Group(group) = &mut *path.tree else { unreachable!() };
            group.items = group.items.iter().filter(|entry| !matches!(entry, syn::UseTree::Rename(rename) if rename.ident == "OriginalPartitionCellKey" && rename.rename == "Key")).cloned().collect();
            self.replace(item, replacement, "partition-import");
        }
        syn::visit::visit_item_use(self, item);
    }
    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        let old = std::mem::replace(&mut self.scope, function.sig.ident.to_string());
        if self.scope == "update_single_compatibility_bubble" {
            self.replace(
                function,
                proc_macro2::TokenStream::new(),
                "remove-bubble-fabrication",
            );
            self.scope = old;
            return;
        }
        if self.scope == "filter_pairs" {
            let Some(syn::Stmt::Expr(tail, None)) = function.block.stmts.last() else {
                panic!("collision hook return shape changed");
            };
            assert!(
                tail.to_token_stream()
                    .to_string()
                    .contains("left . new_bubble_id >= 0")
            );
            self.replace(
                tail,
                quote::quote!(crate::generated_partition::source_pair_filter(
                    self.context.is_master,
                    left.new_bubble_id,
                    right.new_bubble_id,
                    left.is_global,
                    right.is_global
                )),
                "source-pair-filter",
            );
        }
        if ["set_position", "set_radius", "set_max_velocity", "set_free"]
            .contains(&self.scope.as_str())
        {
            let next = syn::Ident::new(&format!("{}_value", self.scope), function.sig.ident.span());
            self.replace(&function.sig.ident, next, "storage-method");
        }
        if self.scope == "restore_evolution_checkpoint" {
            self.insert(
                function.block.brace_token.span.open().end(),
                quote::quote!(self.original_bubbles=checkpoint.original_bubbles.clone();),
                "checkpoint-restore",
            );
        }
        if self.scope == "bubble_at" {
            let Some(syn::Stmt::Expr(tail, _)) = function.block.stmts.last() else {
                unreachable!()
            };
            self.replace(
                tail,
                quote::quote!(Ok(self
                    .partition
                    .bubble(crate::generated_partition::Address::from_cell(key, 0)))),
                "bubble-lookup",
            );
        }
        syn::visit::visit_impl_item_fn(self, function);
        self.scope = old;
    }
    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        let old = std::mem::replace(&mut self.scope, function.sig.ident.to_string());
        if self.scope == "initialize" {
            let mut next = function.sig.clone();
            next.inputs.insert(0, syn::parse_quote!(tracking:Tracking));
            self.replace(&function.sig, next, "tracking-initialize-signature");
        }
        syn::visit::visit_item_fn(self, function);
        self.scope = old;
    }
    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let previous = std::mem::replace(&mut self.arm, arm.pat.to_token_stream().to_string());
        if self.scope == "dispatch_park" && self.arm == "\"GetBoxCenter\"" {
            let extra: Vec<syn::Arm> = vec![
                syn::parse_quote!("GetBallBox"=>{require_arity(args,&[1],title)?;let id=value_i64(args.first(),"srcID")?;if self.original_bubbles.as_ref().is_some_and(|tracking|tracking.partition.undefined_coordinates.contains(&id)){return Err(CompatError::InvalidRequest("source partition coordinate conversion is undefined".into()));}serde_json::to_value(self.original_bubbles.as_ref().and_then(|tracking|tracking.partition.ball_box(id))).map_err(|error|CompatError::Engine(error.to_string()))}),
            ];
            self.insert(
                arm.span().start(),
                quote::quote!(#(#extra,)*),
                "box-query-dispatch",
            );
        }
        syn::visit::visit_arm(self, arm);
        self.arm = previous;
    }
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        if item.ident == "DestinyBubbleCollisionHooks" {
            let syn::Fields::Named(fields) = &item.fields else {
                unreachable!()
            };
            assert!(fields.named.trailing_punct());
            self.insert(
                fields.brace_token.span.close().start(),
                quote::quote!(context:Res<'w,crate::generated_partition::CollisionContext>,),
                "collision-context-reader",
            );
        }
        if ["Tracking", "Entry"].iter().any(|name| item.ident == name) {
            self.insert(
                item.span().start(),
                quote::quote!(#[derive(Debug,serde::Serialize,serde::Deserialize)]),
                "tracking-serde",
            );
        }
        if item.ident == "Tracking" {
            for field in &item.fields {
                if field.ident.as_ref().is_some_and(|name| name == "cells") {
                    let next: syn::Field =
                        syn::parse_quote!(pub partition:crate::generated_partition::Partition);
                    self.replace(field, next, "partition-owner");
                }
            }
        }
        if item.ident == "ParkSnapshotMetadata" || item.ident == "EvolutionCheckpoint" {
            let next: syn::Field = if item.ident == "ParkSnapshotMetadata" {
                syn::parse_quote!(#[serde(default)] source_partition_v1:Option<crate::bubbles::Tracking>)
            } else {
                syn::parse_quote!(original_bubbles:Option<crate::bubbles::Tracking>)
            };
            let syn::Fields::Named(fields) = &item.fields else {
                unreachable!()
            };
            let comma = (!fields.named.is_empty() && !fields.named.trailing_punct())
                .then(|| quote::quote!(,));
            self.insert(
                fields.brace_token.span.close().start(),
                quote::quote!(#comma #next,),
                "snapshot-field",
            );
        }
        syn::visit::visit_item_struct(self, item);
    }
    fn visit_local(&mut self, local: &'ast syn::Local) {
        let name = match &local.pat {
            syn::Pat::Ident(name) => Some(name),
            syn::Pat::Type(typed) => match &*typed.pat {
                syn::Pat::Ident(name) => Some(name),
                _ => None,
            },
            _ => None,
        };
        if let Some(name) = name {
            if self.scope == "initialize" && name.ident == "tracking" {
                self.replace(local,quote::quote!(let mut tracking=Tracking{partition:tracking.partition,..Tracking::default()};let mut cells=tracking.partition.unlabeled_top_cells();),"tracking-initialize-owner");
                return;
            }
            if self.scope == "initialize_source_bubbles" && name.ident == "inflated" {
                let value: Expr = syn::parse_quote!(crate::generated_partition::inflated_radius(
                    &ball,
                    self.app.world().resource::<DestinySpaceFriction>().0,
                    self.app
                        .world()
                        .resource::<Time<Fixed>>()
                        .timestep()
                        .as_secs_f64()
                ));
                self.replace(
                    &*local.init.as_ref().unwrap().expr,
                    value,
                    "inflated-environment",
                );
                return;
            }
            if self.scope == "deserialize_snapshot" && name.ident == "incoming_by_id" {
                self.insert(
                    local.span().start(),
                    quote::quote!(if let Some(tracking) = &snapshot.park.source_partition_v1
                        && !tracking.partition.valid_for_balls(&ids)
                    {
                        return Err(CompatError::InvalidRequest(
                            "invalid source partition snapshot".into(),
                        ));
                    }),
                    "snapshot-validate",
                );
            }
        }
        syn::visit::visit_local(self, local);
    }
    fn visit_expr_field(&mut self, field: &'ast syn::ExprField) {
        if self.scope == "initialize" && field.to_token_stream().to_string() == "tracking . cells" {
            self.replace(field, quote::quote!(cells), "top-cell-view");
            return;
        }
        syn::visit::visit_expr_field(self, field);
    }
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if self.scope == "configured_pair" && call.func.to_token_stream().to_string() == "runtime" {
            assert_eq!(call.args.to_token_stream().to_string(), "false");
            self.replace(&call.args[0], quote::quote!(true), "master-hook-fixture");
        }
        if self.scope == "initialize" && call.func.to_token_stream().to_string() == "Ok" {
            self.insert(
                call.span().start(),
                quote::quote!(tracking.partition.apply_top_labels(cells,next);),
                "top-label-commit",
            );
        }
        if self.scope == "initialize_source_bubbles"
            && call.func.to_token_stream().to_string() == "crate :: bubbles :: initialize"
        {
            let mut next = call.clone();
            next.args.insert(
                0,
                syn::parse_quote!(self.original_bubbles.take().unwrap_or_default()),
            );
            self.replace(call, next, "initialize-existing-owner");
            return;
        }
        if self.scope == "add_ball_with_identity"
            && call.func.to_token_stream().to_string() == "Ok"
            && call
                .args
                .to_token_stream()
                .to_string()
                .starts_with("format !")
        {
            self.insert(
                call.span().start(),
                quote::quote!({
                    self.sync_source_partition(id)?;
                }),
                "constructor-insert",
            );
        }
        syn::visit::visit_expr_call(self, call);
    }
    fn visit_expr_return(&mut self, expr: &'ast syn::ExprReturn) {
        if self.scope == "add_ball_with_identity"
            && expr.expr.as_ref().is_some_and(|value| {
                value
                    .to_token_stream()
                    .to_string()
                    .starts_with("Ok (format !")
            })
        {
            self.insert(
                expr.span().start(),
                quote::quote!(self.sync_source_partition(id)?;),
                "constructor-return-insert",
            );
            return;
        }
        syn::visit::visit_expr_return(self, expr);
    }
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if self.scope == "add_ball_with_identity"
            && ["set_position", "set_radius", "set_max_velocity", "set_free"]
                .iter()
                .any(|name| call.method == name)
        {
            let method = syn::Ident::new(&format!("{}_value", call.method), call.method.span());
            self.replace(&call.method, method, "constructor-storage-call");
        }
        if self.scope == "advance_one_tick_with_motion" && call.method == "set_position" {
            self.replace(
                &call.method,
                quote::quote!(set_position_value),
                "motion-storage-call",
            );
        }
        if self.scope == "advance_one_tick_with_motion" && call.method == "update_all_source_angles"
        {
            *self.counts.entry("motion-site-audit").or_default() += 1;
        }
        if self.scope == "remove_ball_internal" && call.method == "detach_retained_ball" {
            // Insert at the containing statement below, rather than inside a
            // fallible method expression.
        }
        syn::visit::visit_expr_method_call(self, call);
    }
    fn visit_stmt(&mut self, statement: &'ast syn::Stmt) {
        let text = statement.to_token_stream().to_string();
        if self.scope == "v4_zero_angle_cone_extreme_triangle_and_global_blocker_are_defined"
            && text == "add_ball (& mut runtime , 3 , true , true) ;"
        {
            self.insert(statement.span().end(),quote::quote!(runtime.initialize_source_bubbles().expect("explicit source bubble initialization");),"explicit-query-bubble-fixture");
        }
        if self.scope == "new_with_app"
            && text.starts_with("app . insert_resource (DestinySpaceFriction")
        {
            self.insert(statement.span().end(),quote::quote!(app.insert_resource(crate::generated_partition::CollisionContext{is_master:options.is_master});),"collision-context-initialize");
        }
        if self.scope == "advance_one_tick_with_motion"
            && text == "self . update_single_compatibility_bubble () ;"
        {
            self.replace(
                statement,
                proc_macro2::TokenStream::new(),
                "remove-late-bubble-fabrication",
            );
            return;
        }
        if self.scope == "advance_one_tick_with_motion" && text == "self . app . update () ;" {
            self.insert(statement.span().start(),quote::quote!(self.app.world_mut().resource_mut::<crate::generated_partition::CollisionContext>().is_master=self.park.is_master;),"collision-context-refresh");
        }
        if self.scope == "remove_ball_internal" && text.starts_with("self . detach_retained_ball") {
            self.insert(
                statement.span().start(),
                quote::quote!(if let Some(tracking) = &mut self.original_bubbles {
                    tracking.partition.remove(ball_id);
                }),
                "remove-placement",
            );
        }
        if self.scope == "advance_one_tick_with_motion"
            && text == "self . update_all_source_angles () ? ;"
        {
            self.insert(
                statement.span().end(),
                quote::quote!(self.sync_source_partition_after_motion(checkpoint)?;),
                "motion-partition-phase",
            );
        }
        if self.scope == "deserialize_snapshot" && text == "self . rebuild_follower_sets () ;" {
            self.insert(
                statement.span().start(),
                quote::quote!(if partial != 2
                    && let Some(tracking) = snapshot.park.source_partition_v1
                {
                    self.original_bubbles = Some(tracking);
                }),
                "snapshot-restore",
            );
        }
        syn::visit::visit_stmt(self, statement);
    }
    fn visit_expr_if(&mut self, branch: &'ast syn::ExprIf) {
        if self.scope == "begin_sensor_pass"
            && branch.cond.to_token_stream().to_string() == "metadata . new_bubble_id < 0"
        {
            let condition = &branch.cond;
            self.replace(
                &branch.cond,
                quote::quote!(self.park.is_master && (#condition)),
                "source-client-sensor-admission",
            );
        }
        if self.scope == "advance_one_tick_with_motion"
            && branch.cond.to_token_stream().to_string() == "should_step"
            && branch.then_branch.stmts.len() == 1
            && branch.then_branch.stmts[0].to_token_stream().to_string()
                == "self . update_single_compatibility_bubble () ;"
        {
            self.replace(
                branch,
                proc_macro2::TokenStream::new(),
                "remove-early-bubble-fabrication",
            );
            return;
        }
        if self.scope == "prepare_iterative_sphere_tick"
            && branch
                .cond
                .to_token_stream()
                .to_string()
                .contains("new_bubble_id > 0")
        {
            let Expr::Binary(condition) = &*branch.cond else {
                panic!("collision guard shape changed")
            };
            assert!(matches!(condition.op, syn::BinOp::Or(_)));
            let mut next = branch.clone();
            next.cond = condition.left.clone();
            self.replace(branch, next, "source-all-bubble-ids");
            return;
        }
        if self.scope == "initialize_source_bubbles"
            && branch.cond.to_token_stream().to_string() == "self . park . is_master"
        {
            self.replace(branch, proc_macro2::TokenStream::new(), "master-initialize");
            return;
        }
        syn::visit::visit_expr_if(self, branch);
    }
    fn visit_field_value(&mut self, field: &'ast syn::FieldValue) {
        if field.member.to_token_stream().to_string() == "original_bubbles"
            && field.expr.to_token_stream().to_string() == "None"
        {
            self.replace(
                &field.expr,
                quote::quote!(Some(crate::bubbles::Tracking::default())),
                "initial-tracking",
            );
        }
        syn::visit::visit_field_value(self, field);
    }
    fn visit_expr_assign(&mut self, assignment: &'ast syn::ExprAssign) {
        if self.scope == "clear_all"
            && assignment.left.to_token_stream().to_string() == "self . original_bubbles"
        {
            self.replace(
                &*assignment.right,
                quote::quote!(Some({
                    let mut partition = self.original_bubbles.take().unwrap_or_default().partition;
                    partition.clear_owners();
                    crate::bubbles::Tracking::from_partition(partition)
                })),
                "clear-partition",
            );
        }
        syn::visit::visit_expr_assign(self, assignment);
    }
    fn visit_expr_struct(&mut self, value: &'ast syn::ExprStruct) {
        if self.scope == "original_motion_input" && value.path.is_ident("OriginalParkBall") {
            assert!(
                !value
                    .fields
                    .iter()
                    .any(|field| field.member.to_token_stream().to_string() == "is_free")
            );
            self.insert(
                value
                    .dot2_token
                    .as_ref()
                    .expect("source input defaults")
                    .span()
                    .start(),
                quote::quote!(is_free:metadata.is_free,),
                "source-free-state-projection",
            );
        }
        if value.path.to_token_stream().to_string() == "EvolutionCheckpoint" {
            let comma = (!value.fields.is_empty() && !value.fields.trailing_punct())
                .then(|| quote::quote!(,));
            self.insert(
                value.brace_token.span.close().start(),
                quote::quote!(#comma original_bubbles:self.original_bubbles.clone(),),
                "checkpoint-capture",
            );
        }
        if value.path.to_token_stream().to_string() == "ParkSnapshotMetadata" {
            let comma = (!value.fields.is_empty() && !value.fields.trailing_punct())
                .then(|| quote::quote!(,));
            self.insert(value.brace_token.span.close().start(),quote::quote!(#comma source_partition_v1:if selected_set.len()==self.balls.len(){self.original_bubbles.clone()}else{None},),"snapshot-capture");
        }
        syn::visit::visit_expr_struct(self, value);
    }
}

pub(crate) fn pass(source: &str) -> Rewrite<'_> {
    Rewrite {
        source,
        scope: String::new(),
        arm: String::new(),
        edits: Vec::new(),
        counts: BTreeMap::new(),
    }
}

pub fn bubbles(source: &str) -> Result<String, Box<dyn Error>> {
    let file = syn::parse_file(source)?;
    let mut rewrite = pass(source);
    rewrite.visit_file(&file);
    let output = rewrite.finish(&[
        ("partition-import", 1),
        ("tracking-serde", 2),
        ("partition-owner", 1),
        ("tracking-initialize-signature", 1),
        ("tracking-initialize-owner", 1),
        ("top-cell-view", 10),
        ("top-label-commit", 1),
        ("bubble-lookup", 1),
    ])?;
    let constructor: syn::Item = syn::parse_quote!(impl Tracking {pub(crate) fn from_partition(partition:crate::generated_partition::Partition)->Self{Self{partition,..Self::default()}}});
    let file = syn::parse_file(&output)?;
    // Existing BFS kernel, represented with an iterator after cells became a
    // local view. Preserve discovery order and queue indices exactly.
    struct CellIteration {
        edits: Vec<(proc_macro2::Span, syn::ExprForLoop)>,
    }
    impl<'ast> Visit<'ast> for CellIteration {
        fn visit_expr_for_loop(&mut self, original: &'ast syn::ExprForLoop) {
            let mut value = original.clone();
            if value.expr.to_token_stream().to_string() == "0 .. cells . len ()" {
                *value.pat = syn::parse_quote!((index, cell));
                *value.expr = syn::parse_quote!(cells.iter_mut().enumerate());
                value.body.stmts.remove(0);
                struct Index;
                impl VisitMut for Index {
                    fn visit_expr_mut(&mut self, value: &mut syn::Expr) {
                        if value.to_token_stream().to_string() == "cells [index]" {
                            *value = syn::parse_quote!(cell);
                        } else {
                            syn::visit_mut::visit_expr_mut(self, value);
                        }
                    }
                }
                Index.visit_block_mut(&mut value.body);
                self.edits.push((original.span(), value));
            }
            syn::visit::visit_expr_for_loop(self, original);
        }
    }
    let mut iteration = CellIteration { edits: Vec::new() };
    iteration.visit_file(&file);
    if iteration.edits.len() != 1 {
        return Err("bubble BFS iteration selection changed".into());
    }
    let (span, replacement) = iteration.edits.remove(0);
    let mut output = output;
    output.replace_range(
        super::offset(&output, span.start())..super::offset(&output, span.end()),
        &replacement.to_token_stream().to_string(),
    );
    Ok(format!("{output}\n{}\n", constructor.to_token_stream()))
}

pub fn runtime(source: &str) -> Result<String, Box<dyn Error>> {
    let file = syn::parse_file(source)?;
    let mut rewrite = pass(source);
    rewrite.visit_file(&file);
    // The bridge only projects real ECS state into the shared numeric kernel
    // and applies generated ownership transitions. No second world is kept.
    let mut transition = None;
    for item in &file.items {
        if let Item::Impl(item) = item {
            for item in &item.items {
                if let syn::ImplItem::Fn(function) = item
                    && function.sig.ident == "initialize_source_bubbles"
                {
                    for statement in &function.block.stmts {
                        if let syn::Stmt::Expr(Expr::ForLoop(loop_), _) = statement
                            && loop_.expr.to_token_stream().to_string() == "labels"
                        {
                            transition = Some(loop_.body.clone());
                        }
                    }
                }
            }
        }
    }
    let transition = transition.ok_or("missing existing source bubble transition")?;
    let bridge: syn::Item = syn::parse_quote! {
        impl CompatRuntime {
            fn sync_source_partition(&mut self,id:i64)->Result<(),CompatError> {
                let control=self.motion.get(&id).cloned().unwrap_or_default();
                let ball=self.original_motion_input(id,&control)?;
                let friction=self.app.world().resource::<DestinySpaceFriction>().0;
                let dt=self.app.world().resource::<Time<Fixed>>().timestep().as_secs_f64();
                let inflated=crate::generated_partition::inflated_radius(&ball,friction,dt);
                let Some((footprint,modifiers))=destiny_original_spec::original_partition_placement(ball.position,ball.radius,inflated,true) else {
                    self.original_bubbles.get_or_insert_with(Default::default).partition.unplace_undefined(id);
                    return Ok(());
                };
                let owned=(0..footprint.keys.len()).map(|index|footprint.keys.get(index).map(|key|crate::generated_partition::Address::from_cell(key,footprint.level)).ok_or_else(||CompatError::Engine("missing source footprint entry".into()))).collect::<Result<Vec<_>,_>>()?;
                let primary=owned.first().copied().ok_or_else(||CompatError::Engine("empty source footprint".into()))?;
                let current=self.metadata(id)?.new_bubble_id;
                let mut tracking=self.original_bubbles.take().unwrap_or_default();
                let prepared=tracking.partition.prepare_insert(id,primary,self.park.is_master,current);
                if let Some(label)=prepared.transition #transition
                tracking.partition.finish_insert(id,primary,modifiers,owned,prepared);
                self.original_bubbles=Some(tracking);
                Ok(())
            }
            fn sync_source_partition_after_motion(&mut self,checkpoint:&EvolutionCheckpoint)->Result<(),CompatError> {
                let mut old=checkpoint.balls.iter().collect::<Vec<_>>();old.sort_unstable_by_key(|ball|ball.ball_id.0);
                for ball in old {
                    let id=ball.ball_id.0;
                    if self.balls.contains_key(&id) {
                        let current=self.position(id)?;
                        if crate::generated_partition::tick_reinsert(OriginalVec3{x:ball.position.0.x,y:ball.position.0.y,z:ball.position.0.z},OriginalVec3{x:current.x,y:current.y,z:current.z},self.metadata(id)?.is_free,self.park.pending_removals.contains_key(&id),self.motion.get(&id).is_some_and(|control|control.mode==OriginalMode::Mushroom)) {self.sync_source_partition(id)?;}
                    }
                }
                Ok(())
            }
            fn set_position(&mut self,id:i64,position:DVec3)->Result<(),CompatError>{self.set_position_value(id,position)?;self.sync_source_partition(id)}
            fn set_radius(&mut self,id:i64,radius:f64)->Result<(),CompatError>{self.set_radius_value(id,radius)?;self.sync_source_partition(id)}
            fn set_max_velocity(&mut self,id:i64,speed:f64)->Result<(),CompatError>{self.set_max_velocity_value(id,speed)?;self.sync_source_partition(id)}
            fn set_free(&mut self,id:i64,free:bool)->Result<(),CompatError>{let previous=self.metadata(id)?.is_free;self.set_free_value(id,free)?;if crate::generated_partition::became_fixed(previous,free){self.sync_source_partition(id)?;}Ok(())}
        }
    };
    let output = rewrite.finish(&[
        ("remove-bubble-fabrication", 1),
        ("source-pair-filter", 1),
        ("collision-context-reader", 1),
        ("collision-context-initialize", 1),
        ("collision-context-refresh", 1),
        ("remove-early-bubble-fabrication", 1),
        ("remove-late-bubble-fabrication", 1),
        ("source-all-bubble-ids", 1),
        ("source-free-state-projection", 1),
        ("source-client-sensor-admission", 1),
        ("master-hook-fixture", 1),
        ("explicit-query-bubble-fixture", 1),
        ("storage-method", 4),
        ("snapshot-field", 2),
        ("checkpoint-restore", 1),
        ("checkpoint-capture", 1),
        ("snapshot-capture", 1),
        ("snapshot-validate", 1),
        ("snapshot-restore", 1),
        ("constructor-storage-call", 4),
        ("motion-storage-call", 1),
        ("box-query-dispatch", 1),
        ("inflated-environment", 1),
        ("initialize-existing-owner", 1),
        ("master-initialize", 1),
        ("initial-tracking", 1),
        ("clear-partition", 1),
        ("remove-placement", 1),
        ("constructor-insert", 1),
        ("constructor-return-insert", 1),
        ("motion-partition-phase", 1),
        ("motion-site-audit", 1),
    ])?;
    let file = syn::parse_file(&output)?;
    let tests = file
        .items
        .iter()
        .find(|item| matches!(item, Item::Mod(item) if item.ident == "tests"))
        .ok_or("runtime test module missing")?;
    let at = super::offset(&output, tests.span().start());
    let mut output = output;
    output.insert_str(at, &format!("{}\n", bridge.to_token_stream()));
    Ok(output)
}
