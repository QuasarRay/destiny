//! Reuse source missile arithmetic and generate scalar/property/runtime wiring.
use genco::prelude::*;
use quote::ToTokens;
use std::{collections::BTreeMap, error::Error, fs};
use syn::{Expr, spanned::Spanned, visit::Visit, visit_mut::VisitMut};

pub fn emit() -> Result<String, Box<dyn Error>> {
    let file = syn::parse_file(&fs::read_to_string(
        "verification/spec/src/original_motion.rs",
    )?)?;
    let mut function = file
        .items
        .into_iter()
        .find_map(|item| match item {
            syn::Item::Fn(value) if value.sig.ident == "original_missile_thrust" => Some(value),
            _ => None,
        })
        .ok_or("missing reused missile kernel")?;
    function.sig.ident = syn::parse_quote!(thrust);
    function.sig.inputs.push(syn::parse_quote!(friction:f64));
    function.sig.inputs.push(syn::parse_quote!(dt:f64));
    function
        .sig
        .inputs
        .push(syn::parse_quote!(tick_interval_ms:i64));
    struct Environment {
        clocks: usize,
        thrust: usize,
    }
    impl VisitMut for Environment {
        fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
            if call.method == "checked_mul" && call.args.to_token_stream().to_string() == "1000" {
                call.args[0] = syn::parse_quote!(tick_interval_ms);
                self.clocks += 1;
            }
            syn::visit_mut::visit_expr_method_call_mut(self, call);
        }
        fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
            if matches!(&*call.func,Expr::Path(path) if path.path.is_ident("original_goto_thrust"))
            {
                assert_eq!(call.args.len(), 5);
                assert_eq!(call.args[2].to_token_stream().to_string(), "1_000_000.0");
                assert_eq!(call.args[3].to_token_stream().to_string(), "1.0");
                call.args[2] = syn::parse_quote!(friction);
                call.args[3] = syn::parse_quote!(dt);
                if matches!(&call.args[4],Expr::Lit(value) if matches!(&value.lit,syn::Lit::Bool(value) if value.value))
                {
                    call.args[4] = syn::parse_quote!(ball.mode == OriginalMode::Missile);
                }
                self.thrust += 1;
            }
            syn::visit_mut::visit_expr_call_mut(self, call);
        }
        fn visit_path_mut(&mut self, path: &mut syn::Path) {
            if path.segments.len() >= 2
                && path.segments[0].ident == "crate"
                && path.segments[1].ident == "OriginalParkError"
            {
                path.segments = path.segments.iter().skip(1).cloned().collect();
            }
            syn::visit_mut::visit_path_mut(self, path);
        }
    }
    let mut environment = Environment {
        clocks: 0,
        thrust: 0,
    };
    environment.visit_item_fn_mut(&mut function);
    if (environment.clocks, environment.thrust) != (1, 2) {
        return Err("missile environment binding changed".into());
    }
    let kernel = function.to_token_stream().to_string();
    // Domain-independent binary32 representation glue. Finite values retain
    // numeric JSON compatibility; bit tags preserve all nonfinite payloads.
    let output: rust::Tokens = quote! {
        use destiny_original_spec::{OriginalMode,OriginalParkBall,OriginalParkError,Vec3,original_goto_thrust};
        use serde::{Serialize,Deserialize};
        $(kernel)

        #[derive(Debug,Clone,Copy,Serialize,Deserialize)]
        #[serde(transparent)]
        pub struct SourceFloat32(#[serde(with="float32")] pub f32);

        pub mod float32 {
            use serde::{Serialize,Deserialize,Serializer,Deserializer};
            #[derive(Serialize,Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Bits { source_f32_bits_v1:u32 }
            #[derive(Deserialize)]
            #[serde(untagged)]
            enum Stored { Bits(Bits), Numeric(f64) }
            pub fn serialize<S:Serializer>(value:&f32,serializer:S)->Result<S::Ok,S::Error> {
                if value.is_finite() { serializer.serialize_f64(f64::from(*value)) }
                else { Bits { source_f32_bits_v1:value.to_bits() }.serialize(serializer) }
            }
            pub fn deserialize<'de,D:Deserializer<'de>>(deserializer:D)->Result<f32,D::Error> {
                Ok(match Stored::deserialize(deserializer)? {
                    Stored::Bits(value)=>f32::from_bits(value.source_f32_bits_v1),
                    Stored::Numeric(value)=>value as f32,
                })
            }
        }
    };
    Ok(format!(
        "// @generated: source missile AST and scalar representation glue. Do not edit.\n// Original CCP ehf. MIT source; bindings: verification/migration/missile_source_evidence.json.\n// Regenerate: python tools/migrate_destiny.py generate\n{}",
        output.to_file_string()?
    ))
}

