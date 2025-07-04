use std::{
    any::Any,
    collections::{HashMap, HashSet},
};

use regex::Regex;

use crate::{
    nodes::{_base::Node, extras::Parameter, html_entity::HTMLEntity, text::Text},
    utils::parse_anything,
    wikicode::Wikicode,
};

/// Represents a template in wikicode, like `{{foo}}`.
pub struct Template {
    name: Wikicode,
    params: Vec<Parameter>,
}

impl Template {
    pub fn new<N: Into<Wikicode>>(name: N, params: Option<Vec<Parameter>>) -> Self {
        Template {
            name: parse_anything(name, 0, false),
            params: params.unwrap_or_default(),
        }
    }

    pub fn name(&self) -> &Wikicode {
        &self.name
    }

    pub fn set_name<V: Into<Wikicode>>(&mut self, value: V) {
        self.name = parse_anything(value, 0, false);
    }

    pub fn params(&self) -> &Vec<Parameter> {
        &self.params
    }

    pub fn as_str(&self) -> String {
        if !self.params.is_empty() {
            let inner = self
                .params
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join("|");
            format!("{{{{{}|{}}}}}", self.name.as_str(), inner)
        } else {
            format!("{{{{{}}}}}", self.name.as_str())
        }
    }

    pub fn children<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Wikicode> + 'a> {
        let mut v: Vec<&Wikicode> = Vec::with_capacity(1 + self.params.len() * 2);
        v.push(&self.name);
        for param in &self.params {
            if param.showkey() {
                v.push(param.name());
            }
            v.push(param.value());
        }
        Box::new(v.into_iter())
    }

    pub fn strip(
        &self,
        normalize: bool,
        collapse: bool,
        keep_template_params: bool,
    ) -> Option<String> {
        if keep_template_params {
            let parts: Vec<String> = self
                .params
                .iter()
                .filter_map(|p| {
                    Option::from(
                        p.value()
                            .strip_code(normalize, collapse, keep_template_params),
                    )
                })
                .collect();
            Some(parts.join(" "))
        } else {
            None
        }
    }

    pub fn showtree(&self, write: &dyn Fn(&[&str]), get: &dyn Fn(&Wikicode), mark: &dyn Fn()) {
        write(&["{{"]);
        get(&self.name);
        for param in &self.params {
            write(&["    | "]);
            mark();
            get(param.name());
            write(&["    = "]);
            mark();
            get(param.value());
        }
        write(&["}}"]);
    }

    /// Escape a single character in all text nodes of `code`.
    fn surface_escape(code: &Wikicode, ch: char) {
        let repl = HTMLEntity::new(ch as u32, None, false, 'x').to_string();
        unimplemented!("surface_escape requires `filter_text` and `replace` on Wikicode");
    }

    /// Pick the most common key in `theories`, if its frequency >50%.
    fn select_theory(theories: &HashMap<String, usize>) -> Option<String> {
        if theories.is_empty() {
            return None;
        }
        let total: usize = theories.values().sum();
        let (best_k, &best_v) = theories.iter().max_by_key(|&(_, &count)| count).unwrap();
        if (best_v as f64) / (total as f64) > 0.5 {
            Some(best_k.clone())
        } else {
            None
        }
    }

    /// Blank out `value` contents but preserve leading/trailing whitespace.
    fn blank_param_value(value: &mut Wikicode) {
        unimplemented!("blank_param_value requires direct node replacement");
    }

    /// Determine preferred spacing before/after names or values.
    fn get_spacing_conventions(&self, use_names: bool) -> (Option<String>, Option<String>) {
        let mut before: HashMap<String, usize> = HashMap::new();
        let mut after: HashMap<String, usize> = HashMap::new();
        let re = Regex::new(r"^(\s*).*?(\s*)$").unwrap();
        for param in &self.params {
            if !param.showkey() {
                continue;
            }
            let comp = if use_names {
                param.name().as_str().to_string()
            } else {
                param.value().as_str().to_string()
            };
            let caps = re.captures(&comp).unwrap();
            let mut b = caps.get(1).unwrap().as_str().to_string();
            let mut a = caps.get(2).unwrap().as_str().to_string();
            if !use_names && comp.trim().is_empty() && comp.contains('\n') {
                let parts: Vec<&str> = b.splitn(2, '\n').collect();
                b = parts[0].to_string();
                a = format!("\n{}", parts[1]);
            }
            *before.entry(b).or_default() += 1;
            *after.entry(a).or_default() += 1;
        }
        (Self::select_theory(&before), Self::select_theory(&after))
    }

    fn fix_dependent_params(&mut self, idx: usize) {
        if !self.params[idx].showkey() {
            for p in &mut self.params[idx + 1..] {
                if !p.showkey() {
                    p.set_showkey(true);
                }
            }
        }
    }

    fn remove_exact(&mut self, needle: &Parameter, keep_field: bool) {
        if let Some(pos) = self.params.iter().position(|p| std::ptr::eq(p, needle)) {
            if keep_field {
                // blank value
                unimplemented!("blank value for exact remove");
            } else {
                self.fix_dependent_params(pos);
                self.params.remove(pos);
            }
            return;
        }
        panic!("parameter not found");
    }

    fn should_remove(&self, idx: usize, name: &str) -> bool {
        if self.params[idx].showkey() {
            let following = &self.params[idx + 1..];
            following.iter().any(|after| {
                after.name().strip_code(true, true, true).as_ref() == Some(name)
                    && !after.showkey()
            })
        } else {
            false
        }
    }

    pub fn has(&self, name: &str, ignore_empty: bool) -> bool {
        let key = name.trim();
        for p in &self.params {
            if p.name().strip_code(true, true, true).as_ref() == Some(key) {
                if ignore_empty
                    && p.value()
                        .strip_code(true, true, true)
                        .is_empty()
                {
                    continue;
                }
                return true;
            }
        }
        false
    }

    pub fn get(&self, name: &str) -> Option<&Parameter> {
        let key = name.trim();
        self.params
            .iter()
            .rev()
            .find(|p| p.name().strip_code(true, true, true).as_ref() == Some(key))
    }

    pub fn add(
        &mut self,
        name: impl Into<Wikicode>,
        value: impl Into<Wikicode>,
        showkey: Option<bool>,
        before: Option<&Parameter>,
        after: Option<&Parameter>,
        preserve_spacing: bool,
    ) -> &mut Parameter {
        let name_wc = parse_anything(name, 0, false);
        let mut value_wc = parse_anything(value, 0, false);
        Self::surface_escape(&value_wc, '|');

        if let Some(existing) = self.get(&*name_wc.as_str()) {
            // update existing (simplified)
            existing.set_value(value_wc.clone());
            if let Some(sk) = showkey {
                existing.set_showkey(sk);
            }
            return unsafe { &mut *(existing as *const _ as *mut _) };
        }

        let sk = showkey.unwrap_or_else(|| Parameter::can_hide_key(&name_wc));
        if !sk {
            Self::surface_escape(&value_wc, '=');
        }

        let param = Parameter::new(name_wc.clone(), value_wc.clone(), sk);
        if let Some(b) = before {
            let idx = self
                .params
                .iter()
                .position(|p| p.name().as_str() == b.name().as_str())
                .expect("before parameter not found");
            self.params.insert(idx, param);
        } else if let Some(a) = after {
            let idx = self
                .params
                .iter()
                .position(|p| p.name().as_str() == a.name().as_str())
                .expect("after parameter not found");
            self.params.insert(idx + 1, param);
        } else {
            self.params.push(param);
        }
        self.params.last_mut().unwrap()
    }

    pub fn update(&mut self, mapping: HashMap<String, String>) {
        for (k, v) in mapping {
            self.add(k, v, None, None, None, true);
        }
    }

    pub fn remove(&mut self, param: &str, keep_field: bool) {
        // remove by name
        let mut removed = false;
        let mut to_remove = Vec::new();
        for (i, p) in self.params.iter().enumerate() {
            if p.name().strip_code(true, true, true).as_deref() == Some(param.trim()) {
                if keep_field {
                    if self.should_remove(i, param) {
                        to_remove.push(i);
                    } else {
                        unimplemented!("blank this parameter's value");
                    }
                } else {
                    self.fix_dependent_params(i);
                    to_remove.push(i);
                }
                removed = true;
            }
        }
        if !removed {
            panic!("no such parameter {}", param);
        }
        for idx in to_remove.into_iter().rev() {
            self.params.remove(idx);
        }
    }
}

impl Node for Template {
    fn as_str(&self) -> String {
        self.as_str()
    }

    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Wikicode> + 'a> {
        self.children()
    }

    fn strip(&self, normalize: bool, collapse: bool, keep_template_params: bool) -> Option<String> {
        self.strip(normalize, collapse, keep_template_params)
    }

    fn showtree(&self, write: &dyn Fn(&[&str]), get: &dyn Fn(&Wikicode), mark: &dyn Fn()) {
        self.showtree(write, get, mark)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
