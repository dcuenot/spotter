extern crate array_tool;

use collab::array_tool::vec::*;
use flashed::InvadersFlashed;
use spotter::InvaderSpotter;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct InvadersCollab {
    pub tree: BTreeMap<Vec<String>, Vec<u16>>
}

impl InvadersCollab {
    pub fn generate(map_active:BTreeMap<u16, InvaderSpotter>, players: Vec<String>, city: &str, verbose: bool) -> InvadersCollab {
        let mut map_collab = BTreeMap::new();

        for (key, _) in map_active.clone().into_iter() {
            map_collab.insert(key, vec![]);
        }

        for player in &players {
            let flashed = InvadersFlashed::read_already_flashed(city, player.clone(), verbose);
            for i in flashed {
                match map_collab.get_mut(&i) {
                    Some(p) => p.push(player.clone()),
                    _ => (),
                };
            }
        }

        // Calculate the difference to define the RAF
        for (i, mut flashed_collab) in map_collab.clone().into_iter() {
            if players.uniq(flashed_collab.clone()).len() == 0 {
                map_collab.remove(&i);
            } else {
                map_collab.insert(i, players.uniq(flashed_collab.clone()));
            }
        }

        // Combine per group
        let mut map_collab_per_group: BTreeMap<Vec<String>, Vec<u16>> = BTreeMap::new();
        for (i, raf_collab) in map_collab.clone().into_iter() {
            if map_collab_per_group.contains_key(&raf_collab) {
                let mut vec = map_collab_per_group.get(&raf_collab).unwrap().clone();
                vec.push(i);
                map_collab_per_group.insert(raf_collab, vec);
            } else {
                map_collab_per_group.insert(raf_collab, vec![i]);
            }
        }

        if verbose {
            println!("Collab map per group: {:?}", map_collab_per_group);
        }

        return InvadersCollab{
            tree: map_collab_per_group,
        };
    }
}
