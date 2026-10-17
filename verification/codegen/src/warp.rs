//! Reuse the reviewed source alignment guard at the actual goto-thrust caller.
use quote::ToTokens;
use std::{error::Error, fs};
use syn::{Expr, visit::Visit, visit_mut::VisitMut};

fn alignment_guard() -> Result<Expr, Box<dyn Error>> {
    let file = syn::parse_file(&fs::read_to_string(
        "verification/spec/src/original_park.rs",
    )?)?;
    struct Guard {
        conditions: Vec<Expr>,
    }
    impl<'ast> Visit<'ast> for Guard {
        fn visit_arm(&mut self, arm: &'ast syn::Arm) {
            if arm.pat.to_token_stream().to_string() == "OriginalMode :: Warp" {
                struct Conditions {
                    found: Vec<Expr>,
                }
                impl<'ast> Visit<'ast> for Conditions {
                    fn visit_expr_if(&mut self, branch: &'ast syn::ExprIf) {
                        if branch.cond.to_token_stream().to_string() == "b . effect_stamp < 0"
                            && branch
                                .then_branch
                                .to_token_stream()
                                .to_string()
                                .contains("original_goto_thrust")
                        {
                            self.found.push((*branch.cond).clone());
                        }
                        syn::visit::visit_expr_if(self, branch);
                    }
                }
                let mut selection = Conditions { found: Vec::new() };
                selection.visit_expr(&arm.body);
                self.conditions.extend(selection.found);
            }
            syn::visit::visit_arm(self, arm);
        }
    }
    let mut selection = Guard {
        conditions: Vec::new(),
    };
    selection.visit_file(&file);
    if selection.conditions.len() != 1 {
        return Err("missing or ambiguous source warp alignment guard".into());
    }
    let mut condition = selection.conditions.remove(0);
    struct Fields;
    impl VisitMut for Fields {
        fn visit_expr_path_mut(&mut self, path: &mut syn::ExprPath) {
            if path.path.is_ident("b") {
                path.path = syn::parse_quote!(ball);
            }
            syn::visit_mut::visit_expr_path_mut(self, path);
        }
    }
    Fields.visit_expr_mut(&mut condition);
    Ok(condition)
}

pub fn runtime(source: &str) -> Result<String, Box<dyn Error>> {
    let alignment = alignment_guard()?;
    let file = syn::parse_file(source)?;
    struct Calls {
        scope: String,
        conditions: Vec<Expr>,
    }
    impl<'ast> Visit<'ast> for Calls {
        fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
            let previous = std::mem::replace(&mut self.scope, function.sig.ident.to_string());
            syn::visit::visit_impl_item_fn(self, function);
            self.scope = previous;
        }
        fn visit_expr_if(&mut self, branch: &'ast syn::ExprIf) {
            if self.scope == "prepare_motion_tick"
                && branch.cond.to_token_stream().to_string()
                    == "control . mode == OriginalMode :: Goto"
            {
                self.conditions.push((*branch.cond).clone());
            }
            syn::visit::visit_expr_if(self, branch);
        }
    }
    let mut calls = Calls {
        scope: String::new(),
        conditions: Vec::new(),
    };
    calls.visit_file(&file);
    if calls.conditions.len() != 1 {
        return Err("goto-thrust dispatch changed".into());
    }
    let previous = calls.conditions.remove(0);
    let next: Expr =
        syn::parse_quote!(#previous || (control.mode == OriginalMode::Warp && #alignment));
    use syn::spanned::Spanned;
    let mut output = source.to_owned();
    output.replace_range(
        super::offset(source, previous.span().start())
            ..super::offset(source, previous.span().end()),
        &next.to_token_stream().to_string(),
    );
    syn::parse_file(&output)?;
    Ok(output)
}
