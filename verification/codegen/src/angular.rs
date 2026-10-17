//! Source-owned angular state: transplant the reviewed Rust syntax and bind
//! its proof-model environment constants to the corresponding world inputs.
use genco::prelude::*;
use quote::ToTokens;
use serde::Deserialize;
use std::{collections::BTreeMap, error::Error, fs};
use syn::{Expr, Stmt, spanned::Spanned, visit::Visit, visit_mut::VisitMut};

fn formation_kernel() -> Result<String, Box<dyn Error>> {
    let file = syn::parse_file(&fs::read_to_string(
        "verification/spec/src/original_rotation.rs",
    )?)?;
    let mut function = file
        .items
        .into_iter()
        .find_map(|item| match item {
            syn::Item::Fn(function) if function.sig.ident == "original_formation_thrust" => {
                Some(function)
            }
            _ => None,
        })
        .ok_or("missing reused formation kernel")?;
    let start = function.block.stmts.iter().position(|statement| matches!(statement, Stmt::Local(local) if matches!(&local.pat, syn::Pat::Ident(name) if name.ident == "length"))).ok_or("formation arithmetic boundary changed")?;
    function.block.stmts.drain(..start);
    let Some(Stmt::Expr(Expr::Call(result), None)) = function.block.stmts.last_mut() else {
        return Err("formation result shape changed".into());
    };
    if !matches!(&*result.func, Expr::Path(path) if path.path.is_ident("Ok"))
        || result.args.len() != 1
    {
        return Err("formation result wrapper changed".into());
    }
    let value = result.args[0].clone();
    *function.block.stmts.last_mut().unwrap() = Stmt::Expr(value, None);
    struct Calls {
        rotated: usize,
        thrust: usize,
    }
    impl VisitMut for Calls {
        fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
            if matches!(&*call.func, Expr::Path(path) if path.path.is_ident("original_rotated_vector"))
            {
                assert_eq!(call.args.len(), 2);
                let value = call.args[1].clone();
                *call.func = syn::parse_quote!(rotate);
                call.args.clear();
                call.args.push(value);
                self.rotated += 1;
            } else if matches!(&*call.func, Expr::Path(path) if path.path.to_token_stream().to_string()=="crate :: original_goto_thrust")
            {
                assert_eq!(call.args.len(), 5);
                assert_eq!(call.args[2].to_token_stream().to_string(), "1_000_000.0");
                assert_eq!(call.args[3].to_token_stream().to_string(), "1.0");
                *call.func = syn::parse_quote!(destiny_original_spec::original_goto_thrust);
                call.args[2] = syn::parse_quote!(friction);
                call.args[3] = syn::parse_quote!(dt);
                self.thrust += 1;
            }
            syn::visit_mut::visit_expr_call_mut(self, call);
        }
    }
    let mut pass = Calls {
        rotated: 0,
        thrust: 0,
    };
    pass.visit_block_mut(&mut function.block);
    if pass.rotated != 1 || pass.thrust != 1 {
        return Err("formation reuse call identity changed".into());
    }
    function.attrs.clear();
    function.sig = syn::parse_quote!(fn formation_thrust<F: Fn(Vec3) -> Vec3>(ball: &mut OriginalParkBall, other: &OriginalParkBall, offset: Vec3, friction: f64, dt: f64, rotate: F) -> Vec3);
    Ok(function.to_token_stream().to_string())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub schema: u32,
    pub reused_source: String,
    pub state_fields: Vec<String>,
    pub roll_parameters: BTreeMap<String, f64>,
}

fn state_fields() -> [&'static str; 9] {
    [
        "yaw",
        "pitch",
        "roll",
        "old_yaw",
        "old_pitch",
        "old_roll",
        "roll_speed",
        "old_roll_speed",
        "yaw_delta",
    ]
}

fn field_name(expression: &Expr) -> Option<String> {
    if let Expr::Field(field) = expression
        && let syn::Member::Named(member) = &field.member
    {
        return Some(member.to_string());
    }
    None
}

struct Constants {
    names: Vec<(&'static str, &'static str)>,
    count: usize,
}

impl VisitMut for Constants {
    fn visit_expr_mut(&mut self, expression: &mut Expr) {
        let spelling = expression.to_token_stream().to_string();
        if let Some((_, target)) = self.names.iter().find(|(source, _)| *source == spelling) {
            let target: syn::Ident = syn::parse_str(target).expect("parameter identifier");
            *expression = syn::parse_quote!(environment.#target);
            self.count += 1;
            return;
        }
        syn::visit_mut::visit_expr_mut(self, expression);
    }
}

struct Parameters {
    roll_speed_assignments: usize,
    roll_assignments: usize,
    mass_bindings: usize,
}

impl VisitMut for Parameters {
    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        if matches!(&local.pat, syn::Pat::Ident(id) if id.ident == "mass_modifier") {
            let mut binding = Constants {
                names: vec![("1_000_000.0", "friction")],
                count: 0,
            };
            binding.visit_local_mut(local);
            self.mass_bindings += binding.count;
        }
        syn::visit_mut::visit_local_mut(self, local);
    }

