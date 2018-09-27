extern crate serde;
extern crate serde_json;

use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};
use std::path::Path;

use spotter::InvaderSpotter;


static FLASHED_DIRECTORY: &'static str = "./flashed";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InvadersFlashed {
    city: String, // Name of the city in spotter
    ids: Vec<u16>,
}

impl InvadersFlashed {

    fn read_flashed_invaders_from_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<Vec<InvadersFlashed>, Box<Error>> {
        // Open the file in read-only mode.
        let file = File::open(path)?;

        // Read the JSON contents of the file as an instance of `InvadersFlashed`.
        let si = serde_json::from_reader(file)?;

        // Return the `InvadersFlashed`.
        Ok(si)
    }

    pub fn read_already_flashed(city: &str, player: String, verbose: bool) -> Vec<u16> {
        let file = format!("{}/{}.json", FLASHED_DIRECTORY, player).to_string();
        if verbose { println!("Already flashed file: {:#?}", file); }

        let si: Vec<InvadersFlashed> = match InvadersFlashed::read_flashed_invaders_from_file(file.clone()) {
            Ok(s)   => s,
            _       => panic!("File {} not found", file)
        };

        if verbose {
            println!("Space invader {:#?}", si);
        }

        let mut res = Vec::new();
        for value in si.into_iter() {
            if value.city == city {
                res = value.ids;
            }
        }

        return res;
    }

    pub fn split_done_todo(
        map: BTreeMap<u16, InvaderSpotter>,
        flashed: &Vec<u16>,
    ) -> (BTreeMap<u16, InvaderSpotter>, BTreeMap<u16, InvaderSpotter>) {
        let mut map_done = BTreeMap::new();
        let mut map_todo = BTreeMap::new();

        for (key, value) in map.into_iter() {
            if flashed.contains(&key) {
                map_done.insert(key, value);
            } else {
                map_todo.insert(key, value);
            }
        }

        return (map_todo, map_done);

    }

    pub fn get_player(player: Option<String>) -> String {
        return match player {
            Some(s) => s,
            _ => InvadersFlashed::get_player_from_directory(FLASHED_DIRECTORY),
        };
    }

    fn get_player_from_directory(dir: &str) -> String {
        let paths = fs::read_dir(dir).unwrap();

        let names = paths
            .filter_map(Result::ok)
            .filter(|d| d.path().extension().unwrap().to_str() == Some("json") )
            .map(|s| s.file_name().to_string_lossy().into_owned())
            .collect::<Vec<String>>();

        // println!("{:#?}", names);
        return match names.len() {
            1 => names[0].split('.').collect::<Vec<&str>>()[0].to_string(),
            _ => panic!("No json file found in flashed folder"),
        };
    }
}
