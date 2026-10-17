use super::*;
use crate::domain::settings::Settings;

#[test]
fn by_default_npcs_look_like_the_original() {
    assert!(!PortPreferences::default().npc_own_look);
}

#[test]
fn a_code_round_trips_and_starts_with_the_prefix() {
    let preferences = PortPreferences { npc_own_look: true, npc_color: "112233".into() };
    let code = preferences.to_code();
    assert!(code.starts_with(PORT_CODE_PREFIX));
    assert_eq!(PortPreferences::from_code(&code), Ok(preferences));
}

#[test]
fn a_mopimopi_shareable_code_is_refused_and_a_port_code_never_reaches_the_settings() {
    let mopimopi_code = Settings::defaults().export_shareable_code();
    assert_eq!(PortPreferences::from_code(&mopimopi_code), Err(InvalidPortCode));
    let mut settings = Settings::defaults();
    let before = settings.export_shareable_code();
    let port_code = PortPreferences { npc_own_look: true, ..PortPreferences::default() }.to_code();
    assert!(settings.import_shareable_code(&port_code).is_err());
    assert_eq!(settings.export_shareable_code(), before);
}

#[test]
fn the_shareable_code_has_no_port_keys() {
    let code = Settings::defaults().export_shareable_code();
    assert!(!code.contains("npc") && !code.contains("Npc"));
}

#[test]
fn a_bad_colour_is_replaced_by_the_default_and_a_damaged_store_by_the_defaults() {
    let code = format!("{PORT_CODE_PREFIX}{{\"npcOwnLook\":true,\"npcColor\":\"nope\"}}");
    assert_eq!(PortPreferences::from_code(&code).unwrap().npc_color, DEFAULT_NPC_COLOR);
    assert_eq!(PortPreferences::from_stored(Some("{broken")), PortPreferences::default());
    assert_eq!(PortPreferences::from_stored(None), PortPreferences::default());
}
