//! Folding pets into their owners.

use super::encounter_ranking::EncounterRanking;
use super::job_classification::PET_JOB_CODE;
use super::LOCAL_PLAYER_ROW_NAME;

impl EncounterRanking {
    /// Adds the pet at `pet_index` to its owner, and to the local player's row when the owner
    /// is the local character.
    pub(super) fn merge_pet_into_its_owners(&mut self, pet_index: usize) {
        let owner_name = self.players[pet_index].pet_owner_name.clone();
        if self.detected_local_owner_name == owner_name {
            if let Some(local_index) = self.index_of_player(LOCAL_PLAYER_ROW_NAME) {
                self.merge_pet_into_player(local_index, pet_index);
            }
        }
        if let Some(owner_index) = self.index_of_player(&owner_name) {
            self.merge_pet_into_player(owner_index, pet_index);
        }
    }

    fn merge_pet_into_player(&mut self, owner_index: usize, pet_index: usize) {
        let pet = self.players[pet_index].clone_name_and_own_stats();
        // `players` and `encounter` are separate fields, so both can be borrowed at once
        self.players[owner_index].merge_pet_stats(&pet.0, &pet.1, &self.encounter);
    }

    /// The owner's strongest hit / heal becomes the strongest of the owner and their pets.
    pub(super) fn fold_strongest_actions_of_pets(&mut self, local_player_name: &str) {
        for pet_index in 0..self.players.len() {
            if self.players[pet_index].job_code != PET_JOB_CODE {
                continue;
            }
            let owner_name = &self.players[pet_index].pet_owner_name;
            let owner_is_local = owner_name == local_player_name || *owner_name == self.detected_local_owner_name;
            let owner_index = if owner_is_local {
                self.index_of_player(LOCAL_PLAYER_ROW_NAME)
            } else {
                self.index_of_player(owner_name)
            };
            let Some(owner_index) = owner_index else { continue };
            let pet_hit = self.players[pet_index].own_strongest_hit.clone();
            let pet_heal = self.players[pet_index].own_strongest_heal.clone();
            let owner = &mut self.players[owner_index];
            if pet_hit.beats(&owner.merged_strongest_hit) {
                owner.merged_strongest_hit = pet_hit;
            }
            if pet_heal.beats(&owner.merged_strongest_heal) {
                owner.merged_strongest_heal = pet_heal;
            }
        }
    }
}