struct Runtime<'a> {
    source: &'a str,
    scope: String,
    arm: String,
    constructor_normalization: syn::ExprIf,
    edits: Vec<(usize, usize, String)>,
    counts: BTreeMap<&'static str, usize>,
}
impl Runtime<'_> {
    fn replace(&mut self, node: &impl Spanned, text: String, kind: &'static str) {
        self.edits.push((
            super::offset(self.source, node.span().start()),
            super::offset(self.source, node.span().end()),
            text,
        ));
        *self.counts.entry(kind).or_default() += 1;
    }
    fn insert(&mut self, point: proc_macro2::LineColumn, text: String, kind: &'static str) {
        let point = super::offset(self.source, point);
        self.edits.push((point, point, text));
        *self.counts.entry(kind).or_default() += 1;
    }
}
impl<'ast> Visit<'ast> for Runtime<'_> {
    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        let previous = std::mem::replace(&mut self.scope, function.sig.ident.to_string());
        syn::visit::visit_impl_item_fn(self, function);
        self.scope = previous;
    }
    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        let previous = std::mem::replace(&mut self.scope, function.sig.ident.to_string());
        syn::visit::visit_item_fn(self, function);
        self.scope = previous;
    }
    fn visit_item_struct(&mut self, structure: &'ast syn::ItemStruct) {
        if structure.ident == "DestinyBallMetadata" || structure.ident == "BallSnapshot" {
            for field in &structure.fields {
                if field
                    .ident
                    .as_ref()
                    .is_some_and(|name| name == "angular_agility")
                {
                    let mut next = field.clone();
                    next.ty = syn::parse_quote!(f32);
                    next.attrs.push(
                        syn::parse_quote!(#[serde(with="crate::generated_missile::float32")]),
                    );
                    self.replace(
                        field,
                        next.to_token_stream().to_string(),
                        "source-f32-field",
                    );
                }
            }
        }
    }
    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let previous = std::mem::replace(&mut self.arm, arm.pat.to_token_stream().to_string());
        if self.scope == "get_ball_property" && self.arm == "\"Agility\"" {
            let next: syn::Arm = syn::parse_quote! {
                "RotationalAgility"=>serde_json::to_value(crate::generated_missile::SourceFloat32(metadata.angular_agility)).map_err(|error|CompatError::Engine(error.to_string())),
            };
            self.insert(
                arm.span().start(),
                next.to_token_stream().to_string(),
                "property-getter",
            );
        }
        if self.scope == "retained_ball_property" && self.arm == "\"Agility\"" {
            let next: syn::Arm = syn::parse_quote! {
                "RotationalAgility"=>serde_json::to_value(crate::generated_missile::SourceFloat32(snapshot.angular_agility)).map_err(|error|CompatError::Engine(error.to_string())),
            };
            self.insert(
                arm.span().start(),
                next.to_token_stream().to_string(),
                "retained-getter",
            );
        }
        if self.scope == "set_ball_property" && self.arm == "\"Agility\"" {
            let next: syn::Arm = syn::parse_quote! {
                "RotationalAgility"=>{
                    let value:crate::generated_missile::SourceFloat32=serde_json::from_value(value.cloned().unwrap_or(Value::Null)).map_err(|error|CompatError::InvalidRequest(format!("invalid source float32: {error}")))?;
                    self.metadata_mut(ball_id)?.angular_agility=value.0;
                },
            };
            self.insert(
                arm.span().start(),
                next.to_token_stream().to_string(),
                "raw-property-write",
            );
        }
        syn::visit::visit_arm(self, arm);
        self.arm = previous;
    }
    fn visit_expr_if(&mut self, branch: &'ast syn::ExprIf) {
        if self.scope == "prepare_motion_tick"
            && branch.cond.to_token_stream().to_string()
                == "control . mode == OriginalMode :: Formation"
        {
            let next: Expr = syn::parse_quote! {
                if control.mode==OriginalMode::Missile {
                    let tick_interval_ms=i64::try_from(self.app.world().resource::<Time<Fixed>>().timestep().as_millis()).map_err(|_|CompatError::InvalidRequest("source tick interval overflows signed arithmetic".into()))?;
                    crate::generated_missile::thrust(&mut ball,&other,self.park.current_time,friction,dt,tick_interval_ms).map_err(|error|CompatError::InvalidRequest(format!("source missile rejected state: {error:?}")))?
                }
            };
            self.insert(
                branch.span().start(),
                format!("{} else ", next.to_token_stream()),
                "missile-thrust-dispatch",
            );
        }
        if self.scope == "dispatch_inner"
            && branch.cond.to_token_stream().to_string() == "! retained . attached"
        {
            let next: syn::Stmt = syn::parse_quote! {
                if request.operation=="set" && title.rsplit('.').next()==Some("RotationalAgility") {
                    require_arity(&request.args,&[1],title)?;
                    let value:crate::generated_missile::SourceFloat32=serde_json::from_value(request.args[0].clone()).map_err(|error|CompatError::InvalidRequest(format!("invalid source float32: {error}")))?;
                    self.retained_balls.get_mut(&generation).and_then(|ball|ball.detached.as_mut()).ok_or_else(||CompatError::Engine("missing retained scalar owner".into()))?.angular_agility=value.0;
                    return Ok(Value::Null);
                }
            };
            self.insert(
                branch.then_branch.brace_token.span.open().end(),
                next.to_token_stream().to_string(),
                "retained-raw-write",
            );
        }
        if ["validate_ball_snapshot", "add_ball_with_identity"].contains(&self.scope.as_str()) {
            fn prune(expr: Expr, count: &mut usize) -> Option<Expr> {
                let text = expr.to_token_stream().to_string();
                if [
                    "ball . angular_agility < 0.0",
                    "ball . angular_agility > MAX_AGILITY",
                    "angular_agility > MAX_AGILITY",
                ]
                .contains(&text.as_str())
                {
                    *count += 1;
                    return None;
                }
                if let Expr::Binary(mut binary) = expr {
                    if matches!(binary.op, syn::BinOp::Or(_)) {
                        return match (prune(*binary.left, count), prune(*binary.right, count)) {
                            (Some(left), Some(right)) => {
                                binary.left = Box::new(left);
                                binary.right = Box::new(right);
                                Some(Expr::Binary(binary))
                            }
                            (Some(value), None) | (None, Some(value)) => Some(value),
                            (None, None) => None,
                        };
                    }
                    return Some(Expr::Binary(binary));
                }
                Some(expr)
            }
            let mut count = 0;
            let condition =
                prune((*branch.cond).clone(), &mut count).expect("remaining scalar validation");
            if count > 0 {
                self.replace(
                    &*branch.cond,
                    condition.to_token_stream().to_string(),
                    "remove-nonsource-rotational-bound",
                );
            }
        }
        syn::visit::visit_expr_if(self, branch);
    }
    fn visit_expr_macro(&mut self, value: &'ast syn::ExprMacro) {
        if self.scope == "deserialize_snapshot"
            && value.mac.path.is_ident("json")
            && value.mac.tokens.to_string() == "ball . angular_agility"
        {
            let next: Expr = syn::parse_quote! {
                serde_json::to_value(crate::generated_missile::SourceFloat32(ball.angular_agility)).map_err(|error|CompatError::Engine(error.to_string()))?
            };
            self.replace(
                value,
                next.to_token_stream().to_string(),
                "snapshot-f32-argument",
            );
            return;
        }
        if self.scope == "prepare_motion_tick"
            && value.mac.path.is_ident("matches")
            && value
                .mac
                .tokens
                .to_string()
                .contains("OriginalMode :: Formation")
        {
            let tokens = &value.mac.tokens;
            let next: Expr = syn::parse_quote!(matches!(#tokens | OriginalMode::Missile));
            self.replace(
                value,
                next.to_token_stream().to_string(),
                "following-mode-admission",
            );
            return;
        }
        syn::visit::visit_expr_macro(self, value);
    }
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let syn::Pat::Ident(name) = &local.pat {
            if self.scope == "retained_ball_call"
                && self.arm == "\"GetRotatedVector\""
                && name.ident == "result"
            {
                let next: Expr = syn::parse_quote! {
                    crate::generated_angular::rotate(&snapshot.source_angles,OriginalVec3{x:vector.x,y:vector.y,z:vector.z})
                };
                self.replace(
                    &*local.init.as_ref().unwrap().expr,
                    next.to_token_stream().to_string(),
                    "retained-source-rotation",
                );
                return;
            }
            if self.scope == "add_ball_with_identity" && name.ident == "angular_agility" {
                let value = &local.init.as_ref().unwrap().expr;
                // Reuse the existing <=0 -> 1 normalization AST, which is the
                // same scalar rule as original Ballpark.cpp:3552. Parsing and
                // field access are representation glue, not a second rule.
                let normalization = &self.constructor_normalization;
                let next: Expr = syn::parse_quote! {
                    if args.len()==19 {
                        let mut value=serde_json::from_value::<crate::generated_missile::SourceFloat32>(args[18].clone()).map_err(|error|CompatError::InvalidRequest(format!("invalid source float32: {error}")))?.0;
                        #normalization
                        value
                    } else { OriginalParkBall::default().rotational_agility }
                };
                self.replace(
                    &**value,
                    next.to_token_stream().to_string(),
                    "constructor-f32",
                );
                return;
            }
            if self.scope == "validate_ball_snapshot" && name.ident == "scalars" {
                let mut value = local.init.as_ref().unwrap().expr.clone();
                if let Expr::Array(array) = &mut *value {
                    array.elems = array
                        .elems
                        .iter()
                        .filter(|expr| {
                            expr.to_token_stream().to_string() != "ball . angular_agility"
                        })
                        .cloned()
                        .collect();
                }
                self.replace(
                    &*local.init.as_ref().unwrap().expr,
                    value.to_token_stream().to_string(),
                    "allow-source-f32-bits",
                );
                return;
            }
            if self.scope == "dispatch_park" && self.arm == "\"LaunchMissile\"" {
                if name.ident == "rotation" {
                    self.replace(local, String::new(), "remove-quaternion-launch");
                    return;
                }
                if name.ident == "stationary" {
                    let next: Expr =
                        syn::parse_quote!(self.source_rotated_vector(owner_check, -DVec3::Z)?);
                    self.replace(
                        &*local.init.as_ref().unwrap().expr,
                        next.to_token_stream().to_string(),
                        "source-launch-rotation",
                    );
                    return;
                }
            }
        }
        syn::visit::visit_local(self, local);
    }
    fn visit_field_value(&mut self, field: &'ast syn::FieldValue) {
        if field.member.to_token_stream().to_string() == "angular_agility"
            && field.expr.to_token_stream().to_string()
                == "f64 :: from (fields . rotational_agility)"
        {
            let value: Expr = syn::parse_quote!(fields.rotational_agility);
            self.replace(
                &field.expr,
                value.to_token_stream().to_string(),
                "existing-f32-init",
            );
        }
        syn::visit::visit_field_value(self, field);
    }
}

