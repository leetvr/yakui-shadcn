//! Extract displayed code from the compiled gallery sources. The only reduction
//! is selecting string match arms for the current component/variant; state and
//! helper implementations come from the same files as the running preview.
use quote::{ToTokens, quote};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::PathBuf,
};
use syn::{
    Expr, Item, Pat,
    fold::{self, Fold},
    visit::{self, Visit},
};

struct Select {
    values: BTreeMap<String, String>,
}
impl Fold for Select {
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        if let Expr::Match(m) = &expr {
            if let Expr::Path(path) = &*m.expr {
                if let Some(value) = path
                    .path
                    .get_ident()
                    .and_then(|id| self.values.get(&id.to_string()))
                    .cloned()
                {
                    for arm in &m.arms {
                        if matches(&arm.pat, &value)
                            && arm
                                .guard
                                .as_ref()
                                .is_none_or(|(_, e)| self.condition(e) == Some(true))
                        {
                            let previous = self.values.clone();
                            if let Pat::Ident(p) = &arm.pat {
                                self.values.insert(p.ident.to_string(), value.clone());
                            }
                            let result = self.fold_expr(*arm.body.clone());
                            self.values = previous;
                            return result;
                        }
                    }
                }
            }
        }
        fold::fold_expr(self, expr)
    }
}
impl Select {
    fn condition(&self, expr: &Expr) -> Option<bool> {
        let Expr::Binary(b) = expr else { return None };
        let (Expr::Path(p), Expr::Lit(l)) = (&*b.left, &*b.right) else {
            return None;
        };
        let syn::Lit::Str(s) = &l.lit else {
            return None;
        };
        let value = self.values.get(&p.path.get_ident()?.to_string())?;
        match b.op {
            syn::BinOp::Eq(_) => Some(value == &s.value()),
            syn::BinOp::Ne(_) => Some(value != &s.value()),
            _ => None,
        }
    }
}
fn matches(pat: &Pat, value: &str) -> bool {
    match pat {
        Pat::Lit(p) => matches!(&p.lit, syn::Lit::Str(s) if s.value() == value),
        Pat::Or(p) => p.cases.iter().any(|p| matches(p, value)),
        Pat::Wild(_) | Pat::Ident(_) => true,
        _ => false,
    }
}
#[derive(Default)]
struct References {
    fields: BTreeSet<String>,
    functions: BTreeSet<String>,
    modules: BTreeSet<String>,
}
impl<'a> Visit<'a> for References {
    fn visit_expr_field(&mut self, f: &'a syn::ExprField) {
        if matches!(&*f.base, Expr::Path(p) if p.path.is_ident("state")) {
            if let syn::Member::Named(n) = &f.member {
                self.fields.insert(n.to_string());
            }
        }
        visit::visit_expr_field(self, f);
    }
    fn visit_expr_call(&mut self, c: &'a syn::ExprCall) {
        if let Expr::Path(p) = &*c.func {
            if let Some(id) = p.path.get_ident() {
                self.functions.insert(id.to_string());
            }
        }
        visit::visit_expr_call(self, c);
    }
    fn visit_path(&mut self, p: &'a syn::Path) {
        if p.segments.first().is_some_and(|s| s.ident == "crate") {
            if let Some(module) = p.segments.iter().nth(1) {
                self.modules.insert(module.ident.to_string());
            }
        }
        visit::visit_path(self, p);
    }
}
fn formatted(items: Vec<Item>) -> String {
    prettyplease::unparse(&syn::File {
        shebang: None,
        attrs: vec![],
        items,
    })
}
fn string_list(expr: &Expr) -> Vec<String> {
    match expr {
        Expr::Reference(e) => string_list(&e.expr),
        Expr::Array(a) => a
            .elems
            .iter()
            .filter_map(|e| match e {
                Expr::Lit(l) => match &l.lit {
                    syn::Lit::Str(s) => Some(s.value()),
                    _ => None,
                },
                _ => None,
            })
            .collect(),
        _ => vec![],
    }
}
fn main() {
    let mut files = BTreeMap::new();
    for name in [
        "examples",
        "avatars",
        "button_groups",
        "dropdown",
        "input_groups",
        "navigation_menus",
        "selects",
        "tooltips",
        "assets",
    ] {
        let path = format!("src/{name}.rs");
        println!("cargo:rerun-if-changed={path}");
        files.insert(
            name,
            syn::parse_file(&fs::read_to_string(path).unwrap()).unwrap(),
        );
    }
    let main = &files["examples"];
    let functions: BTreeMap<_, _> = main
        .items
        .iter()
        .filter_map(|i| {
            if let Item::Fn(f) = i {
                Some((f.sig.ident.to_string(), f))
            } else {
                None
            }
        })
        .collect();
    let components = main
        .items
        .iter()
        .find_map(|i| {
            if let Item::Const(c) = i {
                (c.ident == "AVAILABLE").then(|| string_list(&c.expr))
            } else {
                None
            }
        })
        .unwrap();
    let mut output = String::from("pub const SNIPPETS: &[Snippet] = &[\n");
    for component in components {
        let mut select = Select {
            values: BTreeMap::from([("component".into(), component.clone())]),
        };
        let variants = select.fold_item_fn(functions["variants"].clone());
        let syn::Stmt::Expr(expr, _) = &variants.block.stmts[0] else {
            panic!("variants must return a match")
        };
        let mut variants = string_list(expr);
        if let Expr::Path(path) = expr {
            let module = path.path.segments.iter().nth(1).unwrap().ident.to_string();
            variants = files[module.as_str()]
                .items
                .iter()
                .find_map(|i| {
                    if let Item::Const(c) = i {
                        (c.ident == "VARIANTS").then(|| string_list(&c.expr))
                    } else {
                        None
                    }
                })
                .unwrap();
        }
        assert!(!variants.is_empty(), "No variants for {component}");
        for variant in variants {
            select.values.insert("variant".into(), variant.clone());
            let mut refs = References::default();
            let mut preview = vec![];
            for name in ["render", "overlays"] {
                let mut f = select.fold_item_fn(functions[name].clone());
                // Shared modal setup belongs only to the modal examples. Other
                // overlays need the selected final expression, not that closure.
                if name == "overlays"
                    && !["dialog", "alert-dialog", "sheet", "drawer", "popover"]
                        .contains(&component.as_str())
                {
                    f.block.stmts = vec![f.block.stmts.last().unwrap().clone()];
                }
                if name == "overlays" && f.block.stmts.iter().all(|s| matches!(s, syn::Stmt::Expr(Expr::Block(b), _) if b.block.stmts.is_empty())) { continue; }
                if let [syn::Stmt::Expr(Expr::Block(block), _)] = f.block.stmts.as_slice() {
                    f.block = Box::new(block.block.clone());
                }
                f.attrs.clear();
                refs.visit_item_fn(&f);
                preview.push(Item::Fn(f));
            }
            let imports: Vec<_> = main
                .items
                .iter()
                .filter(|i| matches!(i, Item::Use(_)))
                .cloned()
                .collect();
            let mut helpers = vec![];
            let mut seen = BTreeSet::from(["render".to_string(), "overlays".to_string()]);
            loop {
                let names: Vec<_> = refs.functions.difference(&seen).cloned().collect();
                let modules: Vec<_> = refs
                    .modules
                    .iter()
                    .filter(|n| !seen.contains(&format!("mod:{n}")))
                    .cloned()
                    .collect();
                if names.is_empty() && modules.is_empty() {
                    break;
                }
                for name in names {
                    seen.insert(name.clone());
                    if let Some(f) = functions.get(&name) {
                        let f = select.fold_item_fn((*f).clone());
                        refs.visit_item_fn(&f);
                        helpers.push(Item::Fn(f));
                    }
                }
                for name in modules {
                    seen.insert(format!("mod:{name}"));
                    if name == "examples" {
                        continue;
                    }
                    if let Some(file) = files.get(name.as_str()) {
                        let mut file = select.fold_file(file.clone());
                        file.items
                            .retain(|i| !matches!(i, Item::Mod(_) | Item::Const(_)));
                        refs.visit_file(&file);
                        let ident = syn::Ident::new(&name, proc_macro_span());
                        let items = file.items;
                        helpers.push(syn::parse2(quote!(mod #ident { #(#items)* })).unwrap());
                    }
                }
            }
            let mut state = vec![];
            for item in &main.items {
                if let Item::Struct(s) = item {
                    if s.ident == "ExampleState" {
                        let mut s = s.clone();
                        if let syn::Fields::Named(f) = &mut s.fields {
                            f.named = f
                                .named
                                .iter()
                                .filter(|f| {
                                    refs.fields.contains(&f.ident.as_ref().unwrap().to_string())
                                })
                                .cloned()
                                .collect();
                        }
                        state.push(Item::Struct(s));
                    }
                }
                if let Item::Impl(i) = item {
                    if i.trait_
                        .as_ref()
                        .is_some_and(|(_, p, _)| p.is_ident("Default"))
                    {
                        let mut i = i.clone();
                        if let syn::ImplItem::Fn(f) = &mut i.items[0] {
                            if let syn::Stmt::Expr(Expr::Struct(s), _) = &mut f.block.stmts[0] {
                                s.fields = s
                                    .fields
                                    .iter()
                                    .filter(|f| {
                                        refs.fields
                                            .contains(&f.member.to_token_stream().to_string())
                                    })
                                    .cloned()
                                    .collect();
                            }
                        }
                        state.push(Item::Impl(i));
                    }
                }
            }
            let example = format!(
                "// component = {component:?}, variant = {variant:?}\n{}",
                formatted(imports.into_iter().chain(preview).collect())
            );
            output.push_str(&format!("Snippet {{ component: {component:?}, variant: {variant:?}, example: {example:?}, state: {:?}, helpers: {:?} }},\n", formatted(state), formatted(helpers).replace("crate::examples::ExampleState", "crate::ExampleState")));
        }
    }
    output.push_str("];\n");
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("snippets.rs"),
        output,
    )
    .unwrap();
}
fn proc_macro_span() -> proc_macro2::Span {
    proc_macro2::Span::call_site()
}
