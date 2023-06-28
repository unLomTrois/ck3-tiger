use fnv::FnvHashMap;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, Comparator, BV};
use crate::context::ScopeContext;
use crate::errorkey::ErrorKey;
use crate::errors::{advice_info, error, warn};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{dup_error, TriBool};
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::scopes::{scope_from_snake_case, scope_iterator, Scopes};
use crate::token::{Loc, Token};
use crate::tooltipped::Tooltipped;
use crate::trigger::{validate_normal_trigger, validate_target_ok_this};
use crate::validate::{
    precheck_iterator_fields, validate_inside_iterator, validate_iterator_fields,
    validate_scope_chain, ListType,
};

#[derive(Clone, Debug, Default)]
pub struct ScriptValues {
    scope_overrides: FnvHashMap<String, Scopes>,
    scriptvalues: FnvHashMap<String, ScriptValue>,
}

impl ScriptValues {
    fn load_item(&mut self, key: &Token, bv: &BV) {
        if let Some(other) = self.scriptvalues.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind {
                dup_error(key, &other.key, "script value");
            }
        }
        let scope_override = self.scope_overrides.get(key.as_str()).copied();
        self.scriptvalues.insert(
            key.to_string(),
            ScriptValue::new(key.clone(), bv.clone(), scope_override),
        );
    }

    pub fn exists(&self, key: &str) -> bool {
        self.scriptvalues.contains_key(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.scriptvalues.values() {
            item.validate(data);
        }
    }

    pub fn validate_call(&self, key: &Token, data: &Everything, sc: &mut ScopeContext) {
        if let Some(item) = self.scriptvalues.get(key.as_str()) {
            item.validate_call(key, data, sc);
        }
    }
}