pub fn runtime(source: &str) -> Result<String, Box<dyn Error>> {
    let file = syn::parse_file(source)?;
    let mut constructor_normalization = file
        .items
        .iter()
        .find_map(|item| {
            let syn::Item::Impl(item) = item else {
                return None;
            };
            item.items.iter().find_map(|item| {
                let syn::ImplItem::Fn(function) = item else {
                    return None;
                };
                if function.sig.ident != "add_ball_with_identity" {
                    return None;
                }
                function.block.stmts.iter().find_map(|statement| {
                    let syn::Stmt::Expr(Expr::If(branch), _) = statement else {
                        return None;
                    };
                    (branch.cond.to_token_stream().to_string() == "agility <= 0.0")
                        .then(|| branch.clone())
                })
            })
        })
        .ok_or("missing reviewed nonpositive scalar normalization")?;
    struct RenameScalar;
    impl VisitMut for RenameScalar {
        fn visit_ident_mut(&mut self, name: &mut syn::Ident) {
            if name == "agility" {
                *name = syn::parse_quote!(value);
            }
        }
    }
    RenameScalar.visit_expr_if_mut(&mut constructor_normalization);
    let mut pass = Runtime {
        source,
        scope: String::new(),
        arm: String::new(),
        constructor_normalization,
        edits: Vec::new(),
        counts: BTreeMap::new(),
    };
    pass.visit_file(&file);
    for (kind, count) in [
        ("source-f32-field", 2),
        ("property-getter", 1),
        ("retained-getter", 1),
        ("raw-property-write", 1),
        ("missile-thrust-dispatch", 1),
        ("retained-raw-write", 1),
        ("following-mode-admission", 1),
        ("constructor-f32", 1),
        ("snapshot-f32-argument", 1),
        ("existing-f32-init", 1),
        ("allow-source-f32-bits", 1),
        ("remove-nonsource-rotational-bound", 2),
        ("remove-quaternion-launch", 1),
        ("source-launch-rotation", 1),
        ("retained-source-rotation", 1),
    ] {
        if pass.counts.get(kind) != Some(&count) {
            return Err(format!("missile wiring {kind} count changed: {:?}", pass.counts).into());
        }
    }
    pass.edits.sort_by_key(|edit| (edit.0, edit.1));
    if pass.edits.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Err("overlapping missile transformations".into());
    }
    let mut output = source.to_owned();
    for (start, end, text) in pass.edits.into_iter().rev() {
        output.replace_range(start..end, &text);
    }
    syn::parse_file(&output)?;
    Ok(output)
}
