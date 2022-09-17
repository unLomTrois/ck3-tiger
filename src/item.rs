use std::fmt::{Display, Formatter};

/// "items" are all the things that can be looked up in string-indexed databases.
/// There is some overlap with scopes, but the difference is that scopes are runtime values
/// while items are always strings.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Amenity,
    Artifact,
    ArtifactCategory,
    ArtifactFeature,
    ArtifactFeatureGroup,
    ArtifactModifier,
    ArtifactRarity,
    ArtifactSlot,
    Building,
    BuildingFlag,
    BuildingGfx,
    CasusBelli,
    Catalyst,
    Character,
    ClothingGfx,
    CoaGfx,
    CouncilPosition,
    CouncilTask,
    CourtPosition,
    CourtPositionCategory,
    CourtType,
    Culture,
    CultureEra,
    CultureParameter,
    CulturePillar,
    CultureTradition,
    Decision,
    Doctrine,
    DoctrineParameter,
    Dynasty,
    DynastyLegacy,
    DynastyPerk,
    EducationFocus,
    Event,
    Faction,
    Faith,
    FaithIcon,
    File,
    GameConcept,
    Government,
    GraphicalFaith,
    Holding,
    HolySite,
    HolySiteFlag,
    Hook,
    House,
    Innovation,
    InnovationFlag,
    Inspiration,
    Interaction,
    InteractionCategory,
    Language,
    Law,
    Lifestyle,
    Localization,
    MenAtArms,
    MenAtArmsBase,
    Modifier,
    Music,
    NameList,
    Nickname,
    Perk,
    PrisonType,
    Province,
    Region,
    Relation,
    Religion,
    ReligiousFamily,
    Scheme,
    ScriptedEffect,
    ScriptedList,
    ScriptedTrigger,
    ScriptValue,
    Secret,
    Sexuality,
    Skill,
    SpecialBuilding,
    Story,
    Struggle,
    StrugglePhase,
    StrugglePhaseParameter,
    Terrain,
    Title,
    TitleHistory,
    TitleLaw,
    TitleLawFlag,
    Tradition,
    Trait,
    UnitGfx,
    VassalObligation,
}

use crate::item::Item::*;