    fn visit_expr_mut(&mut self, expression: &mut Expr) {
        if let Expr::Binary(binary) = expression
            && matches!(binary.op, syn::BinOp::AddAssign(_))
        {
            let field = field_name(&binary.left);
            let names = match field.as_deref() {
                Some("roll_speed") => {
                    self.roll_speed_assignments += 1;
                    if self.roll_speed_assignments == 1 {
                        vec![("- 3.0", "roll_speed_acceleration")]
                    } else {
                        vec![("- 0.9", "roll_speed_decay"), ("1.0", "dt")]
                    }
                }
                Some("roll") => {
                    self.roll_assignments += 1;
                    if self.roll_assignments == 1 {
                        // The source groups (speed * acceleration) * dt. Bind
                        // the two identical model constants by AST position.
                        let Expr::Binary(outer) = &mut *binary.right else {
                            panic!("roll product shape");
                        };
                        let Expr::Binary(inner) = &mut *outer.left else {
                            panic!("roll product shape");
                        };
                        assert_eq!(inner.right.to_token_stream().to_string(), "1.0");
                        assert_eq!(outer.right.to_token_stream().to_string(), "1.0");
                        *inner.right = syn::parse_quote!(environment.roll_acceleration);
                        *outer.right = syn::parse_quote!(environment.dt);
                        Vec::new()
                    } else {
                        vec![("- 0.5", "roll_decay"), ("1.0", "dt")]
                    }
                }
                _ => Vec::new(),
            };
            if !names.is_empty() {
                let expected = names.len();
                let mut binding = Constants { names, count: 0 };
                binding.visit_expr_mut(&mut binary.right);
                assert_eq!(binding.count, expected, "angular environment binding drift");
            }
        }
        syn::visit_mut::visit_expr_mut(self, expression);
    }
}

struct Bindings {
    removed_client_boundary: usize,
}

struct ExactConstants {
    count: usize,
}

impl VisitMut for ExactConstants {
    fn visit_expr_mut(&mut self, expression: &mut Expr) {
        if let Expr::Lit(literal) = expression
            && let syn::Lit::Float(value) = &literal.lit
        {
            let replacement: Option<(f64, Expr)> = match value.to_string().as_str() {
                "1.57079632679489655" => Some((
                    std::f64::consts::FRAC_PI_2,
                    syn::parse_quote!(std::f64::consts::FRAC_PI_2),
                )),
                "6.2831853071795862" => Some((
                    std::f64::consts::TAU,
                    syn::parse_quote!(std::f64::consts::TAU),
                )),
                _ => None,
            };
            if let Some((constant, replacement)) = replacement {
                assert_eq!(
                    value
                        .base10_parse::<f64>()
                        .expect("source double")
                        .to_bits(),
                    constant.to_bits(),
                    "source constant bits differ"
                );
                *expression = replacement;
                self.count += 1;
                return;
            }
        }
        syn::visit_mut::visit_expr_mut(self, expression);
    }
}

