//! How the `pets` option ("Combine Pets with Owner") becomes a question to the Encounter.

use crate::domain::combat::PetView;

/// The application layer reads the option and decides when to ask; the Encounter only answers.
/// `local_player_name` is empty until ACT has reported it.
pub fn pet_view(fold_pets_into_owners: bool, local_player_name: &str) -> PetView<'_> {
    if fold_pets_into_owners {
        PetView::FoldedIntoOwners { local_player_name: Some(local_player_name).filter(|name| !name.is_empty()) }
    } else {
        PetView::Separate
    }
}