impl Item {
    pub fn path(self) -> &'static str {
        #[allow(clippy::match_same_arms)]
        match self {
            Amenity => "common/court_amenities/",
            Artifact => "common/artifacts/types",
            ArtifactCategory => "common/artifacts/",
            ArtifactFeature => "common/artifacts/features/",
            ArtifactFeatureGroup => "common/artifacts/feature_groups/",
            ArtifactModifier => "common/artifacts/",
            ArtifactRarity => "common/artifacts/",
            ArtifactSlot => "common/artifacts/",
            Building => "common/buildings/",
            BuildingFlag => "common/buildings/",
            BuildingGfx => "common/culture/cultures/",
            CasusBelli => "common/casus_belli_types",
            Catalyst => "common/struggle/catalysts/",
            Character => "history/characters/",
            ClothingGfx => "common/culture/cultures/",
            CoaGfx => "common/culture/cultures/",
            CouncilPosition => "common/council_positions/",
            CouncilTask => "common/council_tasks/",
            CourtPosition => "common/court_positions/types/",
            CourtPositionCategory => "common/court_positions/categories/",
            CourtType => "common/court_types/",
            Culture => "common/culture/cultures/",
            CultureEra => "common/culture/eras/",
            CultureParameter => "common/culture/cultures/",
            CulturePillar => "common/culture/pillars/",
            CultureTradition => "common/culture/traditions/",
            Decision => "common/decisions/",
            Doctrine => "common/religion/doctrines/",
            DoctrineParameter => "common/religion/doctrines/",
            Dynasty => "common/dynasties/",
            DynastyLegacy => "common/dynasty_legacies/",
            DynastyPerk => "common/dynasty_perks/",
            EducationFocus => "common/focuses/",
            Event => "events/",
            Faith => "common/religion/religions/",
            FaithIcon => "common/religion/religions/",
            Faction => "common/factions/",
            File => "",
            GameConcept => "common/game_concepts/",
            Government => "common/governments/",
            GraphicalFaith => "common/religion/religions/",
            Holding => "",
            HolySite => "common/religion/holy_sites/",
            HolySiteFlag => "common/religion/holy_sites/",
            Hook => "common/hook_types/",
            House => "common/dynasty_houses/",
            Innovation => "common/culture/innovations/",
            InnovationFlag => "common/culture/innovations/",
            Inspiration => "common/inspirations/",
            Interaction => "common/character_interactions/",
            InteractionCategory => "common/character_interaction_categories/",
            Language => "common/culture/pillars/",
            Law => "common/laws/",
            Lifestyle => "common/lifestyles/",
            Localization => "localization/",
            MenAtArms => "common/men_at_arms_types/",
            MenAtArmsBase => "common/men_at_arms_types/",
            Modifier => "common/modifiers/",
            Music => "music/",
            NameList => "common/culture/name_lists/",
            Nickname => "common/nicknames/",
            Perk => "common/lifestyle_perks/",
            PrisonType => "",
            Province => "map_data/definition.csv",
            Region => "map_data/geographical_regions/",
            Relation => "common/scripted_relations/",
            Religion => "common/religion/religions/",
            ReligiousFamily => "common/religion/religion_families/",
            Scheme => "common/schemes/",
            ScriptedEffect => "common/scripted_effects/",
            ScriptedList => "common/scripted_lists/",
            ScriptedTrigger => "common/scripted_triggers/",
            ScriptValue => "common/script_values/",
            Secret => "common/secret_types/",
            Sexuality => "",
            Skill => "",
            Story => "common/story_cycle/",
            Struggle => "common/struggle/struggles/",
            StrugglePhase => "common/struggle/struggles/",
            StrugglePhaseParameter => "common/struggle/struggles/",
            SpecialBuilding => "common/buildings/",
            Terrain => "common/terrain_types/",
            Title => "common/landed_titles/",
            TitleHistory => "history/titles/",
            TitleLaw => "common/laws/",
            TitleLawFlag => "common/laws/",
            Tradition => "common/culture/traditions/",
            Trait => "common/traits/",
            UnitGfx => "common/culture/cultures/",
            VassalObligation => "common/vassal_contracts/",
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            Amenity => write!(f, "amenity"),
            Artifact => write!(f, "artifact"),
            ArtifactCategory => write!(f, "artifact category"),
            ArtifactFeature => write!(f, "artifact feature"),
            ArtifactFeatureGroup => write!(f, "artifact feature group"),
            ArtifactModifier => write!(f, "artifact modifier"),
            ArtifactRarity => write!(f, "artifact rarity"),
            ArtifactSlot => write!(f, "artifact slot"),
            Building => write!(f, "building"),
            BuildingFlag => write!(f, "building flag"),
            BuildingGfx => write!(f, "building gfx"),
            CasusBelli => write!(f, "casus belli"),
            Catalyst => write!(f, "catalyst"),
            Character => write!(f, "character"),
            ClothingGfx => write!(f, "clothing gfx"),
            CoaGfx => write!(f, "coa gfx"),
            CouncilPosition => write!(f, "council position"),
            CouncilTask => write!(f, "council task"),
            CourtPosition => write!(f, "court position"),
            CourtPositionCategory => write!(f, "court position category"),
            CourtType => write!(f, "court type"),
            Culture => write!(f, "culture"),
            CultureEra => write!(f, "culture era"),
            CultureParameter => write!(f, "culture parameter"),
            CulturePillar => write!(f, "culture pillar"),
            CultureTradition => write!(f, "culture tradition"),
            Decision => write!(f, "decision"),
            Doctrine => write!(f, "doctrine"),
            DoctrineParameter => write!(f, "doctrine parameter"),
            Dynasty => write!(f, "dynasty"),
            DynastyLegacy => write!(f, "dynasty legacy"),
            DynastyPerk => write!(f, "dynasty perk"),
            EducationFocus => write!(f, "education focus"),
            Event => write!(f, "event"),
            Faction => write!(f, "faction"),
            Faith => write!(f, "faith"),
            FaithIcon => write!(f, "faith icon"),
            File => write!(f, "file"),
            GameConcept => write!(f, "game concept"),
            Government => write!(f, "government"),
            GraphicalFaith => write!(f, "graphical faith"),
            Holding => write!(f, "holding"),
            HolySite => write!(f, "holy site"),
            HolySiteFlag => write!(f, "holy site flag"),
            Hook => write!(f, "hook"),
            House => write!(f, "house"),
            Innovation => write!(f, "innovation"),
            InnovationFlag => write!(f, "innovation flag"),
            Inspiration => write!(f, "inspiration"),
            Interaction => write!(f, "interaction"),
            InteractionCategory => write!(f, "interaction category"),
            Language => write!(f, "language"),
            Law => write!(f, "law"),
            Lifestyle => write!(f, "lifestyle"),
            Localization => write!(f, "localization"),
            MenAtArms => write!(f, "men at arms"),
            MenAtArmsBase => write!(f, "men at arms base"),
            Modifier => write!(f, "modifier"),
            Music => write!(f, "music"),
            NameList => write!(f, "name list"),
            Nickname => write!(f, "nickname"),
            Perk => write!(f, "perk"),
            PrisonType => write!(f, "prison type"),
            Province => write!(f, "province"),
            Region => write!(f, "region"),
            Relation => write!(f, "relation"),
            Religion => write!(f, "religion"),
            ReligiousFamily => write!(f, "religious family"),
            Scheme => write!(f, "scheme"),
            ScriptedEffect => write!(f, "effect"),
            ScriptedList => write!(f, "scripted list"),
            ScriptedTrigger => write!(f, "trigger"),
            ScriptValue => write!(f, "script value"),
            Secret => write!(f, "secret"),
            Sexuality => write!(f, "sexuality"),
            Skill => write!(f, "skill"),
            SpecialBuilding => write!(f, "special building"),
            Story => write!(f, "story"),
            Struggle => write!(f, "struggle"),
            StrugglePhase => write!(f, "struggle phase"),
            StrugglePhaseParameter => write!(f, "struggle phase parameter"),
            Terrain => write!(f, "terrain"),
            Title => write!(f, "title"),
            TitleHistory => write!(f, "title history"),
            TitleLaw => write!(f, "title law"),
            TitleLawFlag => write!(f, "title law flag"),
            Tradition => write!(f, "tradition"),
            Trait => write!(f, "trait"),
            UnitGfx => write!(f, "unit gfx"),
            VassalObligation => write!(f, "vassal obligation"),
        }
    }
}
