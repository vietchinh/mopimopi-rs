//! Known pet names in every game language (KR, JP, CN, DE, FR, EN), per owning job.

use super::player_role::PlayerRole;

/// A job whose pets ACT reports as separate combatants.
pub struct PetOwnerJob {
    pub class_code: &'static str,
    /// Role forced on the pet; `None` keeps the default (damage).
    pub role: Option<PlayerRole>,
    pub names: &'static [&'static str],
}

const SUMMONER_PETS: &[&str] = &[
    "카벙클 에메랄드", "カーバンクル・エメラルド", "绿宝石兽", "Smaragd-Karfunkel", "Carbuncle émeraude", "Emerald Carbuncle",
    "카벙클 토파즈", "カーバンクル・トパーズ", "黄宝石兽", "Topas-Karfunkel", "Carbuncle topaze", "Topaz Carbuncle",
    "카벙클 루비", "カーバンクル・ルビー", "红宝石兽", "Rubin-Karfunkel", "Carbuncle rubis", "Ruby Carbuncle",
    "가루다 에기", "ガルーダ・エギ", "迦楼罗之灵", "Garuda-Egi",
    "이프리트 에기", "イフリート・エギ", "伊弗利特之灵", "Ifrit-Egi",
    "타이탄 에기", "タイタン・エギ", "泰坦之灵", "Titan-Egi",
    "데미바하무트", "デミ・バハムート", "亚灵神巴哈姆特", "Demi-Bahamut", "デミ・フェニックス",
    "데미피닉스", "Demi-Phönix", "Demi-Phénix", "Demi-Phoenix", "亚灵神不死鸟",
    "Ruby Ifrit", "Ifrit rubis", "Rubin-Ifrit", "イフリート・ルビー", "이프리트 루비", "伊芙利特之灵",
    "Topaz Titan", "Titan topaze", "Topas-Titan", "タイタン・トパーズ", "타이탄 토파즈", "泰坦之灵",
    "Emerald Garuda", "Garuda émeraude", "Smaragd-Garuda", "ガルーダ・エメラルド", "가루다 에메랄드", "迦楼罗之灵",
    "카벙클", "カーバンクル", "Karfunkel", "Carbuncle", "宝石兽",
    "솔 바하무트", "Solar Bahamut", "ソルバハムート", "Sol-Bahamut",
];

const MACHINIST_PETS: &[&str] = &[
    "자동포탑 룩", "オートタレット・ルーク", "车式浮空炮塔", "Selbstschuss-Gyrocopter Turm", "Auto-tourelle Tour", "Rook Autoturret",
    "자동포탑 비숍", "オートタレット・ビショップ", "象式浮空炮塔", "Selbstschuss-Gyrocopter Läufer", "Auto-tourelle Fou", "Bishop Autoturret",
    "オートマトン・クイーン", "Automaton Dame", "Automate Reine", "Automaton Queen", "后式自走人偶", "자동인형 퀸",
];

const SCHOLAR_PETS: &[&str] = &[
    "요정 에오스", "フェアリー・エオス", "朝日小仙女", "Eos",
    "요정 셀레네", "フェアリー・セレネ", "夕月小仙女", "Selene",
    "セラフィム", "Seraph", "Séraphin", "炽天使", "세라핌",
];

const DARK_KNIGHT_PETS: &[&str] = &["영웅의 환영", "英雄の影身", "Hochachtung", "Estime", "Esteem", "英雄的掠影"];

const NINJA_PETS: &[&str] = &["分身", "Gedoppeltes Ich", "Ombre", "Bunshin", "분신"];

const ASTROLOGIAN_PETS: &[&str] = &["지상의 별", "アーサリースター", "地星", "Earthly Star", "Étoile terrestre", "Irdischer Stern"];

const WHITE_MAGE_PETS: &[&str] = &[
    "Liturgic Bell", "liturgic bell", "リタージー・オブ・ベル", "Tintinnabule", "tintinnabule", "Glockenspiel", "예배종", "礼仪之铃",
];

const SAGE_PETS: &[&str] = &["ペプシス", "Pepsis", "소화 작용", "消化"];

/// Order matters: the first job whose list contains the name wins.
pub const PET_OWNER_JOBS: &[PetOwnerJob] = &[
    PetOwnerJob { class_code: "SMN", role: None, names: SUMMONER_PETS },
    PetOwnerJob { class_code: "SCH", role: Some(PlayerRole::Healer), names: SCHOLAR_PETS },
    PetOwnerJob { class_code: "MCH", role: None, names: MACHINIST_PETS },
    PetOwnerJob { class_code: "DRK", role: Some(PlayerRole::Tank), names: DARK_KNIGHT_PETS },
    PetOwnerJob { class_code: "NIN", role: None, names: NINJA_PETS },
    PetOwnerJob { class_code: "AST", role: Some(PlayerRole::Healer), names: ASTROLOGIAN_PETS },
    PetOwnerJob { class_code: "WHM", role: Some(PlayerRole::Healer), names: WHITE_MAGE_PETS },
    PetOwnerJob { class_code: "SGE", role: Some(PlayerRole::Healer), names: SAGE_PETS },
];

/// Finds which job a pet belongs to from its display name (without the "(Owner)" suffix).
pub fn find_pet_owner_job(pet_name: &str) -> Option<&'static PetOwnerJob> {
    PET_OWNER_JOBS.iter().find(|job| job.names.contains(&pet_name))
}