impl FileHandler for ScriptValues {
    fn config(&mut self, config: &Block) {
        if let Some(block) = config.get_field_block("scope_override") {
            for (key, token) in block.iter_assignments() {
                let mut scopes = Scopes::empty();
                if token.lowercase_is("all") {
                    scopes = Scopes::all();
                } else {
                    for part in token.split('|') {
                        if let Some(scope) = scope_from_snake_case(part.as_str()) {
                            scopes |= scope;
                        } else {
                            let msg = format!("unknown scope type `{part}`");
                            warn(part, ErrorKey::Config, &msg);
                        }
                    }
                }
                self.scope_overrides
                    .insert(key.as_str().to_string(), scopes);
            }
        }
    }

    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/script_values")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(block) = PdxFile::read(entry, fullpath) else { return };
        for (key, bv) in block.iter_bv_definitions_warn() {
            self.load_item(key, bv);
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScriptValue {
    key: Token,
    bv: BV,
    cache: RefCell<FnvHashMap<Loc, ScopeContext>>,
    scope_override: Option<Scopes>,
}

impl ScriptValue {
    pub fn new(key: Token, bv: BV, scope_override: Option<Scopes>) -> Self {
        Self {
            key,
            bv,
            cache: RefCell::new(FnvHashMap::default()),
            scope_override,
        }
    }

    fn validate_inner(
        mut vd: Validator,
        data: &Everything,
        sc: &mut ScopeContext,
        mut have_value: TriBool,
        check_desc: bool,
    ) {
        if check_desc {
            vd.field_item("desc", Item::Localization);
            vd.field_item("format", Item::Localization);
        } else {
            vd.field_value("desc");
            vd.field_value("format");
        }

        let mut seen_if;
        let mut next_seen_if = false;
        for (token, cmp, bv) in vd.unknown_fields_cmp() {
            seen_if = next_seen_if;
            next_seen_if = false;

            // save_temporary_scope_as is now allowed in script values
            if token.is("save_temporary_scope_as") {
                if let Some(name) = bv.expect_value() {
                    sc.save_current_scope(name.as_str());
                }
            } else if token.is("save_temporary_value_as") {
                // seen in vanilla
                if let Some(name) = bv.expect_value() {
                    sc.define_name(name.as_str(), Scopes::Value, name.clone());
                }
            } else if token.is("value") {
                if have_value == TriBool::True {
                    let msg = "setting value here will overwrite the previous calculations";
                    warn(token, ErrorKey::Logic, msg);
                }
                have_value = TriBool::True;
                Self::validate_bv_2(bv, data, sc, check_desc);
            } else if token.is("add") || token.is("subtract") || token.is("min") || token.is("max")
            {
                have_value = TriBool::True;
                Self::validate_bv_2(bv, data, sc, check_desc);
            } else if token.is("multiply") || token.is("divide") || token.is("modulo") {
                if have_value == TriBool::False {
                    let msg = format!("nothing to {token} yet");
                    warn(token, ErrorKey::Logic, &msg);
                }
                Self::validate_bv_2(bv, data, sc, check_desc);
            } else if token.is("round") || token.is("ceiling") || token.is("floor") {
                if have_value == TriBool::False {
                    let msg = format!("nothing to {token} yet");
                    warn(token, ErrorKey::Logic, &msg);
                }
                if let Some(value) = bv.expect_value() {
                    if !value.is("yes") && !value.is("no") {
                        let msg = "expected yes or no";
                        warn(value, ErrorKey::Validation, msg);
                    }
                }
            } else if token.is("fixed_range") || token.is("integer_range") {
                if have_value == TriBool::True {
                    let msg = "using fixed_range here will overwrite the previous calculations";
                    warn(token, ErrorKey::Logic, msg);
                }
                if let Some(block) = bv.expect_block() {
                    Self::validate_minmax_range(block, data, sc, check_desc);
                }
                have_value = TriBool::True;
            } else if token.is("if") {
                if let Some(block) = bv.expect_block() {
                    Self::validate_if(block, data, sc, check_desc);
                }
                have_value = TriBool::Maybe;
                next_seen_if = true;
            } else if token.is("else_if") {
                if !seen_if {
                    let msg = "`else_if` without preceding `if`";
                    warn(token, ErrorKey::Validation, msg);
                }
                if let Some(block) = bv.expect_block() {
                    Self::validate_if(block, data, sc, check_desc);
                }
                have_value = TriBool::Maybe;
                next_seen_if = true;
            } else if token.is("else") {
                if !seen_if {
                    let msg = "`else` without preceding `if`";
                    warn(token, ErrorKey::Validation, msg);
                }
                if let Some(block) = bv.expect_block() {
                    Self::validate_else(block, data, sc, check_desc);
                    if block.has_key("limit") {
                        // Another `else` after an `else` with a limit does work, so don't warn about it if it comes.
                        // There will already be an "advice" about this limit, so no need for an extra message.
                        next_seen_if = true;
                    }
                }
                have_value = TriBool::Maybe;
            } else {
                if let Some((it_type, it_name)) = token.split_once('_') {
                    if it_type.is("every")
                        || it_type.is("ordered")
                        || it_type.is("random")
                        || it_type.is("any")
                    {
                        if let Some((inscopes, outscope)) = scope_iterator(&it_name, data) {
                            if it_type.is("any") {
                                let msg = "cannot use `any_` iterators in a script value";
                                error(token, ErrorKey::Validation, msg);
                            }
                            sc.expect(inscopes, token);
                            if let Some(block) = bv.expect_block() {
                                let ltype = ListType::try_from(it_type.as_str()).unwrap();
                                precheck_iterator_fields(ltype, block, data, sc);
                                sc.open_scope(outscope, token.clone());
                                Self::validate_iterator(
                                    ltype, &it_name, block, data, sc, check_desc,
                                );
                                sc.close();
                                have_value = TriBool::Maybe;
                            }
                        }
                        continue;
                    }
                }

                // Check for target = { script_value } or target = compare_value
                sc.open_builder();
                if validate_scope_chain(token, data, sc, matches!(cmp, Comparator::QEq)) {
                    if let Some(block) = bv.expect_block() {
                        sc.finalize_builder();
                        let vd = Validator::new(block, data);
                        Self::validate_inner(vd, data, sc, have_value, check_desc);
                        have_value = TriBool::Maybe;
                    }
                }
                sc.close();
            }
        }
    }

    fn validate_iterator(
        ltype: ListType,
        it_name: &Token,
        block: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
        check_desc: bool,
    ) {
        let mut vd = Validator::new(block, data);
        vd.field_validated_block("limit", |block, data| {
            validate_normal_trigger(block, data, sc, Tooltipped::No);
        });

        let mut tooltipped = Tooltipped::No;
        validate_iterator_fields("", ltype, data, sc, &mut vd, &mut tooltipped);

        validate_inside_iterator(
            it_name.as_str(),
            ltype,
            block,
            data,
            sc,
            &mut vd,
            Tooltipped::No,
        );

        Self::validate_inner(vd, data, sc, TriBool::Maybe, check_desc);
    }

    fn validate_minmax_range(
        block: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
        check_desc: bool,
    ) {
        let mut vd = Validator::new(block, data);
        vd.req_field("min");
        vd.req_field("max");
        vd.field_validated_bvs("min", |bv, data| {
            Self::validate_bv_2(bv, data, sc, check_desc);
        });
        vd.field_validated_bvs("max", |bv, data| {
            Self::validate_bv_2(bv, data, sc, check_desc);
        });
    }

    fn validate_if(block: &Block, data: &Everything, sc: &mut ScopeContext, check_desc: bool) {
        let mut vd = Validator::new(block, data);
        vd.req_field_warn("limit");
        vd.field_validated_block("limit", |block, data| {
            validate_normal_trigger(block, data, sc, Tooltipped::No);
        });
        Self::validate_inner(vd, data, sc, TriBool::Maybe, check_desc);
    }

    fn validate_else(block: &Block, data: &Everything, sc: &mut ScopeContext, check_desc: bool) {
        let mut vd = Validator::new(block, data);
        vd.field_validated_key_block("limit", |key, block, data| {
            let msg = "`else` with a `limit` does work, but may indicate a mistake";
            let info = "normally you would use `else_if` instead.";
            advice_info(key, ErrorKey::IfElse, msg, info);
            validate_normal_trigger(block, data, sc, Tooltipped::No);
        });
        Self::validate_inner(vd, data, sc, TriBool::Maybe, check_desc);
    }

    pub fn validate_bv_2(bv: &BV, data: &Everything, sc: &mut ScopeContext, check_desc: bool) {
        // Using validate_target_ok_this here because when chaining script values to each other, you often do `value = this`
        match bv {
            BV::Value(t) => validate_target_ok_this(t, data, sc, Scopes::Value | Scopes::Bool),
            BV::Block(b) => {
                let mut vd = Validator::new(b, data);
                if let Some((None, _, _)) = b.iter_items().next() {
                    // It's a range like { 1 5 }
                    let vec = vd.values();
                    if vec.len() == 2 {
                        for v in vec {
                            validate_target_ok_this(v, data, sc, Scopes::Value | Scopes::Bool);
                        }
                    } else {
                        warn(b, ErrorKey::Validation, "invalid script value range");
                    }
                } else {
                    Self::validate_inner(vd, data, sc, TriBool::False, check_desc);
                }
            }
        }
    }

    pub fn validate_bv(bv: &BV, data: &Everything, sc: &mut ScopeContext) {
        Self::validate_bv_2(bv, data, sc, true);
    }

    pub fn validate_bv_no_breakdown(bv: &BV, data: &Everything, sc: &mut ScopeContext) {
        Self::validate_bv_2(bv, data, sc, false);
    }

    pub fn cached_compat(&self, key: &Token, sc: &mut ScopeContext) -> bool {
        if let Some(our_sc) = self.cache.borrow().get(&key.loc) {
            sc.expect_compatibility(our_sc, key);
            true
        } else {
            false
        }
    }

    pub fn validate(&self, data: &Everything) {
        // For some reason, script values can be set to bools as well
        if let Some(token) = self.bv.get_value() {
            if token.is("yes") || token.is("no") {
                return;
            }
        }
        let mut sc = ScopeContext::new_unrooted(Scopes::all(), self.key.clone());
        sc.set_strict_scopes(false);
        self.validate_call(&self.key, data, &mut sc);
    }

    pub fn validate_call(&self, key: &Token, data: &Everything, sc: &mut ScopeContext) {
        if !self.cached_compat(key, sc) {
            let mut our_sc = ScopeContext::new_unrooted(Scopes::all(), self.key.clone());
            our_sc.set_strict_scopes(false);
            self.cache
                .borrow_mut()
                .insert(key.loc.clone(), our_sc.clone());
            Self::validate_bv(&self.bv, data, &mut our_sc);
            if let Some(scopes) = self.scope_override {
                our_sc = ScopeContext::new_unrooted(scopes, self.key.clone());
                our_sc.set_strict_scopes(false);
            }
            sc.expect_compatibility(&our_sc, key);
            self.cache.borrow_mut().insert(key.loc.clone(), our_sc);
        }
    }
}
