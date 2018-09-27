use regex::Regex;
use spotter::{InvaderSpotter, Point};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::BufReader;
use std::io::Result;
use std::io::prelude::*;

pub fn map_by_status(city: &str, verbose: bool)  -> (
    BTreeMap<u16, InvaderSpotter>,
    BTreeMap<u16, InvaderSpotter>,
    BTreeMap<u16, InvaderSpotter>,
) {
    // Get status from spotter
    let (active, dead, unknown) = InvaderSpotter::get_invaders_by_status(city, verbose);

    let hashmap_geo_json = self::read_map(city);
    return self::enrich_with_coordinates((active, dead, unknown), hashmap_geo_json.unwrap());
}

pub fn map_active(city: &str, verbose: bool) -> (BTreeMap<u16, InvaderSpotter>) {
    // Get status from spotter
    let (active, _, _) = InvaderSpotter::get_invaders_by_status(city, verbose);

    let hashmap_geo_json = self::read_map(city);
    return self::update_coordinate(active, &hashmap_geo_json.unwrap());
}

fn enrich_with_coordinates(
    (a, d, u): (
        BTreeMap<u16, InvaderSpotter>,
        BTreeMap<u16, InvaderSpotter>,
        BTreeMap<u16, InvaderSpotter>,
    ),
    hash: HashMap<u16, Point>,
) -> (
    BTreeMap<u16, InvaderSpotter>,
    BTreeMap<u16, InvaderSpotter>,
    BTreeMap<u16, InvaderSpotter>,
) {
    return (
        update_coordinate(a, &hash),
        update_coordinate(d, &hash),
        update_coordinate(u, &hash),
    );
}

fn update_coordinate(
    map: BTreeMap<u16, InvaderSpotter>,
    hash: &HashMap<u16, Point>,
) -> BTreeMap<u16, InvaderSpotter> {
    let mut b = BTreeMap::new();
    for (k, mut v) in map.into_iter() {
        let mut i = v.clone();
        i.coordinates = hash.get(&k).unwrap_or(&Point::null()).clone();

        if i.coordinates == Point::null() {
            println!("{} is not in the map file", k);
        }

        b.insert(k, i);
    }

    return b;
}

fn read_map(city: &str) -> Result<HashMap<u16, Point>> {
    let file = File::open("maps/".to_string() + city + ".map");
    if file.is_ok() {
        let mut hash = HashMap::new();

        let re = Regex::new(
            r"(?x)
(?P<id>\d+)
;
(?P<desc>.+)
;
(?P<long>[0-9\.\s-]+)
;
(?P<lat>[0-9\.\s-]+)
",
        ).unwrap();

        for line in BufReader::new(file.unwrap()).lines() {
            let l = line?;
            match re.captures(&l) {
                Some(caps) => {
                    hash.insert(
                        caps["id"].parse::<u16>().ok().unwrap(),
                        Point::from_str(&caps["lat"], &caps["long"]),
                    );
                }
                None => panic!("Issue on reading \"{}\"", &l),
            }
        }
        // println!("{:?}", hash);

        Ok(hash)
    } else {
        panic!("No file found in folder './map' for {}", city);
    }
}
