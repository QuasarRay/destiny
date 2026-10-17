//! Small typed statement IR for source-backed state transitions. No raw Rust
//! expression/body escape hatch is accepted in the rule input.
use quote::{ToTokens, quote};
use serde::Deserialize;
use std::error::Error;
use syn::visit_mut::VisitMut;

// Representation-only idioms. The selected rules are restricted to simple
// paths/fields or boolean guards; no indexed lvalue or effectful expression is
// evaluated fewer times as a result of these transformations.
struct Idioms;
impl VisitMut for Idioms {
    fn visit_expr_if_mut(&mut self, branch: &mut syn::ExprIf) {
        syn::visit_mut::visit_expr_if_mut(self, branch);
        if branch.else_branch.is_none()
            && branch.then_branch.stmts.len() == 1
            && let syn::Stmt::Expr(syn::Expr::If(inner), _) = &branch.then_branch.stmts[0]
            && inner.else_branch.is_none()
        {
            let condition = |value: &syn::Expr| {
                if matches!(value, syn::Expr::Let(_)) {
                    quote!(#value)
                } else {
                    quote!((#value))
                }
            };
            let first = condition(&branch.cond);
            let second = condition(&inner.cond);
            let body = &inner.then_branch;
            *branch = syn::parse_quote!(if #first && #second #body);
        }
    }
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        syn::visit_mut::visit_expr_mut(self, expression);
        if let syn::Expr::Assign(assignment) = expression
            && let syn::Expr::Binary(binary) = &*assignment.right
            && matches!(binary.op, syn::BinOp::Add(_))
        {
            fn plain(value: &syn::Expr) -> Option<&syn::Expr> {
                match value {
                    syn::Expr::Paren(value) => plain(&value.expr),
                    syn::Expr::Path(_) => Some(value),
                    syn::Expr::Field(field) if plain(&field.base).is_some() => Some(value),
                    _ => None,
                }
            }
            if let (Some(left), Some(repeated)) = (plain(&assignment.left), plain(&binary.left))
                && left.to_token_stream().to_string() == repeated.to_token_stream().to_string()
            {
                let right = match &*binary.right {
                    syn::Expr::Paren(value) => &*value.expr,
                    value => value,
                };
                *expression = syn::parse_quote!(#left += #right);
            }
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Type {
    Callback {
        parameters: Vec<Type>,
        result: Box<Type>,
    },
    Named {
        path: Vec<String>,
        #[serde(default)]
        arguments: Vec<Type>,
    },
    Array {
        element: Box<Type>,
        length: usize,
    },
    Slice {
        element: Box<Type>,
    },
    Ref {
        element: Box<Type>,
        #[serde(default)]
        mutable: bool,
    },
    Tuple {
        elements: Vec<Type>,
    },
}

fn ident(name: &str) -> Result<syn::Ident, Box<dyn Error>> {
    Ok(syn::parse_str(name)?)
}

fn path(names: &[String]) -> Result<syn::Path, Box<dyn Error>> {
    if names.is_empty() {
        return Err("empty state-rule path".into());
    }
    for name in names {
        if !matches!(name.as_str(), "crate" | "self" | "super" | "Self") {
            ident(name)?;
        }
    }
    Ok(syn::parse_str(&names.join("::"))?)
}

impl Type {
    fn lower(&self) -> Result<syn::Type, Box<dyn Error>> {
        Ok(match self {
            Self::Callback { parameters, result } => {
                let parameters = parameters
                    .iter()
                    .map(Self::lower)
                    .collect::<Result<Vec<_>, _>>()?;
                let result = result.lower()?;
                syn::parse2(quote!(dyn FnMut(#(#parameters),*) -> #result))?
            }
            Self::Slice { element } => {
                let element = element.lower()?;
                syn::parse2(quote!([#element]))?
            }
            Self::Named {
                path: names,
                arguments,
            } => {
                let path = path(names)?;
                if arguments.is_empty() {
                    syn::parse2(quote!(#path))?
                } else {
                    let arguments = arguments
                        .iter()
                        .map(Self::lower)
                        .collect::<Result<Vec<_>, _>>()?;
                    syn::parse2(quote!(#path<#(#arguments),*>))?
                }
            }
            Self::Array { element, length } => {
                let element = element.lower()?;
                syn::parse2(quote!([#element;#length]))?
            }
            Self::Ref { element, mutable } => {
                let element = element.lower()?;
                if *mutable {
                    syn::parse2(quote!(&mut #element))?
                } else {
                    syn::parse2(quote!(&#element))?
                }
            }
            Self::Tuple { elements } => {
                let elements = elements
                    .iter()
                    .map(Self::lower)
                    .collect::<Result<Vec<_>, _>>()?;
                syn::parse2(quote!((#(#elements,)*)))?
            }
        })
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Pattern {
    Path {
        components: Vec<String>,
    },
    Tuple {
        components: Vec<String>,
        bindings: Vec<String>,
    },
}
impl Pattern {
    fn lower(&self) -> Result<syn::Pat, Box<dyn Error>> {
        let tokens = match self {
            Self::Path { components } => {
                let p = path(components)?;
                quote!(#p)
            }
            Self::Tuple {
                components,
                bindings,
            } => {
                let p = path(components)?;
                let names = bindings
                    .iter()
                    .map(|n| ident(n))
                    .collect::<Result<Vec<_>, _>>()?;
                quote!(#p(#(#names),*))
            }
        };
        Ok(syn::parse::Parser::parse2(syn::Pat::parse_single, tokens)?)
    }
}

#[derive(Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
pub enum Expression {
    Match {
        value: Box<Self>,
        arms: Vec<(Pattern, Self)>,
    },
    Try {
        value: Box<Self>,
    },
    Path {
        components: Vec<String>,
    },
    Name {
        name: String,
    },
    Literal {
        value: serde_json::Value,
        machine_type: String,
    },
    Field {
        base: Box<Self>,
        member: String,
    },
    Index {
        base: Box<Self>,
        index: Box<Self>,
    },
    Call {
        function: Vec<String>,
        arguments: Vec<Self>,
    },
    Method {
        receiver: Box<Self>,
        method: String,
        arguments: Vec<Self>,
        #[serde(default)]
        type_arguments: Vec<Type>,
    },
    Binary {
        operator: String,
        left: Box<Self>,
        right: Box<Self>,
    },
    Unary {
        operator: String,
        value: Box<Self>,
    },
    Record {
        path: Vec<String>,
        fields: Vec<(String, Self)>,
    },
    Array {
        elements: Vec<Self>,
    },
    Tuple {
        elements: Vec<Self>,
    },
    Ref {
        value: Box<Self>,
        #[serde(default)]
        mutable: bool,
    },
    Closure {
        parameters: Vec<String>,
        expression: Box<Self>,
    },
    Cast {
        value: Box<Self>,
        machine_type: Type,
    },
    Range {
        start: Box<Self>,
        end: Box<Self>,
    },
}

impl Expression {
    fn lower(&self) -> Result<syn::Expr, Box<dyn Error>> {
        let tokens = match self {
            Self::Match { value, arms } => {
                if arms.is_empty() {
                    return Err("empty state match".into());
                }
                let value = value.lower()?;
                let arms = arms
                    .iter()
                    .map(|(p, e)| {
                        let p = p.lower()?;
                        let e = e.lower()?;
                        Ok(quote!(#p=>#e,))
                    })
                    .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
                quote!(match #value {#(#arms)*})
            }
            Self::Try { value } => {
                let value = value.lower()?;
                quote!((#value)?)
            }
            Self::Path { components } => {
                let path = path(components)?;
                quote!(#path)
            }
            Self::Name { name } if name == "self" => quote!(self),
            Self::Name { name } => {
                let name = ident(name)?;
                quote!(#name)
            }
            Self::Literal {
                value,
                machine_type,
            } => match (machine_type.as_str(), value) {
                ("bool", serde_json::Value::Bool(value)) => quote!(#value),
                ("str", serde_json::Value::String(value)) => {
                    let value = syn::LitStr::new(value, proc_macro2::Span::call_site());
                    quote!(#value)
                }
                ("i64" | "u32" | "usize" | "f64", serde_json::Value::Number(value)) => {
                    if machine_type == "u32" || machine_type == "usize" {
                        value
                            .as_u64()
                            .ok_or("unsigned state literal is out of range")?;
                    } else if machine_type == "i64" {
                        value
                            .as_i64()
                            .ok_or("signed state literal is out of range")?;
                    }
                    let value: syn::Expr = syn::parse_str(&format!("{value}_{machine_type}"))?;
                    quote!(#value)
                }
                _ => return Err("unsupported state literal machine type/value".into()),
            },
            Self::Field { base, member } => {
                let base = base.lower()?;
                let member: syn::Member = syn::parse_str(member)?;
                quote!(#base.#member)
            }
            Self::Index { base, index } => {
                let base = base.lower()?;
                let index = index.lower()?;
                quote!(#base[#index])
            }
            Self::Call {
                function,
                arguments,
            } => {
                let function = path(function)?;
                let arguments = arguments
                    .iter()
                    .map(Self::lower)
                    .collect::<Result<Vec<_>, _>>()?;
                quote!(#function(#(#arguments),*))
            }
            Self::Method {
                receiver,
                method,
                arguments,
                type_arguments,
            } => {
                let receiver = receiver.lower()?;
                let method = ident(method)?;
                let arguments = arguments
                    .iter()
                    .map(Self::lower)
                    .collect::<Result<Vec<_>, _>>()?;
                let generics = if type_arguments.is_empty() {
                    None
                } else {
                    let types = type_arguments
                        .iter()
                        .map(Type::lower)
                        .collect::<Result<Vec<_>, _>>()?;
                    Some(quote!(::<#(#types),*>))
                };
                quote!((#receiver).#method #generics (#(#arguments),*))
            }
            Self::Binary {
                operator,
                left,
                right,
            } => {
                if ![
                    "+", "-", "*", "/", "==", "!=", "<", "<=", ">", ">=", "&&", "||", "<<", ">>",
                ]
                .contains(&operator.as_str())
                {
                    return Err("unsupported state binary operator".into());
                }
                let left = left.lower()?;
                let right = right.lower()?;
                let op: syn::BinOp = syn::parse_str(operator)?;
                quote!((#left) #op (#right))
            }
            Self::Unary { operator, value } => {
                let value = value.lower()?;
                match operator.as_str() {
                    "!" => quote!(!(#value)),
                    "-" => quote!(-(#value)),
                    "*" => quote!(*(#value)),
                    _ => return Err("unsupported state unary operator".into()),
                }
            }
            Self::Record {
                path: names,
                fields,
            } => {
                let path = path(names)?;
                let fields = fields
                    .iter()
                    .map(|(name, value)| {
                        let name = ident(name)?;
                        let value = value.lower()?;
                        if matches!(&value, syn::Expr::Path(value) if value.path.is_ident(&name)) {
                            Ok(quote!(#name))
                        } else {
                            Ok(quote!(#name:#value))
                        }
                    })
                    .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
                quote!(#path{#(#fields),*})
            }
            Self::Array { elements } => {
                let elements = elements
                    .iter()
                    .map(Self::lower)
                    .collect::<Result<Vec<_>, _>>()?;
                quote!([#(#elements),*])
            }
            Self::Tuple { elements } => {
                let elements = elements
                    .iter()
                    .map(Self::lower)
                    .collect::<Result<Vec<_>, _>>()?;
                quote!((#(#elements,)*))
            }
            Self::Ref { value, mutable } => {
                let value = value.lower()?;
                if *mutable {
                    quote!(&mut #value)
                } else {
                    quote!(&#value)
                }
            }
            Self::Closure {
                parameters,
                expression,
            } => {
                let parameters = parameters
                    .iter()
                    .map(|name| ident(name))
                    .collect::<Result<Vec<_>, _>>()?;
                let expression = expression.lower()?;
                quote!(|#(#parameters),*|#expression)
            }
            Self::Cast {
                value,
                machine_type,
            } => {
                let value = value.lower()?;
                let machine_type = machine_type.lower()?;
                quote!((#value) as #machine_type)
            }
            Self::Range { start, end } => {
                let start = start.lower()?;
                let end = end.lower()?;
                quote!(#start..#end)
            }
        };
        Ok(syn::parse2(tokens)?)
    }
}

#[derive(Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
pub enum Statement {
    WhileSome {
        name: String,
        value: Expression,
        body: Vec<Self>,
    },
    Let {
        name: String,
        #[serde(default)]
        mutable: bool,
        #[serde(default)]
        machine_type: Option<Type>,
        value: Expression,
    },
    LetSome {
        name: String,
        value: Expression,
        otherwise: Vec<Self>,
    },
    Assign {
        place: Expression,
        value: Expression,
    },
    Evaluate {
        value: Expression,
    },
    Return {
        value: Expression,
    },
    If {
        condition: Expression,
        then: Vec<Self>,
        #[serde(default)]
        otherwise: Vec<Self>,
    },
    IfSome {
        name: String,
        value: Expression,
        then: Vec<Self>,
        #[serde(default)]
        otherwise: Vec<Self>,
    },
    For {
        name: String,
        iterator: Expression,
        body: Vec<Self>,
    },
}

fn block(statements: &[Statement], tail: bool) -> Result<syn::Block, Box<dyn Error>> {
    let mut output = Vec::new();
    for (index, statement) in statements.iter().enumerate() {
        let last = index + 1 == statements.len();
        output.push(match statement {
            Statement::WhileSome { name, value, body } => {
                let name = ident(name)?;
                let value = value.lower()?;
                let body = block(body, false)?;
                quote!(while let Some(#name)=#value #body)
            }
            Statement::Let {
                name,
                mutable,
                machine_type,
                value,
            } => {
                let name = ident(name)?;
                let value = value.lower()?;
                let mutable = mutable.then(|| quote!(mut));
                let ty = machine_type
                    .as_ref()
                    .map(Type::lower)
                    .transpose()?
                    .map(|ty| quote!(:#ty));
                quote!(let #mutable #name #ty = #value;)
            }
            Statement::LetSome {
                name,
                value,
                otherwise,
            } => {
                let name = ident(name)?;
                let value = value.lower()?;
                let otherwise = block(otherwise, false)?;
                if otherwise.to_token_stream().to_string() == "{ return None ; }" {
                    quote!(let #name=#value?;)
                } else {
                    quote!(let Some(#name)=#value else #otherwise;)
                }
            }
            Statement::Assign { place, value } => {
                let place = place.lower()?;
                if !matches!(
                    place,
                    syn::Expr::Path(_) | syn::Expr::Field(_) | syn::Expr::Index(_)
                ) {
                    return Err("invalid state assignment place".into());
                }
                let value = value.lower()?;
                quote!(#place=#value;)
            }
            Statement::Evaluate { value } => {
                let value = value.lower()?;
                quote!(#value;)
            }
            Statement::Return { value } => {
                let value = value.lower()?;
                if last && tail {
                    if matches!(&value,syn::Expr::Tuple(value) if value.elems.is_empty()) {
                        quote!()
                    } else {
                        quote!(#value)
                    }
                } else {
                    if matches!(&value,syn::Expr::Tuple(value) if value.elems.is_empty()) {
                        quote!(return;)
                    } else {
                        quote!(return #value;)
                    }
                }
            }
            Statement::If {
                condition,
                then,
                otherwise,
            } => {
                let condition = condition.lower()?;
                let then = block(then, last && tail)?;
                if otherwise.is_empty() {
                    quote!(if #condition #then)
                } else {
                    let otherwise = block(otherwise, last && tail)?;
                    quote!(if #condition #then else #otherwise)
                }
            }
            Statement::For {
                name,
                iterator,
                body,
            } => {
                let name = ident(name)?;
                let iterator = iterator.lower()?;
                let body = block(body, false)?;
                quote!(for #name in #iterator #body)
            }
            Statement::IfSome {
                name,
                value,
                then,
                otherwise,
            } => {
                let name = ident(name)?;
                let value = value.lower()?;
                let then = block(then, last && tail)?;
                if otherwise.is_empty() {
                    quote!(if let Some(#name)=#value #then)
                } else {
                    let otherwise = block(otherwise, last && tail)?;
                    quote!(if let Some(#name)=#value #then else #otherwise)
                }
            }
        });
    }
    Ok(syn::parse2(quote!({#(#output)*}))?)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Function {
    #[serde(default)]
    pub visibility: Option<String>,
    pub name: String,
    pub owner: Option<String>,
    pub receiver: Option<String>,
    pub parameters: Vec<(String, Type)>,
    pub result: Type,
    pub source_regions: Vec<String>,
    pub body: Vec<Statement>,
}

impl Function {
    pub fn lower(&self) -> Result<proc_macro2::TokenStream, Box<dyn Error>> {
        if self.source_regions.is_empty() {
            return Err("state function has no source provenance".into());
        }
        let name = ident(&self.name)?;
        let result = self.result.lower()?;
        let result = if matches!(result,syn::Type::Tuple(ref value) if value.elems.is_empty()) {
            None
        } else {
            Some(quote!(->#result))
        };
        let mut body = block(&self.body, true)?;
        Idioms.visit_block_mut(&mut body);
        let mut parameters = Vec::new();
        if let Some(receiver) = &self.receiver {
            parameters.push(match receiver.as_str() {
                "ref" => quote!(&self),
                "mut" => quote!(&mut self),
                "value" => quote!(self),
                _ => return Err("unknown state receiver".into()),
            });
        }
        for (name, ty) in &self.parameters {
            let name = ident(name)?;
            let ty = ty.lower()?;
            parameters.push(quote!(#name:#ty));
        }
        let provenance = self.source_regions.join(", ");
        let visibility = match self.visibility.as_deref().unwrap_or("public") {
            "public" => quote!(pub),
            "crate" => quote!(pub(crate)),
            "private" => quote!(),
            _ => return Err("unreviewed state function visibility".into()),
        };
        let function =
            quote!(#[doc=#provenance] #visibility fn #name(#(#parameters),*) #result #body);
        Ok(if let Some(owner) = &self.owner {
            let owner = ident(owner)?;
            quote!(impl #owner {#function})
        } else {
            function
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;
    #[test]
    fn callbacks_and_yielding_loops_are_structured_typed_nodes() {
        let ty:Type=serde_json::from_str(r#"{"kind":"Ref","mutable":true,"element":{"kind":"Callback","parameters":[{"kind":"Named","path":["Event"]}],"result":{"kind":"Named","path":["bool"]}}}"#).unwrap();
        let parsed = ty.lower().unwrap();
        assert_eq!(
            parsed.to_token_stream().to_string(),
            "& mut dyn FnMut (Event) -> bool"
        );
        let statement:Statement=serde_json::from_str(r#"{"op":"WhileSome","name":"event","value":{"op":"Name","name":"next"},"body":[{"op":"Evaluate","value":{"op":"Name","name":"event"}}]}"#).unwrap();
        let parsed = block(&[statement], false).unwrap();
        assert!(matches!(
            &parsed.stmts[0],
            syn::Stmt::Expr(syn::Expr::While(_), _)
        ));
        assert!(serde_json::from_str::<Type>(r#"{"kind":"Callback","parameters":[],"result":{"kind":"Named","path":["bool"]},"body":"panic!()"}"#).is_err());
    }
    #[test]
    fn generic_methods_and_match_patterns_preserve_typed_boundaries() {
        let value = r#"{"op":"Method","receiver":{"op":"Name","name":"world"},"method":"resource","arguments":[],"type_arguments":[{"kind":"Named","path":["crate","Frame"]}]}"#;
        let expression: Expression = serde_json::from_str(value).unwrap();
        let syn::Expr::MethodCall(call) = expression.lower().unwrap() else {
            panic!("generic method lost");
        };
        assert!(call.turbofish.is_some());
        let value = r#"{"op":"Match","value":{"op":"Name","name":"key"},"arms":[[{"kind":"Tuple","components":["ColliderKey","Ball"],"bindings":["id"]},{"op":"Name","name":"id"}],[{"kind":"Path","components":["ColliderKey","None"]},{"op":"Literal","value":-1,"machine_type":"i64"}]]}"#;
        let expression: Expression = serde_json::from_str(value).unwrap();
        let syn::Expr::Match(branch) = expression.lower().unwrap() else {
            panic!("match lost");
        };
        assert_eq!(branch.arms.len(), 2);
        let invalid = value.replace("\"id\"]", "\"id; panic!()\"]");
        assert!(
            serde_json::from_str::<Expression>(&invalid)
                .unwrap()
                .lower()
                .is_err()
        );
        let empty: Expression =
            serde_json::from_str(r#"{"op":"Match","value":{"op":"Name","name":"key"},"arms":[]}"#)
                .unwrap();
        assert!(empty.lower().is_err());
    }
    #[test]
    fn rooted_paths_and_try_preserve_structure() {
        let ty: Type = serde_json::from_str(
            r#"{"kind":"Named","path":["crate","generated_liveness","LivenessState"]}"#,
        )
        .unwrap();
        assert_eq!(
            ty.lower().unwrap().to_token_stream().to_string(),
            "crate :: generated_liveness :: LivenessState"
        );
        assert!(path(&["crate".into(), "x; panic!()".into()]).is_err());
        let expression: Expression =
            serde_json::from_str(r#"{"op":"Try","value":{"op":"Name","name":"value"}}"#).unwrap();
        assert!(matches!(expression.lower().unwrap(), syn::Expr::Try(_)));
        let slice:Type=serde_json::from_str(r#"{"kind":"Ref","element":{"kind":"Slice","element":{"kind":"Named","path":["crate","iterative_collision","Body"]}}}"#).unwrap();
        assert_eq!(
            slice.lower().unwrap().to_token_stream().to_string(),
            "& [crate :: iterative_collision :: Body]"
        );
        assert!(
            serde_json::from_str::<Type>(
                r#"{"kind":"Slice","element":{"kind":"Tuple","elements":[]},"length":8}"#
            )
            .is_err()
        );
    }
    #[test]
    fn state_rules_reject_target_code_and_unsupported_operators() {
        assert!(serde_json::from_str::<Expression>(r#"{"op":"Rust","body":"return 0;"}"#).is_err());
        let inject = r#"{"op":"Name","name":"value; panic!()"}"#;
        assert!(
            serde_json::from_str::<Expression>(inject)
                .unwrap()
                .lower()
                .is_err()
        );
        let unsupported = r#"{"op":"Binary","operator":"%","left":{"op":"Name","name":"x"},"right":{"op":"Name","name":"y"}}"#;
        assert!(
            serde_json::from_str::<Expression>(unsupported)
                .unwrap()
                .lower()
                .is_err()
        );
    }
    #[test]
    fn state_rules_preserve_structured_grouping() {
        let expression = r#"{"op":"Binary","operator":"*","left":{"op":"Binary","operator":"+","left":{"op":"Name","name":"x"},"right":{"op":"Name","name":"y"}},"right":{"op":"Name","name":"z"}}"#;
        let expression = serde_json::from_str::<Expression>(expression)
            .unwrap()
            .lower()
            .unwrap();
        assert_eq!(
            expression.to_token_stream().to_string(),
            "((x) + (y)) * (z)"
        );
    }
    #[test]
    fn unary_operator_preserves_the_whole_operand() {
        let expression = r#"{"op":"Unary","operator":"!","value":{"op":"Binary","operator":"<","left":{"op":"Name","name":"distance"},"right":{"op":"Name","name":"threshold"}}}"#;
        let expression = serde_json::from_str::<Expression>(expression)
            .unwrap()
            .lower()
            .unwrap();
        let syn::Expr::Unary(unary) = expression else {
            panic!("expected unary")
        };
        let syn::Expr::Paren(operand) = *unary.expr else {
            panic!("operand lost its group")
        };
        assert!(matches!(*operand.expr, syn::Expr::Binary(_)));
    }
}