impl VisitMut for Bindings {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        block.stmts.retain(|statement| {
            let remove = matches!(statement, Stmt::Expr(Expr::If(branch), _) if branch.cond.to_token_stream().to_string() == "b . client_ball");
            self.removed_client_boundary += usize::from(remove);
            !remove
        });
        syn::visit_mut::visit_block_mut(self, block);
    }

    fn visit_expr_mut(&mut self, expression: &mut Expr) {
        if let Expr::MethodCall(call) = expression
            && call.method == "is_empty"
            && call.receiver.to_token_stream().to_string() == "b . collisions"
        {
            *expression = syn::parse_quote!(!input.has_collisions);
            return;
        }
        if let Expr::Field(field) = expression
            && let syn::Member::Named(member) = &field.member
        {
            if matches!(&*field.base, Expr::Path(path) if path.path.is_ident("b")) {
                let base: Expr = if state_fields().contains(&member.to_string().as_str()) {
                    syn::parse_quote!(state)
                } else {
                    assert!(
                        [
                            "mass",
                            "velocity",
                            "goto",
                            "position",
                            "mode",
                            "effect_stamp"
                        ]
                        .contains(&member.to_string().as_str()),
                        "unbound angular input"
                    );
                    syn::parse_quote!(input)
                };
                *field.base = base;
            } else if matches!(&*field.base, Expr::Path(path) if path.path.is_ident("self")) {
                match member.to_string().as_str() {
                    "current_time" => *expression = syn::parse_quote!(environment.current_time),
                    "scope_error" => *expression = syn::parse_quote!(scope_error),
                    _ => panic!("unbound angular environment"),
                }
            }
        }
        if let Expr::Call(call) = expression
            && let Expr::Path(path) = &*call.func
        {
            if path.path.to_token_stream().to_string() == "Self :: modulo_2pi" {
                *call.func = syn::parse_quote!(modulo_2pi);
            } else if path.path.to_token_stream().to_string() == "libm :: atan2" {
                assert_eq!(call.args.len(), 2);
                let first = call.args[0].clone();
                let second = call.args[1].clone();
                // Original C atan2 uses the platform math implementation.
                *expression = syn::parse_quote!((#first).atan2(#second));
            }
        }
        if let Expr::Return(value) = expression
            && value.expr.is_none()
        {
            value.expr = Some(Box::new(syn::parse_quote!(scope_error)));
        }
        syn::visit_mut::visit_expr_mut(self, expression);
    }
}

pub fn emit(rules: &Rules) -> Result<String, Box<dyn Error>> {
    if rules.schema != 1 || rules.state_fields != state_fields() {
        return Err("angular state schema differs from reviewed source fields".into());
    }
    let source = syn::parse_file(&fs::read_to_string(&rules.reused_source)?)?;
    let original = super::method(
        &source,
        "OriginalParkProjection",
        "calculate_yaw_pitch_roll",
    )?;
    let mut update = original.clone();
    let mut parameters = Parameters {
        roll_speed_assignments: 0,
        roll_assignments: 0,
        mass_bindings: 0,
    };
    parameters.visit_block_mut(&mut update.block);
    if parameters.roll_speed_assignments != 2
        || parameters.roll_assignments != 2
        || parameters.mass_bindings != 1
    {
        return Err("angular source parameterization changed".into());
    }
    let mut bindings = Bindings {
        removed_client_boundary: 0,
    };
    bindings.visit_block_mut(&mut update.block);
    if bindings.removed_client_boundary != 1 {
        return Err("client presentation boundary changed".into());
    }
    update.attrs.clear();
    update.vis = syn::parse_quote!(pub);
    update.sig = syn::parse_quote!(fn update(state: &mut Angles, input: &Input, environment: &Environment, snap: bool) -> bool);
    update
        .block
        .stmts
        .insert(0, syn::parse_quote!(let mut scope_error = false;));
    update
        .block
        .stmts
        .push(Stmt::Expr(syn::parse_quote!(scope_error), None));
    let mut modulo = super::method(&source, "OriginalParkProjection", "modulo_2pi")?.clone();
    modulo.attrs.clear();
    let mut constants = ExactConstants { count: 0 };
    constants.visit_block_mut(&mut update.block);
    constants.visit_block_mut(&mut modulo.block);
    if constants.count != 2 {
        return Err("source pi constant identity changed".into());
    }
    let mut fields = rust::Tokens::new();
    let mut bit_fields = rust::Tokens::new();
    let mut to_bits = rust::Tokens::new();
    let mut from_bits = rust::Tokens::new();
    for name in &rules.state_fields {
        let name: syn::Ident = syn::parse_str(name)?;
        let name = name.to_string();
        quote_in! { fields => pub $(&name): f64, }
        quote_in! { bit_fields => $(&name): u64, }
        quote_in! { to_bits => $(&name): value.$(&name).to_bits(), }
        quote_in! { from_bits => $(&name): f64::from_bits(value.$(&name)), }
    }
    let mut parameter_fields = rust::Tokens::new();
    let mut defaults = rust::Tokens::new();
    if rules.roll_parameters.len() != 4 {
        return Err("roll parameter inventory differs".into());
    }
    for (name, value) in &rules.roll_parameters {
        let name: syn::Ident = syn::parse_str(name)?;
        let name = name.to_string();
        quote_in! { parameter_fields => pub $(&name): f64, }
        let number = value.to_string();
        let number = if number.contains('.') {
            number
        } else {
            format!("{number}.0")
        };
        quote_in! { defaults => $name: $number, }
    }
    let update = update.to_token_stream().to_string();
    let modulo = modulo.to_token_stream().to_string();
    let formation = formation_kernel()?;
    let output: rust::Tokens = quote! {
        use destiny_original_spec::{OriginalMode, OriginalParkBall, Vec3};
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
        #[serde(from = "AngleBits", into = "AngleBits")]
        pub struct Angles { $fields }

        #[derive(Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct AngleBits { $bit_fields }
        impl From<Angles> for AngleBits {
            fn from(value: Angles) -> Self { Self { $to_bits } }
        }
        impl From<AngleBits> for Angles {
            fn from(value: AngleBits) -> Self { Self { $from_bits } }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct RollParameters { $(&parameter_fields) }
        impl Default for RollParameters {
            fn default() -> Self { Self { $defaults } }
        }

        pub struct Input {
            pub mass: f64,
            pub velocity: Vec3,
            pub position: Vec3,
            pub goto: Vec3,
            pub mode: OriginalMode,
            pub effect_stamp: i64,
            pub has_collisions: bool,
        }
        pub struct Environment {
            pub current_time: i64,
            pub friction: f64,
            pub dt: f64,
            $parameter_fields
        }

        $update
        $modulo
        $formation

        pub fn rotate(state: &Angles, coordinate: Vec3) -> Vec3 {
            destiny_original_spec::original_rotated_vector(&OriginalParkBall {
                yaw: state.yaw, pitch: state.pitch, roll: state.roll, ..OriginalParkBall::default()
            }, coordinate)
        }

        #[cfg(all(test, target_os = "linux", target_arch = "x86_64"))]
        mod tests {
            use super::*;

            #[test]
            fn original_cpp_angular_transitions_match_exact_bits() {
                let report: serde_json::Value = serde_json::from_str(include_str!("../verification/migration/angular_oracle_samples.json")).unwrap();
                let rows = report["rows"].as_array().unwrap();
                assert!(!rows.is_empty());
                for (index, row) in rows.iter().enumerate() {
                    let encoded = row["state_bits"].as_object().unwrap().iter().map(|(key, value)| {
                        (key.clone(), serde_json::json!(u64::from_str_radix(value.as_str().unwrap(), 16).unwrap()))
                    }).collect::<serde_json::Map<_, _>>();
                    let mut state: Angles = serde_json::from_value(serde_json::Value::Object(encoded)).unwrap();
                    let values: Vec<f64> = serde_json::from_value(row["input"].clone()).unwrap();
                    let mode = match row["mode"].as_u64().unwrap() {
                        0 => OriginalMode::Goto, 1 => OriginalMode::Follow, 2 => OriginalMode::Stop,
                        3 => OriginalMode::Warp, 4 => OriginalMode::Orbit, 5 => OriginalMode::Missile,
                        6 => OriginalMode::Mushroom, 7 => OriginalMode::Boid, 8 => OriginalMode::Troll,
                        9 => OriginalMode::Miniball, 10 => OriginalMode::Field, 11 => OriginalMode::Rigid,
                        12 => OriginalMode::Formation, _ => panic!("unknown source mode"),
                    };
                    let input = Input {
                        mass: values[0], velocity: Vec3 { x: values[1], y: values[2], z: values[3] },
                        position: Vec3 { x: values[4], y: values[5], z: values[6] },
                        goto: Vec3 { x: values[7], y: values[8], z: values[9] }, mode,
                        effect_stamp: row["effect_stamp"].as_i64().unwrap(), has_collisions: row["has_collisions"].as_bool().unwrap(),
                    };
                    let environment = Environment {
                        current_time: row["current_time"].as_i64().unwrap(), friction: values[10], dt: values[11],
                        roll_acceleration: values[12], roll_decay: values[13], roll_speed_acceleration: values[14], roll_speed_decay: values[15],
                    };
                    assert!(!update(&mut state, &input, &environment, row["snap"].as_bool().unwrap()));
                    let actual = serde_json::to_value(state).unwrap();
                    for (name, expected) in row["expected_bits"].as_object().unwrap() {
                        assert_eq!(actual[name].as_u64().unwrap(), u64::from_str_radix(expected.as_str().unwrap(), 16).unwrap(), "case {index} field {name}");
                    }
                }
            }

            #[test]
            fn angular_snapshot_preserves_nonfinite_payloads_and_signed_zero() {
                let words = [0x8000000000000000_u64, 0x7ff0000000000000_u64, 0xfff0000000000000_u64,
                             0x7ff8000000001234_u64, 0x7ff0000000004321_u64, 0xfff8000000008765_u64,
                             0x1_u64, 0x10000000000000_u64, 0x7fefffffffffffff_u64];
                let mut encoded = serde_json::to_value(Angles::default()).unwrap();
                for (slot, word) in encoded.as_object_mut().unwrap().values_mut().zip(words) { *slot = serde_json::json!(word); }
                let state: Angles = serde_json::from_value(encoded.clone()).unwrap();
                let restored: Angles = serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
                assert_eq!(serde_json::to_value(restored).unwrap(), encoded);
            }
        }
    };
    let text = format!(
        "// @generated: source-bound angular AST transplantation. Do not edit.\n// Copyright CCP ehf.; MIT. Source and rule hashes: verification/migration/angular_source_evidence.json.\n// Regenerate: python tools/migrate_destiny.py generate\n{}",
        output.to_file_string()?
    );
    syn::parse_file(&text)?;
    Ok(text)
}

struct Runtime<'a> {
    source: &'a str,
    scope: String,
    edits: Vec<(usize, usize, String)>,
    counts: BTreeMap<&'static str, usize>,
}

impl Runtime<'_> {
    fn replace(&mut self, node: &impl syn::spanned::Spanned, text: String, kind: &'static str) {
        let span = node.span();
        self.edits.push((
            super::offset(self.source, span.start()),
            super::offset(self.source, span.end()),
            text,
        ));
        *self.counts.entry(kind).or_default() += 1;
    }

    fn insert(&mut self, point: proc_macro2::LineColumn, text: String, kind: &'static str) {
        let offset = super::offset(self.source, point);
        self.edits.push((offset, offset, text));
        *self.counts.entry(kind).or_default() += 1;
    }
}

