use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, DefinitionItem};
use crate::desc::verify_desc_locas;
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, info, warn, will_log, LogPauseRaii};
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler, FileKind};
use crate::pdxfile::PdxFile;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Decisions {
    decisions: FnvHashMap<String, Decision>,
}

impl Decisions {
    pub fn load_decision(&mut self, key: Token, block: &Block, values: Vec<(Token, Token)>) {
        if let Some(other) = self.decisions.get(key.as_str()) {
            if other.key.loc.kind == key.loc.kind && will_log(&key, ErrorKey::Duplicate) {
                error(
                    &key,
                    ErrorKey::Duplicate,
                    "decision redefines an existing decision",
                );
                info(
                    &other.key,
                    ErrorKey::Duplicate,
                    "the other decision is here",
                );
            }
        }
        self.decisions
            .insert(key.to_string(), Decision::new(key, block.clone(), values));
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.decisions.values() {
            item.validate(data);
        }
    }
}

impl FileHandler for Decisions {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/decisions")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let _pause = LogPauseRaii::new(entry.kind() != FileKind::ModFile);

        let block = match PdxFile::read(entry.path(), entry.kind(), fullpath) {
            Ok(block) => block,
            Err(e) => {
                error_info(
                    entry,
                    ErrorKey::ReadError,
                    "could not read file",
                    &format!("{:#}", e),
                );
                return;
            }
        };

        let mut decision_values: Vec<(Token, Token)> = Vec::new();

        for def in block.iter_definitions_warn() {
            match def {
                DefinitionItem::Keyword(key) => error_info(
                    key,
                    ErrorKey::Validation,
                    "unexpected token",
                    "Did you forget an = ?",
                ),
                DefinitionItem::Assignment(key, value) => {
                    if key.as_str().starts_with('@') {
                        decision_values.push((key.clone(), value.clone()));
                    } else {
                        error(
                            key,
                            ErrorKey::Validation,
                            "unknown setting in decision file",
                        );
                    }
                }
                DefinitionItem::Definition(key, b) => {
                    self.load_decision(key.clone(), b, decision_values.clone());
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Decision {
    key: Token,
    values: Vec<(Token, Token)>,
    block: Block,
}

impl Decision {
    pub fn new(key: Token, block: Block, values: Vec<(Token, Token)>) -> Self {
        Decision { key, values, block }
    }

    fn validate(&self, data: &Everything) {
        let mut vd = Validator::new(&self.block, data);

        vd.req_field("picture");
        if let Some(token) = vd.field_value("picture") {
            data.fileset.verify_exists(token);
        }
        if let Some(token) = vd.field_value("extra_picture") {
            data.fileset.verify_exists(token);
        }
        vd.field_bool("major");
        vd.field_integer("sort_order");
        vd.field_bool("is_invisible");
        vd.field_bool("ai_goal");
        vd.field_integer("ai_check_interval");
        if self.block.get_field_bool("ai_goal").unwrap_or(false) {
            vd.advice_field("ai_check_interval", "not needed if ai_goal = yes");
        }
        vd.field_block("cooldown");

        // kind of looks like a filename but it isn't.
        vd.field_value("confirm_click_sound");

        if let Some(bv) = vd.field("selection_tooltip") {
            verify_desc_locas(bv, &data.localization);
        } else {
            let loca = format!("{}_tooltip", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
        }

        if let Some(bv) = vd.field("title") {
            verify_desc_locas(bv, &data.localization);
        } else {
            data.localization.verify_exists(&self.key);
        }

        if let Some(bv) = vd.field("desc") {
            verify_desc_locas(bv, &data.localization);
        } else {
            let loca = format!("{}_desc", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
        }

        if let Some(bv) = vd.field("confirm_text") {
            verify_desc_locas(bv, &data.localization);
        } else {
            let loca = format!("{}_confirm", self.key);
            data.localization.verify_exists_implied(&loca, &self.key);
        }

        vd.field_block("is_shown");
        vd.field_block("is_valid_showing_failures_only");
        vd.field_block("is_valid");

        // cost can have multiple definitions and they will be combined
        // however, two costs of the same type are not summed
        vd.field_validated_blocks("cost", validate_cost);
        check_cost(&self.block.get_field_blocks("cost"));
        vd.field_validated_blocks("minimum_cost", validate_cost);
        check_cost(&self.block.get_field_blocks("minimum_cost"));

        vd.field_block("effect");
        vd.field_block("ai_potential");
        vd.field_block("ai_will_do");
        vd.field_block("should_create_alert");
        vd.field("widget");
        vd.warn_remaining();
    }
}

fn validate_cost(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    // These can all be script values
    vd.field("gold");
    vd.field("prestige");
    vd.field("piety");
    vd.warn_remaining();
}

fn check_cost(blocks: &[&Block]) {
    let mut seen_gold = false;
    let mut seen_prestige = false;
    let mut seen_piety = false;
    if blocks.len() > 1 {
        for cost in blocks {
            if let Some(gold) = cost.get_field("gold") {
                if seen_gold {
                    warn(
                        gold,
                        ErrorKey::Conflict,
                        "This value of the gold cost overrides the previously set cost.",
                    );
                }
                seen_gold = true;
            }
            if let Some(prestige) = cost.get_field("prestige") {
                if seen_prestige {
                    warn(
                        prestige,
                        ErrorKey::Conflict,
                        "This value of the prestige cost overrides the previously set cost.",
                    );
                }
                seen_prestige = true;
            }
            if let Some(piety) = cost.get_field("piety") {
                if seen_piety {
                    warn(
                        piety,
                        ErrorKey::Conflict,
                        "This value of the piety cost overrides the previously set cost.",
                    );
                }
                seen_piety = true;
            }
        }
    }
}