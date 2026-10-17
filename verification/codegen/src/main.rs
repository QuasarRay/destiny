//! Restricted semantic emission and span-preserving Rust transformations.
//! Domain expressions are input data, never target-language text.
use genco::prelude::*;
use quote::ToTokens;
use serde::Deserialize;
use std::{collections::BTreeSet, env, error::Error, fs, path::Path};
use syn::{Expr, ImplItem, Item, visit::Visit, visit_mut::VisitMut};

mod angular;
mod contacts;
mod forces;
mod kinematics;
mod liveness;
mod missile;
mod partition;
mod speed;
mod state_rules;
mod warp;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
enum Expression {
    #[serde(rename = "field")]
    Field { name: String, machine_type: String },
    #[serde(rename = "binary")]
    Binary {
        operator: String,
        machine_type: String,
        left: Box<Self>,
        right: Box<Self>,
    },
    #[serde(rename = "sqrt")]
    Sqrt {
        machine_type: String,
        value: Box<Self>,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Function {
    original_symbol: String,
    name: String,
    return_type: String,
    expression: Expression,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Unit {
    schema: u32,
    source_sha256: String,
    original_revisions: Vec<String>,
    extractor_sha256: String,
    configuration_sha256: String,
    rule_version: String,
    generator_sha256: String,
    functions: Vec<Function>,
    reuse: Option<Reuse>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reuse {
    type_path: String,
    method: String,
    sources: Vec<String>,
    fields: std::collections::BTreeMap<String, usize>,
}

fn rust_expression(
    expression: &Expr,
    methods: &std::collections::BTreeMap<String, Expr>,
    depth: usize,
) -> Result<Expression, Box<dyn Error>> {
    if depth > 8 {
        return Err("recursive reuse expression".into());
    }
    let lower = |expr| rust_expression(expr, methods, depth + 1);
    Ok(match expression {
        Expr::Paren(expr) => lower(&expr.expr)?,
        Expr::Field(field) if matches!(&*field.base, Expr::Path(path) if path.path.is_ident("self")) =>
        {
            let syn::Member::Named(name) = &field.member else {
                return Err("unnamed reused field".into());
            };
            Expression::Field {
                name: name.to_string(),
                machine_type: "double".into(),
            }
        }
        Expr::Binary(binary) => {
            let operator = match binary.op {
                syn::BinOp::Add(_) => "+",
                syn::BinOp::Sub(_) => "-",
                syn::BinOp::Mul(_) => "*",
                syn::BinOp::Div(_) => "/",
                _ => return Err("unsupported reused binary operation".into()),
            };
            Expression::Binary {
                operator: operator.into(),
                machine_type: "double".into(),
                left: Box::new(lower(&binary.left)?),
                right: Box::new(lower(&binary.right)?),
            }
        }
        Expr::MethodCall(call) if call.args.is_empty() => {
            if call.method == "sqrt" {
                Expression::Sqrt {
                    machine_type: "double".into(),
                    value: Box::new(lower(&call.receiver)?),
                }
            } else if matches!(&*call.receiver, Expr::Path(path) if path.path.is_ident("self")) {
                lower(
                    methods
                        .get(&call.method.to_string())
                        .ok_or("unresolved reused method")?,
                )?
            } else {
                return Err("unresolved reused receiver".into());
            }
        }
        _ => return Err("unsupported reused expression".into()),
    })
}

fn reused_expression(reuse: &Reuse) -> Result<Expression, Box<dyn Error>> {
    let path: syn::Path = syn::parse_str(&reuse.type_path)?;
    let type_name = &path.segments.last().ok_or("empty reused path")?.ident;
    let mut methods = std::collections::BTreeMap::new();
    for source in &reuse.sources {
        let file = syn::parse_file(&fs::read_to_string(source)?)?;
        for item in file.items {
            if let Item::Impl(implementation) = item {
                if !matches!(&*implementation.self_ty, syn::Type::Path(p) if p.path.is_ident(type_name))
                {
                    continue;
                }
                for member in implementation.items {
                    if let ImplItem::Fn(method) = member {
                        if method.sig.inputs.len() != 1 || method.block.stmts.len() != 1 {
                            continue;
                        }
                        if let syn::Stmt::Expr(expression, None) = &method.block.stmts[0]
                            && methods
                                .insert(method.sig.ident.to_string(), expression.clone())
                                .is_some()
                        {
                            return Err("ambiguous reused method".into());
                        }
                    }
                }
            }
        }
    }
    rust_expression(
        methods.get(&reuse.method).ok_or("missing reused method")?,
        &methods,
        0,
    )
}

fn emit_reuse(reuse: &Reuse, original: &Expression) -> Result<rust::Tokens, Box<dyn Error>> {
    if reused_expression(reuse)? != *original {
        return Err("existing Rust arithmetic differs from compiler-extracted source".into());
    }
    let path: syn::Path = syn::parse_str(&reuse.type_path)?;
    let type_path = path.to_token_stream().to_string();
    let method: syn::Ident = syn::parse_str(&reuse.method)?;
    let method = method.to_string();
    let mut fields = rust::Tokens::new();
    for (field, index) in &reuse.fields {
        let field: syn::Ident = syn::parse_str(field)?;
        if *index > 2 {
            return Err("unbound reused representation index".into());
        }
        let name = field.to_string();
        let index = *index;
        quote_in! { fields => $name: vector[$index], }
    }
    Ok(quote! { $type_path { $fields }.$method() })
}

fn emit_expr(expr: &Expression) -> Result<rust::Tokens, Box<dyn Error>> {
    Ok(match expr {
        Expression::Field { name, machine_type } => {
            if machine_type != "double" {
                return Err("unsupported field machine type".into());
            }
            let index = match name.as_str() {
                "x" => 0,
                "y" => 1,
                "z" => 2,
                _ => return Err("unbound source member".into()),
            };
            quote! { vector[$index] }
        }
        Expression::Binary {
            operator,
            machine_type,
            left,
            right,
        } => {
            if machine_type != "double" {
                return Err("unsupported arithmetic machine type".into());
            }
            let lhs = emit_expr(left)?;
            let rhs = emit_expr(right)?;
            match operator.as_str() {
                "+" => quote! { ($lhs + $rhs) },
                "-" => quote! { ($lhs - $rhs) },
                "*" => quote! { ($lhs * $rhs) },
                "/" => quote! { ($lhs / $rhs) },
                _ => return Err("unsupported source binary operator".into()),
            }
        }
        Expression::Sqrt {
            machine_type,
            value,
        } => {
            if machine_type != "double" {
                return Err("unsupported sqrt machine type".into());
            }
            let value = emit_expr(value)?;
            quote! { ($value).sqrt() }
        }
    })
}

fn emit(unit: &Unit) -> Result<String, Box<dyn Error>> {
    if unit.schema != 1 || unit.rule_version != "scalar-expression-v1" {
        return Err("unsupported semantic schema or rule version".into());
    }
    let mut names = BTreeSet::new();
    let mut output = rust::Tokens::new();
    for function in &unit.functions {
        let ident: syn::Ident = syn::parse_str(&function.name)?;
        if function.return_type != "double" || !names.insert(function.name.clone()) {
            return Err("invalid or repeated function signature".into());
        }
        let name = ident.to_string();
        let expression = match &unit.reuse {
            Some(reuse) => emit_reuse(reuse, &function.expression)?,
            None => emit_expr(&function.expression)?,
        };
        quote_in! { output =>
            pub(crate) fn $name(vector: [f64; 3]) -> f64 {
                #[cfg(kani)]
                kani::cover!(true, "actual generated geometry adapter executed");
                $expression
            }
        }
    }
    let symbols = unit
        .functions
        .iter()
        .map(|f| f.original_symbol.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let text = format!(
        "// @generated; do not edit. Original Destiny Copyright CCP ehf.; MIT.\n\
         // Source SHA-256: {}\n// Original revisions: {}\n// Symbols: {}\n\
         // Extractor SHA-256: {}\n// Generator SHA-256: {}\n// Configuration SHA-256: {}\n\
         // Rule: {}\n// Regenerate: python tools/migrate_destiny.py generate\n{}",
        unit.source_sha256,
        unit.original_revisions.join(", "),
        symbols,
        unit.extractor_sha256,
        unit.generator_sha256,
        unit.configuration_sha256,
        unit.rule_version,
        output.to_file_string()?
    );
    syn::parse_file(&text)?;
    Ok(text)
}

fn emit_harness(input: &str, unit: &Unit) -> Result<String, Box<dyn Error>> {
    let report: serde_json::Value = serde_json::from_str(input)?;
    if report["status"] != "PASSED_SCOPED" || report["proof_credit"] != false {
        return Err(
            "harness requires successful independent original-oracle runtime observations".into(),
        );
    }
    let mut seen = BTreeSet::new();
    let mut assertions = rust::Tokens::new();
    for row in report["rows"].as_array().ok_or("missing oracle rows")? {
        if row["operation"] != "GetCenterDist" || seen.len() == 7 {
            continue;
        }
        let bits = row["input_bits"].as_array().ok_or("missing vector bits")?;
        if bits.len() != 3 {
            return Err("oracle input arity differs".into());
        }
        let words = bits
            .iter()
            .map(|word| {
                u64::from_str_radix(word.as_str().ok_or("non-string input bits")?, 16)
                    .map_err(Into::into)
            })
            .collect::<Result<Vec<u64>, Box<dyn Error>>>()?;
        if !seen.insert(words.clone()) {
            continue;
        }
        let mut lanes = rust::Tokens::new();
        for word in words {
            quote_in! { lanes => f64::from_bits($word), }
        }
        let expected = u64::from_str_radix(
            row["expected_bits"]
                .as_str()
                .ok_or("missing expected bits")?,
            16,
        )?;
        quote_in! { assertions =>
            assert_eq!(production::vector_length([$lanes]).to_bits(), $expected);
        }
    }
    if seen.len() != 7 {
        return Err("incomplete independent oracle selection".into());
    }
    let output: rust::Tokens = quote! {
        #[path = "../../../src/generated_geometry.rs"]
        mod production;

        #[kani::proof]
        fn original_geometry_oracle_samples() {
            $assertions
        }
    };
    Ok(format!(
        "// @generated by destiny-semantic-codegen; independent C++ oracle inputs/results.\n\
         // Source SHA-256: {}\n// Generator SHA-256: {}\n// Configuration SHA-256: {}\n\
         // Oracle samples SHA-256: {}\n// Rule: scalar-oracle-harness-v1\n\
         // Regenerate: python tools/migrate_destiny.py generate\n{}",
        unit.source_sha256,
        unit.generator_sha256,
        unit.configuration_sha256,
        report["samples_sha256"]
            .as_str()
            .ok_or("missing oracle sample digest")?,
        output.to_file_string()?
    ))
}

struct Calls<'a> {
    source: &'a str,
    edits: Vec<(usize, usize, String)>,
    selected: bool,
    match_selected: bool,
}

fn offset(source: &str, location: proc_macro2::LineColumn) -> usize {
    source
        .split_inclusive('\n')
        .take(location.line - 1)
        .map(str::len)
        .sum::<usize>()
        + location.column
}

impl<'ast> Visit<'ast> for Calls<'_> {
    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        let prior = self.selected;
        if let ImplItem::Fn(method) = item {
            self.selected =
                method.sig.ident == "get_ball_property" || method.sig.ident == "surface_distance";
        }
        syn::visit::visit_impl_item(self, item);
        self.selected = prior;
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let prior = self.match_selected;
        self.match_selected = matches!(&arm.pat, syn::Pat::Lit(lit) if matches!(&lit.lit, syn::Lit::Str(s) if s.value() == "GetCenterDist"));
        syn::visit::visit_arm(self, arm);
        self.match_selected = prior;
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        use syn::spanned::Spanned;
        if (self.selected || self.match_selected)
            && let Expr::Call(call) = expr
            && matches!(&*call.func, Expr::Path(path) if path.path.is_ident("stable_vec3_length"))
        {
            if call.args.len() != 1 {
                panic!("unreviewed vector conversion arity");
            }
            let argument = call.args.first().unwrap();
            let target: Expr = syn::parse_quote! {
                crate::generated_geometry::vector_length((#argument).to_array())
            };
            self.edits.push((
                offset(self.source, expr.span().start()),
                offset(self.source, expr.span().end()),
                target.to_token_stream().to_string(),
            ));
        }
        syn::visit::visit_expr(self, expr);
    }
}

fn transform(source: &str) -> Result<String, Box<dyn Error>> {
    let ast = syn::parse_file(source)?;
    let mut calls = Calls {
        source,
        edits: Vec::new(),
        selected: false,
        match_selected: false,
    };
    calls.visit_file(&ast);
    if calls.edits.is_empty() {
        let count = source
            .matches("crate::generated_geometry::vector_length")
            .count();
        if count != 3 {
            return Err("expected three previously transformed distance call sites".into());
        }
        return Ok(source.to_owned());
    }
    if calls.edits.len() != 3 {
        return Err("expected exactly three source-backed distance call sites".into());
    }
    calls.edits.sort_by_key(|edit| edit.0);
    let mut result = source.to_owned();
    for (start, end, replacement) in calls.edits.into_iter().rev() {
        result.replace_range(start..end, &replacement);
    }
    syn::parse_file(&result)?;
    Ok(result)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GuardRule {
    schema: u32,
    kind: String,
    reused_source: String,
    reused_owner: String,
    reused_method: String,
}

fn method<'a>(
    file: &'a syn::File,
    owner: &str,
    name: &str,
) -> Result<&'a syn::ImplItemFn, Box<dyn Error>> {
    let mut matches = Vec::new();
    for item in &file.items {
        if let Item::Impl(implementation) = item
            && matches!(&*implementation.self_ty, syn::Type::Path(path) if path.path.is_ident(owner))
        {
            for member in &implementation.items {
                if let ImplItem::Fn(function) = member
                    && function.sig.ident == name
                {
                    matches.push(function);
                }
            }
        }
    }
    if matches.len() != 1 {
        return Err("missing or ambiguous rule method".into());
    }
    Ok(matches[0])
}

fn equal_tokens(left: &impl ToTokens, right: &impl ToTokens) -> bool {
    left.to_token_stream().to_string() == right.to_token_stream().to_string()
}

fn reused_uncloak_guard(source: &str, rule: &GuardRule) -> Result<Expr, Box<dyn Error>> {
    let file = syn::parse_file(source)?;
    let function = method(&file, &rule.reused_owner, &rule.reused_method)?;
    let assignment: syn::Stmt = syn::parse_quote! { b.is_massive = true; };
    let mut conditions = Vec::new();
    for statement in &function.block.stmts {
        if let syn::Stmt::Expr(Expr::If(branch), _) = statement
            && branch
                .then_branch
                .stmts
                .first()
                .is_some_and(|first| equal_tokens(first, &assignment))
        {
            if branch.else_branch.is_some() {
                return Err("unreviewed uncloak else branch".into());
            }
            conditions.push((*branch.cond).clone());
        }
    }
    if conditions.len() != 1 {
        return Err("missing or ambiguous uncloak guard".into());
    }
    Ok(conditions.remove(0))
}

struct BindGuard {
    mode_adapter: Expr,
    mode_count: usize,
    stamp_count: usize,
}

impl VisitMut for BindGuard {
    fn visit_expr_mut(&mut self, expression: &mut Expr) {
        if let Expr::Field(field) = expression
            && matches!(&*field.base, Expr::Path(path) if path.path.is_ident("b"))
        {
            if field.member == syn::Member::Named(syn::parse_quote!(mode)) {
                *expression = self.mode_adapter.clone();
                self.mode_count += 1;
                return;
            }
            if field.member == syn::Member::Named(syn::parse_quote!(effect_stamp)) {
                *expression = syn::parse_quote!(self.metadata(ball_id)?.effect_stamp);
                self.stamp_count += 1;
                return;
            }
        }
        syn::visit_mut::visit_expr_mut(self, expression);
    }
}

fn guard_uncloak(source: &str, rule: &GuardRule) -> Result<String, Box<dyn Error>> {
    use syn::spanned::Spanned;
    if rule.schema != 1 || rule.kind != "reuse-uncloak-state-guard-v1" {
        return Err("unsupported state guard rule".into());
    }
    let mut condition = reused_uncloak_guard(&fs::read_to_string(&rule.reused_source)?, rule)?;
    let file = syn::parse_file(source)?;
    // Reuse the existing public mode lookup, including its absent-control default.
    let mode_adapter: Expr = syn::parse_quote! {
        self.motion.get(&ball_id).map_or(OriginalMode::Stop, |control| control.mode)
    };
    let getter = method(&file, "CompatRuntime", "get_ball_property")?;
    if !getter
        .to_token_stream()
        .to_string()
        .contains(&mode_adapter.to_token_stream().to_string())
    {
        return Err("existing mode representation adapter changed".into());
    }
    let mut binding = BindGuard {
        mode_adapter,
        mode_count: 0,
        stamp_count: 0,
    };
    binding.visit_expr_mut(&mut condition);
    if binding.mode_count != 1 || binding.stamp_count != 1 {
        return Err("unreviewed reused guard representation".into());
    }
    let target = method(&file, "CompatRuntime", "uncloak_ball")?;
    let massive: syn::Stmt = syn::parse_quote! { self.set_massive(ball_id, true)?; };
    let sensors: syn::Stmt = syn::parse_quote! { self.remove_from_sensor_members(ball_id)?; };
    let mut regions = Vec::new();
    for (index, pair) in target.block.stmts.windows(2).enumerate() {
        if equal_tokens(&pair[0], &massive) && equal_tokens(&pair[1], &sensors) {
            if index == 0 {
                return Err("uncloak prelude missing".into());
            }
            let prior_end = target.block.stmts[index - 1].span().end();
            regions.push((prior_end, pair[1].span().end(), pair.to_vec()));
        }
    }
    if regions.len() != 1 {
        return Err("uncloak mutation region missing or already guarded".into());
    }
    let (start, end, statements) = regions.remove(0);
    let guarded: Expr = syn::parse_quote! { if #condition { #(#statements)* } };
    let replacement = format!(
        "\n\n// Source-bound guard reused from OriginalParkProjection::uncloak_ball.\n{}",
        guarded.to_token_stream()
    );
    let mut output = source.to_owned();
    output.replace_range(offset(source, start)..offset(source, end), &replacement);
    syn::parse_file(&output)?;
    Ok(output)
}

fn module(source: &str) -> Result<String, Box<dyn Error>> {
    let ast = syn::parse_file(source)?;
    if ast
        .items
        .iter()
        .any(|item| matches!(item, Item::Mod(m) if m.ident == "generated_geometry"))
    {
        return Ok(source.to_owned());
    }
    let mut result = source.to_owned();
    result.push_str("\nmod generated_geometry;\n");
    syn::parse_file(&result)?;
    Ok(result)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [mode, input, output] if mode == "kinematic-mutate-acceleration" => {
            fs::write(
                output,
                kinematics::mutate_acceleration(&fs::read_to_string(input)?)?,
            )?;
        }
        [mode, input, output] if mode == "kinematic-mutate-projection" => {
            fs::write(
                output,
                kinematics::mutate_projection(&fs::read_to_string(input)?)?,
            )?;
        }
        [mode, output] if mode == "kinematic-emit" => {
            fs::write(output, kinematics::emit()?)?;
        }
        [mode, input, output] if mode == "kinematic-runtime" => {
            fs::write(output, kinematics::transform(&fs::read_to_string(input)?)?)?;
        }
        [mode, output] if mode == "contact-emit" => {
            fs::write(output, contacts::emit()?)?;
        }
        [mode, input, output] if mode.starts_with("contact-") => {
            let stage = match mode.as_str() {
                "contact-runtime" => "runtime",
                "contact-gradient" => "gradient",
                "contact-iterative" => "iterative",
                "contact-frame" => "frame",
                "contact-iterative-fixture" => "iterative-fixture",
                "contact-gradient-fixture" => "gradient-fixture",
                _ => return Err("unknown contact mode".into()),
            };
            fs::write(
                output,
                contacts::transform(&fs::read_to_string(input)?, stage)?,
            )?;
        }
        [mode, input, output]
            if ["force-runtime", "force-gradient", "force-fixture"].contains(&mode.as_str()) =>
        {
            let stage = match mode.as_str() {
                "force-runtime" => "runtime",
                "force-gradient" => "gradient",
                _ => "fixture",
            };
            fs::write(
                output,
                forces::transform(&fs::read_to_string(input)?, stage)?,
            )?;
        }
        [mode, output] if mode == "force-emit" => {
            fs::write(output, forces::emit()?)?;
        }

        [mode, input, output] if mode == "liveness-runtime" => {
            fs::write(
                output,
                liveness::transform(&fs::read_to_string(input)?, "runtime")?,
            )?;
        }
        [mode, input, output] if mode == "liveness-bubbles" => {
            fs::write(
                output,
                liveness::transform(&fs::read_to_string(input)?, "bubbles")?,
            )?;
        }
        [mode, input, output] if mode == "liveness-iterative" => {
            fs::write(
                output,
                liveness::transform(&fs::read_to_string(input)?, "iterative")?,
            )?;
        }
        [mode, input, output] if mode == "liveness-noniterative" => {
            fs::write(
                output,
                liveness::transform(&fs::read_to_string(input)?, "noniterative")?,
            )?;
        }
        [mode, input, output] if mode == "liveness-fixture" => {
            fs::write(
                output,
                liveness::transform(&fs::read_to_string(input)?, "fixture")?,
            )?;
        }
        [mode, output] if mode == "liveness-emit" => {
            fs::write(output, liveness::emit()?)?;
        }
        [mode, input, output] if mode == "missile-runtime" => {
            fs::write(output, missile::runtime(&fs::read_to_string(input)?)?)?;
        }
        [mode, output] if mode == "missile-emit" => {
            fs::write(output, missile::emit()?)?;
        }
        [mode, input, output] if mode == "partition-runtime" => {
            fs::write(output, partition::runtime(&fs::read_to_string(input)?)?)?;
        }
        [mode, input, output] if mode == "partition-bubbles" => {
            fs::write(output, partition::bubbles(&fs::read_to_string(input)?)?)?;
        }
        [mode, output] if mode == "partition-emit" => {
            fs::write(output, partition::emit()?)?;
        }
        [mode, input, output] if mode == "partition-spec" => {
            fs::write(
                output,
                partition::specification(&fs::read_to_string(input)?)?,
            )?;
        }
        [mode, input, output] if mode == "partition-spec-lib" => {
            fs::write(
                output,
                partition::specification_library(&fs::read_to_string(input)?)?,
            )?;
        }
        [mode, input, output] if mode == "warp-runtime" => {
            fs::write(output, warp::runtime(&fs::read_to_string(input)?)?)?;
        }
        [mode, input, output] if mode == "speed-runtime" => {
            fs::write(output, speed::runtime(&fs::read_to_string(input)?)?)?;
        }
        [mode, input, output] if mode == "speed-network" => {
            fs::write(output, speed::network(&fs::read_to_string(input)?)?)?;
        }
        [mode, input, output] if mode == "angular-runtime" => {
            fs::write(output, angular::runtime(&fs::read_to_string(input)?)?)?;
        }
        [mode, rules, output] if mode == "angular-emit" => {
            let rules: angular::Rules = serde_json::from_str(&fs::read_to_string(rules)?)?;
            fs::write(output, angular::emit(&rules)?)?;
        }
        [mode, input, rules, output] if mode == "uncloak" => {
            let rule: GuardRule = serde_json::from_str(&fs::read_to_string(rules)?)?;
            fs::write(output, guard_uncloak(&fs::read_to_string(input)?, &rule)?)?;
        }
        [mode, input, specification, output] if mode == "harness" => {
            let unit: Unit = serde_json::from_str(&fs::read_to_string(specification)?)?;
            fs::write(output, emit_harness(&fs::read_to_string(input)?, &unit)?)?;
        }
        [mode, input, output] if mode == "emit" => {
            let unit: Unit = serde_json::from_str(&fs::read_to_string(input)?)?;
            fs::write(output, emit(&unit)?)?;
        }
        [mode, input, output] if mode == "transform" || mode == "module" => {
            let source = fs::read_to_string(input)?;
            let output_text = if mode == "transform" {
                transform(&source)?
            } else {
                module(&source)?
            };
            if let Some(parent) = Path::new(output).parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(output, output_text)?;
        }
        _ => {
            return Err(
                "usage: destiny-semantic-codegen {emit|transform|module} INPUT OUTPUT".into(),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_operators_and_types_fail_closed() {
        let field = || {
            Box::new(Expression::Field {
                name: "x".into(),
                machine_type: "double".into(),
            })
        };
        assert!(
            emit_expr(&Expression::Binary {
                operator: "%".into(),
                machine_type: "double".into(),
                left: field(),
                right: field()
            })
            .is_err()
        );
        assert!(
            emit_expr(&Expression::Field {
                name: "w".into(),
                machine_type: "double".into()
            })
            .is_err()
        );
        assert!(
            emit_expr(&Expression::Field {
                name: "x".into(),
                machine_type: "float".into()
            })
            .is_err()
        );
        assert!(serde_json::from_str::<Expression>(r#"{"op":"raw","rust":"return 0.0"}"#).is_err());
    }

    #[test]
    fn preserves_grouping_and_rejects_incomplete_transform() {
        let value: Expression = serde_json::from_str(r#"{"op":"binary","operator":"+","machine_type":"double","left":{"op":"field","name":"x","machine_type":"double"},"right":{"op":"binary","operator":"*","machine_type":"double","left":{"op":"field","name":"y","machine_type":"double"},"right":{"op":"field","name":"z","machine_type":"double"}}}"#).unwrap();
        let emitted = emit_expr(&value).unwrap().to_file_string().unwrap();
        syn::parse_str::<Expr>(emitted.trim()).unwrap();
        assert!(emitted.contains("vector[1] * vector[2]"));
        assert!(transform("fn unrelated() {}\n").is_err());
    }

    #[test]
    fn refuses_missing_or_ambiguous_state_guard() {
        let rule = GuardRule {
            schema: 1,
            kind: "reuse-uncloak-state-guard-v1".into(),
            reused_source: String::new(),
            reused_owner: "OriginalParkProjection".into(),
            reused_method: "uncloak_ball".into(),
        };
        assert!(reused_uncloak_guard("impl Unrelated {}", &rule).is_err());
        let missing =
            "impl OriginalParkProjection { fn uncloak_ball(&mut self) { b.is_massive = true; } }";
        assert!(reused_uncloak_guard(missing, &rule).is_err());
        let ambiguous = "impl OriginalParkProjection { fn uncloak_ball(&mut self) { if first { b.is_massive = true; } if second { b.is_massive = true; } } }";
        assert!(reused_uncloak_guard(ambiguous, &rule).is_err());
    }
}
