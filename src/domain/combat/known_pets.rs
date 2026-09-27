//! Pet names as the game shows them, in every client language (EN, DE, FR, JA, KO, ZH).
//!
//! Taken from mopimopi (`js/core.js`), plus Cu Sith for Beastmaster, which is newer than
//! mopimopi's list. Add names here when a new pet or job appears.

/// The jobs that have pets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PetJob {
    Summoner,
    Scholar,
    Machinist,
    DarkKnight,
    Ninja,
    Astrologian,
    WhiteMage,
    Sage,
    Beastmaster,
}

pub fn job_of_pet(pet_name: &str) -> Option<PetJob> {
    PETS_BY_JOB
        .iter()
        .find(|(_, names)| names.contains(&pet_name))
        .map(|(job, _)| *job)
}

const PETS_BY_JOB: &[(PetJob, &[&str])] = &[
    (PetJob::Summoner, SUMMONER_PETS),
    (PetJob::Scholar, SCHOLAR_PETS),
    (PetJob::Machinist, MACHINIST_PETS),
    (PetJob::DarkKnight, DARK_KNIGHT_PETS),
    (PetJob::Ninja, NINJA_PETS),
    (PetJob::Astrologian, ASTROLOGIAN_PETS),
    (PetJob::WhiteMage, WHITE_MAGE_PETS),
    (PetJob::Sage, SAGE_PETS),
    (PetJob::Beastmaster, &[]),
];

const SUMMONER_PETS: &[&str] = &[
    "카벙클 에메랄드", "カーバンクル・エメラルド", "绿宝石兽", "Smaragd-Karfunkel", "Carbuncle émeraude", "Emerald Carbuncle",
    "카벙클 토파즈", "カーバンクル・トパーズ", "黄宝石兽", "Topas-Karfunkel", "Carbuncle topaze", "Topaz Carbuncle",
    "카벙클 루비", "カーバンクル・ルビー", "红宝石兽", "Rubin-Karfunkel", "Carbuncle rubis", "Ruby Carbuncle",
    "가루다 에기", "ガルーダ・エギ", "迦楼罗之灵", "Garuda-Egi",
    "이프리트 에기", "イフリート・エギ", "伊弗利特之灵", "Ifrit-Egi",
    "타이탄 에기", "タイタン・エギ", "泰坦之灵", "Titan-Egi",
    "데미바하무트", "デミ・バハムート", "亚灵神巴哈姆特", "Demi-Bahamut",
    "데미피닉스", "デミ・フェニックス", "Demi-Phönix", "Demi-Phénix", "Demi-Phoenix", "亚灵神不死鸟",
    "Ruby Ifrit", "Ifrit rubis", "Rubin-Ifrit", "イフリート・ルビー", "이프리트 루비", "伊芙利特之灵",
    "Topaz Titan", "Titan topaze", "Topas-Titan", "タイタン・トパーズ", "타이탄 토파즈",
    "Emerald Garuda", "Garuda émeraude", "Smaragd-Garuda", "ガルーダ・エメラルド", "가루다 에메랄드",
    "카벙클", "カーバンクル", "Karfunkel", "Carbuncle", "宝石兽",
    "솔 바하무트", "Solar Bahamut", "ソルバハムート", "Sol-Bahamut",
];

const SCHOLAR_PETS: &[&str] = &[
    "요정 에오스", "フェアリー・エオス", "朝日小仙女", "Eos",
    "요정 셀레네", "フェアリー・セレネ", "夕月小仙女", "Selene",
    "セラフィム", "Seraph", "Séraphin", "炽天使", "세라핌",
];

const MACHINIST_PETS: &[&str] = &[
    "자동포탑 룩", "オートタレット・ルーク", "车式浮空炮塔", "Selbstschuss-Gyrocopter Turm", "Auto-tourelle Tour", "Rook Autoturret",
    "자동포탑 비숍", "オートタレット・ビショップ", "象式浮空炮塔", "Selbstschuss-Gyrocopter Läufer", "Auto-tourelle Fou", "Bishop Autoturret",
    "オートマトン・クイーン", "Automaton Dame", "Automate Reine", "Automaton Queen", "后式自走人偶", "자동인형 퀸",
];

const DARK_KNIGHT_PETS: &[&str] = &["영웅의 환영", "英雄の影身", "Hochachtung", "Estime", "Esteem", "英雄的掠影"];

const NINJA_PETS: &[&str] = &["分身", "Gedoppeltes Ich", "Ombre", "Bunshin", "분신"];

const ASTROLOGIAN_PETS: &[&str] = &["지상의 별", "アーサリースター", "地星", "Earthly Star", "Étoile terrestre", "Irdischer Stern"];

const WHITE_MAGE_PETS: &[&str] = &[
    "Liturgic Bell", "liturgic bell", "リタージー・オブ・ベル", "Tintinnabule", "tintinnabule", "Glockenspiel", "예배종", "礼仪之铃",
];

const SAGE_PETS: &[&str] = &["ペプシス", "Pepsis", "소화 작용", "消化"];