impl<'ast> Visit<'ast> for Runtime<'_> {
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        if item.to_token_stream().to_string().contains("EulerRot") {
            fn remove(tree: &mut syn::UseTree) {
                match tree {
                    syn::UseTree::Group(group) => {
                        group.items = group.items.iter().filter(|item| !matches!(item, syn::UseTree::Name(name) if name.ident == "EulerRot")).cloned().collect();
                        for item in &mut group.items {
                            remove(item);
                        }
                    }
                    syn::UseTree::Path(path) => remove(&mut path.tree),
                    _ => (),
                }
            }
            let mut output = item.clone();
            remove(&mut output.tree);
            self.replace(
                item,
                output.to_token_stream().to_string(),
                "remove-euler-import",
            );
        }
    }
    fn visit_impl_item_fn(&mut self, method: &'ast syn::ImplItemFn) {
        let prior = std::mem::replace(&mut self.scope, method.sig.ident.to_string());
        if method.sig.ident == "set_velocity" {
            let mut value = method.clone();
            value.sig.ident = syn::parse_quote!(set_velocity_value);
            for statement in &mut value.block.stmts {
                if let Stmt::Local(local) = statement
                    && matches!(&local.pat, syn::Pat::Ident(id) if id.ident == "planned_rotation")
                {
                    *statement = syn::parse_quote!(let update_orientation = !self.park.use_dynamical_orientation;);
                }
                if let Stmt::Expr(Expr::If(branch), _) = statement
                    && let Expr::Let(condition) = &*branch.cond
                    && condition.expr.to_token_stream().to_string() == "planned_rotation"
                {
                    let (_, other) = branch
                        .else_branch
                        .as_ref()
                        .expect("velocity publication else");
                    *statement = Stmt::Expr((**other).clone(), None);
                }
            }
            struct Planned;
            impl VisitMut for Planned {
                fn visit_expr_mut(&mut self, expr: &mut Expr) {
                    if expr.to_token_stream().to_string() == "planned_rotation . is_some ()" {
                        *expr = syn::parse_quote!(update_orientation);
                        return;
                    }
                    syn::visit_mut::visit_expr_mut(self, expr);
                }
            }
            Planned.visit_block_mut(&mut value.block);
            let mut wrapper = method.clone();
            wrapper.block = syn::parse_quote!({
                self.set_velocity_value(ball_id, velocity)?;
                if !self.park.use_dynamical_orientation {
                    self.update_source_angles(ball_id, true)?;
                }
                Ok(())
            });
            self.replace(
                method,
                format!("{}\n{}", value.to_token_stream(), wrapper.to_token_stream()),
                "velocity-boundary",
            );
        } else {
            if method.sig.ident == "add_ball_with_identity" {
                let last = method.block.stmts.last().expect("add tail");
                self.insert(
                    last.span().start(),
                    "if is_free { self.update_source_angles(id, true)?; }\n".into(),
                    "spawn-snap",
                );
            }
            syn::visit::visit_impl_item_fn(self, method);
        }
        self.scope = prior;
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        let prior = std::mem::replace(&mut self.scope, function.sig.ident.to_string());
        syn::visit::visit_item_fn(self, function);
        self.scope = prior;
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        if let syn::Fields::Named(fields) = &item.fields {
            let name = item.ident.to_string();
            if name == "ParkMetadata" {
                let field = fields
                    .named
                    .iter()
                    .find(|field| {
                        field
                            .ident
                            .as_ref()
                            .is_some_and(|name| name == "network_components")
                    })
                    .expect("network boundary field");
                self.insert(
                    field.span().start(),
                    "#[cfg(feature = \"carbon-network\")] ".into(),
                    "network-field-boundary",
                );
            }
            let field = match name.as_str() {
                "DestinyBallMetadata" | "BallSnapshot" => {
                    Some("#[serde(default)] pub source_angles: crate::generated_angular::Angles,")
                }
                "ParkSnapshotMetadata" => Some(
                    "#[serde(default)] roll_parameters: crate::generated_angular::RollParameters,",
                ),
                "ParkMetadata" => {
                    Some("roll_parameters: crate::generated_angular::RollParameters,")
                }
                _ => None,
            };
            if let Some(field) = field {
                self.insert(
                    fields.brace_token.span.close().start(),
                    field.into(),
                    "state-field",
                );
            }
        }
        syn::visit::visit_item_struct(self, item);
    }

    fn visit_expr_struct(&mut self, expression: &'ast syn::ExprStruct) {
        let Some(segment) = expression.path.segments.last() else {
            return;
        };
        let mut output = expression.clone();
        let field: Option<syn::FieldValue> = match segment.ident.to_string().as_str() {
            "DestinyBallMetadata" => Some(syn::parse_quote!(source_angles: Default::default())),
            "BallSnapshot" if self.scope == "ball_snapshot" => {
                Some(syn::parse_quote!(source_angles: metadata.source_angles))
            }
            "BallSnapshot" => Some(syn::parse_quote!(source_angles: Default::default())),
            "ParkMetadata" => Some(syn::parse_quote!(roll_parameters: Default::default())),
            "ParkSnapshotMetadata" => {
                Some(syn::parse_quote!(roll_parameters: self.park.roll_parameters))
            }
            _ => None,
        };
        if let Some(field) = field {
            if segment.ident == "ParkMetadata" {
                for field in &mut output.fields {
                    if matches!(&field.member, syn::Member::Named(name) if name == "network_components")
                    {
                        field
                            .attrs
                            .push(syn::parse_quote!(#[cfg(feature = "carbon-network")]));
                    }
                }
            }
            output.fields.push(field);
            self.replace(
                expression,
                output.to_token_stream().to_string(),
                "state-initializer",
            );
            return;
        }
        if self.scope == "original_motion_input" && segment.ident == "OriginalParkBall" {
            output
                .fields
                .push(syn::parse_quote!(formation_id: metadata.formation_id as i8));
            for name in state_fields() {
                let field: syn::Ident = syn::parse_str(name).unwrap();
                output
                    .fields
                    .push(syn::parse_quote!(#field: metadata.source_angles.#field));
            }
            if output.rest.is_some() && !output.fields.trailing_punct() {
                output.fields.push_punct(Default::default());
            }
            self.replace(
                expression,
                output.to_token_stream().to_string(),
                "motion-angle-input",
            );
            return;
        }
        syn::visit::visit_expr_struct(self, expression);
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if self.scope == "add_ball_with_identity"
            && matches!(&local.pat, syn::Pat::Ident(id) if id.ident == "initial_rotation")
        {
            self.replace(
                &*local.init.as_ref().expect("rotation initializer").expr,
                "DQuat::IDENTITY".into(),
                "initial-quaternion",
            );
            return;
        }
        syn::visit::visit_local(self, local);
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        if self.scope == "dispatch_ball"
            && arm.pat.to_token_stream().to_string() == "\"GetRotatedVector\""
        {
            let Expr::Block(block) = &*arm.body else {
                panic!("rotation query shape");
            };
            let mut block = block.clone();
            block.block.stmts.retain(|statement| !matches!(statement, Stmt::Local(local) if matches!(&local.pat, syn::Pat::Ident(id) if id.ident == "entity" || id.ident == "rotation")));
            for statement in &mut block.block.stmts {
                if let Stmt::Local(local) = statement
                    && matches!(&local.pat, syn::Pat::Ident(id) if id.ident == "result")
                {
                    *local.init.as_mut().unwrap().expr =
                        syn::parse_quote!(self.source_rotated_vector(ball_id, vector)?);
                }
            }
            self.replace(
                &*arm.body,
                block.to_token_stream().to_string(),
                "rotation-query",
            );
            return;
        }
        if ["get_ball_property", "retained_ball_property"].contains(&self.scope.as_str())
            && arm.pat.to_token_stream().to_string() == "\"roll\" | \"pitch\" | \"yaw\""
        {
            let Expr::Block(block) = &*arm.body else {
                panic!("angle getter shape");
            };
            let mut result = match block.block.stmts.last().unwrap() {
                Stmt::Expr(expr, None) => expr.clone(),
                _ => panic!("angle result shape"),
            };
            struct Getters {
                base: syn::Ident,
            }
            impl VisitMut for Getters {
                fn visit_expr_mut(&mut self, expr: &mut Expr) {
                    if let Expr::Path(path) = expr
                        && ["yaw", "pitch", "roll"]
                            .iter()
                            .any(|name| path.path.is_ident(name))
                    {
                        let field = &path.path.segments[0].ident;
                        let base = &self.base;
                        *expr = syn::parse_quote!(#base.source_angles.#field);
                        return;
                    }
                    syn::visit_mut::visit_expr_mut(self, expr);
                }
            }
            Getters {
                base: syn::parse_str(if self.scope == "get_ball_property" {
                    "metadata"
                } else {
                    "snapshot"
                })
                .unwrap(),
            }
            .visit_expr_mut(&mut result);
            let result: Expr = syn::parse_quote!({ #result });
            self.replace(
                &*arm.body,
                result.to_token_stream().to_string(),
                "angle-getter",
            );
            return;
        }
        syn::visit::visit_arm(self, arm);
    }

    fn visit_expr_macro(&mut self, expression: &'ast syn::ExprMacro) {
        if self.scope == "prepare_motion_tick"
            && expression.mac.path.is_ident("matches")
            && expression.mac.tokens.to_string()
                == "control . mode , OriginalMode :: Follow | OriginalMode :: Orbit"
        {
            let output: Expr = syn::parse_quote!(matches!(
                control.mode,
                OriginalMode::Follow | OriginalMode::Orbit | OriginalMode::Formation
            ));
            self.replace(
                expression,
                output.to_token_stream().to_string(),
                "formation-dispatch",
            );
            return;
        }
        syn::visit::visit_expr_macro(self, expression);
    }

    fn visit_expr_if(&mut self, branch: &'ast syn::ExprIf) {
        if self.scope == "prepare_motion_tick"
            && branch.cond.to_token_stream().to_string()
                == "control . mode == OriginalMode :: Follow"
        {
            self.insert(branch.span().start(), "if control.mode == OriginalMode::Formation { self.source_formation_thrust(&mut ball, &other)? } else ".into(), "formation-thrust-binding");
        }
        syn::visit::visit_expr_if(self, branch);
    }

    fn visit_expr_match(&mut self, expression: &'ast syn::ExprMatch) {
        if self.scope == "dispatch_park" {
            for arm in &expression.arms {
                if arm.pat.to_token_stream().to_string() != "\"friction\"" {
                    continue;
                }
                let getter = matches!(&*arm.body, Expr::Call(_));
                let mut additions = String::new();
                for (label, field) in [
                    ("rollAcceleration", "roll_acceleration"),
                    ("rollDecay", "roll_decay"),
                    ("rollSpeedAcceleration", "roll_speed_acceleration"),
                    ("rollSpeedDecay", "roll_speed_decay"),
                ] {
                    let field: syn::Ident = syn::parse_str(field).unwrap();
                    let next: syn::Arm = if getter {
                        syn::parse_quote!(#label => json_number(self.park.roll_parameters.#field),)
                    } else {
                        syn::parse_quote!(#label => { self.park.roll_parameters.#field = value_f64(args.first(), name)?; },)
                    };
                    additions.push_str(&next.to_token_stream().to_string());
                    additions.push('\n');
                }
                self.insert(
                    arm.span().start(),
                    additions,
                    if getter {
                        "roll-getters"
                    } else {
                        "roll-setters"
                    },
                );
            }
        }
        syn::visit::visit_expr_match(self, expression);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if [
            "prepare_motion_tick",
            "advance_one_tick_with_motion",
            "add_ball_with_identity",
        ]
        .contains(&self.scope.as_str())
            && call.method == "set_velocity"
        {
            let mut output = call.clone();
            output.method = syn::parse_quote!(set_velocity_value);
            self.replace(
                call,
                output.to_token_stream().to_string(),
                "internal-velocity-publication",
            );
            return;
        }
        if self.scope == "advance_one_tick_with_motion"
            && call.method == "and_then"
            && call.args.len() == 1
            && let Expr::Closure(closure) = &call.args[0]
            && closure.body.to_token_stream().to_string()
                == "self . validate_authoritative_world (false)"
        {
            let result = &closure.body;
            let output: Expr = syn::parse_quote!({self.update_all_source_angles()?; #result});
            self.replace(
                &*closure.body,
                output.to_token_stream().to_string(),
                "tick-angle-phase",
            );
            self.visit_expr(&call.receiver);
            return;
        }
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_stmt(&mut self, statement: &'ast Stmt) {
        if self.scope == "add_ball_with_identity"
            && matches!(statement, Stmt::Expr(Expr::Return(value), _) if value.expr.as_ref().is_some_and(|value| value.to_token_stream().to_string().starts_with("Ok (format !")))
        {
            self.insert(
                statement.span().start(),
                "if is_free { self.update_source_angles(id, true)?; }\n".into(),
                "replacement-snap",
            );
        }
        if let Stmt::Expr(Expr::Try(attempt), _) = statement
            && let Expr::Call(call) = &*attempt.expr
            && matches!(&*call.func, Expr::Path(path) if path.path.is_ident("validate_formations"))
            && call.args.to_token_stream().to_string() == "& snapshot . park . formations"
        {
            let validation: Stmt = syn::parse_quote! {
                if [snapshot.park.roll_parameters.roll_acceleration, snapshot.park.roll_parameters.roll_decay,
                    snapshot.park.roll_parameters.roll_speed_acceleration, snapshot.park.roll_parameters.roll_speed_decay]
                    .iter().any(|value| !value.is_finite()) {
                    return Err(CompatError::InvalidRequest("non-finite snapshot roll parameter".into()));
                }
            };
            self.insert(
                statement.span().start(),
                validation.to_token_stream().to_string(),
                "roll-snapshot-validation",
            );
        }
        if let Stmt::Expr(Expr::Assign(assign), _) = statement {
            let left = assign.left.to_token_stream().to_string();
            let right = assign.right.to_token_stream().to_string();
            if left == "metadata . angular_agility" && right == "ball . angular_agility" {
                self.insert(
                    statement.span().end(),
                    "\nmetadata.source_angles = ball.source_angles;".into(),
                    "restore-angles",
                );
            }
            if left == "self . park . use_new_orbit" && right == "snapshot . park . use_new_orbit" {
                self.insert(
                    statement.span().end(),
                    "\nself.park.roll_parameters = snapshot.park.roll_parameters;".into(),
                    "restore-roll-environment",
                );
            }
        }
        syn::visit::visit_stmt(self, statement);
    }
}

pub fn runtime(source: &str) -> Result<String, Box<dyn Error>> {
    let file = syn::parse_file(source)?;
    let mut pass = Runtime {
        source,
        scope: String::new(),
        edits: Vec::new(),
        counts: BTreeMap::new(),
    };
    pass.visit_file(&file);
    for (kind, expected) in [
        ("velocity-boundary", 1),
        ("state-field", 4),
        ("motion-angle-input", 1),
        ("spawn-snap", 1),
        ("angle-getter", 2),
        ("replacement-snap", 1),
        ("internal-velocity-publication", 3),
        ("tick-angle-phase", 1),
        ("restore-angles", 1),
        ("restore-roll-environment", 1),
        ("initial-quaternion", 1),
        ("remove-euler-import", 1),
        ("rotation-query", 1),
        ("formation-dispatch", 1),
        ("formation-thrust-binding", 1),
    ] {
        if pass.counts.get(kind) != Some(&expected) {
            return Err(format!(
                "angular wiring {kind} site count differs: {:?}",
                pass.counts
            )
            .into());
        }
    }
    pass.edits.sort_by_key(|edit| (edit.0, edit.1));
    if pass.edits.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Err("overlapping angular transformations".into());
    }
    let mut output = source.to_owned();
    for (start, end, replacement) in pass.edits.into_iter().rev() {
        output.replace_range(start..end, &replacement);
    }
    let glue: rust::Tokens = quote! {
        impl CompatRuntime {
            fn source_rotation(&self, id: i64) -> Result<impl Fn(OriginalVec3) -> OriginalVec3, CompatError> {
                let entity = self.entity(id)?;
                let rotation = self.app.world().get::<Rotation>(entity).ok_or_else(|| CompatError::Engine("missing Rotation".into()))?.0;
                let angles = self.metadata(id)?.source_angles;
                let dynamic = self.park.use_dynamical_orientation;
                Ok(move |vector: OriginalVec3| {
                    if dynamic {
                        let result = rotation * DVec3::new(vector.x, vector.y, vector.z);
                        OriginalVec3 { x: result.x, y: result.y, z: result.z }
                    } else { crate::generated_angular::rotate(&angles, vector) }
                })
            }

            fn source_rotated_vector(&self, id: i64, vector: DVec3) -> Result<DVec3, CompatError> {
                let value = (self.source_rotation(id)?)(OriginalVec3 { x: vector.x, y: vector.y, z: vector.z });
                Ok(DVec3::new(value.x, value.y, value.z))
            }

            fn source_formation_thrust(&self, ball: &mut OriginalParkBall, other: &OriginalParkBall) -> Result<OriginalVec3, CompatError> {
                let Some(points) = self.park.formations.get(other.formation_id as usize) else { return Ok(OriginalVec3::ZERO); };
                let slot = ball.effect_stamp as i8;
                if slot < 0 { return Err(CompatError::InvalidRequest("negative source signed-char formation index is undefined".into())); }
                let Some(offset) = points.get(slot as usize) else { return Ok(OriginalVec3::ZERO); };
                let offset = OriginalVec3 { x: offset[0], y: offset[1], z: offset[2] };
                let friction = self.app.world().resource::<DestinySpaceFriction>().0;
                let dt = self.app.world().resource::<Time<Fixed>>().timestep().as_secs_f64();
                Ok(crate::generated_angular::formation_thrust(ball, other, offset, friction, dt, self.source_rotation(other.id)?))
            }

            fn update_source_angles(&mut self, id: i64, snap: bool) -> Result<(), CompatError> {
                if self.park.use_dynamical_orientation { return Ok(()); }
                let control = self.motion.get(&id).cloned().unwrap_or_default();
                let ball = self.original_motion_input(id, &control)?;
                let entity = self.entity(id)?;
                let input = crate::generated_angular::Input {
                    mass: ball.mass, velocity: ball.velocity, position: ball.position,
                    goto: ball.goto, mode: ball.mode, effect_stamp: ball.effect_stamp,
                    has_collisions: self.app.world().get::<CollidingEntities>(entity).is_some_and(|contacts| !contacts.is_empty()),
                };
                let parameters = self.park.roll_parameters;
                let environment = crate::generated_angular::Environment {
                    current_time: self.park.current_time,
                    friction: self.app.world().resource::<DestinySpaceFriction>().0,
                    dt: self.app.world().resource::<Time<Fixed>>().timestep().as_secs_f64(),
                    roll_acceleration: parameters.roll_acceleration, roll_decay: parameters.roll_decay,
                    roll_speed_acceleration: parameters.roll_speed_acceleration, roll_speed_decay: parameters.roll_speed_decay,
                };
                let mut state = self.metadata(id)?.source_angles;
                if crate::generated_angular::update(&mut state, &input, &environment, snap) {
                    return Err(CompatError::InvalidRequest("angular timestamp arithmetic overflow".into()));
                }
                self.metadata_mut(id)?.source_angles = state;
                Ok(())
            }

            fn update_all_source_angles(&mut self) -> Result<(), CompatError> {
                let mut ids = self.balls.keys().copied().collect::<Vec<_>>(); ids.sort_unstable();
                for id in ids {
                    if !self.park.pending_removals.contains_key(&id) && self.metadata(id)?.is_free
                        && !self.motion.get(&id).is_some_and(|control| control.mode == OriginalMode::Mushroom) {
                        self.update_source_angles(id, false)?;
                    }
                }
                Ok(())
            }
        }
    };
    let transformed = syn::parse_file(&output)?;
    let test_module = transformed
        .items
        .iter()
        .find(|item| matches!(item, syn::Item::Mod(module) if module.ident == "tests"))
        .ok_or("missing runtime test boundary")?;
    let insertion = super::offset(&output, test_module.span().start());
    output.insert_str(
        insertion,
        &format!(
            "// Generated state-access wiring; semantics are in generated_angular.\n{}\n",
            glue.to_file_string()?
        ),
    );
    if let Err(error) = syn::parse_file(&output) {
        let line = error.span().start().line;
        let context = output
            .lines()
            .skip(line.saturating_sub(2))
            .take(3)
            .collect::<Vec<_>>()
            .join("\n");
        return Err(
            format!("angular output parse error on line {line}: {error}: {context}").into(),
        );
    }
    Ok(output)
}
