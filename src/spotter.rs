extern crate chrono;
extern crate rand;
extern crate regex;
extern crate reqwest;
extern crate select;
extern crate serde_json;
extern crate curl;


use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::error::Error;
use std::path::Path;

use self::chrono::prelude::*;
use self::regex::Regex;
use self::select::node::Node;

static FOLDER_CACHE: &'static str = "./cache/";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: Option<f32>,
    pub y: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Custom {
    pub reactivations: Option<Vec<u16>>,
    pub insiders: Option<Vec<u16>>,
    pub diurnals: Option<Vec<u16>>,
    pub uncertains: Option<Vec<u16>>,
}

impl Point {
    pub fn null() -> Point {
        Point { x: None, y: None }
    }

    pub fn from_str(lat: &str, long: &str) -> Point {
        Point {
            x: lat.trim().parse::<f32>().ok(),
            y: long.trim().parse::<f32>().ok(),
        }
    }
}

impl Custom {
    pub fn empty() -> Custom {
        Custom { reactivations: None, insiders: None, diurnals: None, uncertains: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvaderSpotter {
    pub city: String,
    pub city_detail: String,
    pub id: Option<u16>,
    pub status: InvaderStatus,
    pub status_detail: String, // OK / Non localisé / Détruit / Dégradé / Réactivé / Un peu dégradé
    arrondissement: Option<String>,
    pub points: Option<u16>,
    pub update_time: Option<NaiveDate>,
    pub source: Option<String>,
    pub links: Vec<String>,
    pub coordinates: Point,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub enum InvaderStatus {
    Active,
    Dead,
    Unknown,
}

impl InvaderSpotter {
    pub fn get_invaders_by_status(
        city: &str,
        verbose: bool,
    ) -> (
        BTreeMap<u16, InvaderSpotter>,
        BTreeMap<u16, InvaderSpotter>,
        BTreeMap<u16, InvaderSpotter>,
    ) {
        let dt = Local::now();
        if fs::create_dir(FOLDER_CACHE).is_ok() {
            println!("Folder {} was created", FOLDER_CACHE)
        }

        let path = FOLDER_CACHE.to_owned()
            + city
            + "_"
            + &dt.year().to_string()
            + &dt.month().to_string()
            + &dt.day().to_string()
            + ".cache";

        // Open the file in read-only mode.
        let file = File::open(&path);
        let status;
        if file.is_ok() {
            if verbose { println!("Read from cache : {}", path); }
            let br = BufReader::new(file.unwrap());
            let (map_active, map_dead, map_unknown): (
                BTreeMap<u16, InvaderSpotter>,
                BTreeMap<u16, InvaderSpotter>,
                BTreeMap<u16, InvaderSpotter>,
            ) = serde_json::from_reader(br).unwrap();
            status = (map_active, map_dead, map_unknown);
        } else {
            status = InvaderSpotter::generate_list_invaders_by_status(city, verbose);

            let v = serde_json::to_value(&status).unwrap();
            let mut file = File::create(&path).unwrap();
            file.write_all(v.to_string().as_bytes()).ok();
        }

        let (mut active, mut dead, mut unknown) = status;
        // Override Spotter with custom reactivations
        let custom_reactivations = InvaderSpotter::get_custom_file(city).reactivations;
        for (invader_id, mut value) in dead.clone().into_iter() {
            if custom_reactivations != None && custom_reactivations.clone().unwrap().contains(&invader_id) {
                value.status_detail = "Réactivation manuelle / ".to_owned() + &value.status_detail;
                active.insert(invader_id, value);
                dead.remove(&invader_id);
            }
        }
        for (invader_id, mut value) in unknown.clone().into_iter() {
            if custom_reactivations != None && custom_reactivations.clone().unwrap().contains(&invader_id) {
                value.status_detail = "Réactivation manuelle / ".to_owned() + &value.status_detail;
                active.insert(invader_id, value);
                unknown.remove(&invader_id);
            }
        }

        return (active, dead, unknown);
    }

    pub fn generate_list_invaders_by_status(
        city: &str,
        verbose: bool,
    ) -> (
        BTreeMap<u16, InvaderSpotter>,
        BTreeMap<u16, InvaderSpotter>,
        BTreeMap<u16, InvaderSpotter>,
    ) {

        println!("Get Invaders by Status for {}", city);
        const URL: &str = "http://invader.spotter.free.fr/listing.php";

        // Call Spotter to get the listing
        let resp_init = reqwest::Client::new()
            .post(URL)
            .form(&[("ville", city)])
            .send()
            .unwrap();
        assert!(resp_init.status().is_success());
        let resp = Document::from_read(resp_init).unwrap();

        // Get the max of the pagination
        let max =
            resp.find(
                Attr("id", "contenu")
                    .descendant(Name("p"))
                    .descendant(Name("a")),
            ).last()
                .unwrap()
                .text()
                .parse::<u16>()
                .unwrap_or(1);

        // Parse all the pages and split data in three list: Active / Dead / Unknown
        let mut map_active = BTreeMap::new();
        let mut map_dead = BTreeMap::new();
        let mut map_unknown = BTreeMap::new();

        for page in 1..=max {
            if verbose {
                println!("{} - Parsing page {} / {}", city, page, max);
            }
            let resp = reqwest::Client::new()
                .post(URL)
                .form(&[("ville", city), ("page", &page.to_string())])
                .send()
                .unwrap();
            assert!(resp.status().is_success());
            let resp = Document::from_read(resp).unwrap();

            for node in resp.find(Class("haut")) {
                let invader = InvaderSpotter::from_html(node, city);
                if verbose {
                    println!("{:?}", invader);
                }

                let invader_id = invader.id.unwrap_or(9000 + rand::random::<u8>() as u16);
                if invader.status == InvaderStatus::Active {
                    map_active.insert(
                        invader_id,
                        invader,
                    );
                } else if invader.status == InvaderStatus::Dead {
                    map_dead.insert(
                        invader_id,
                        invader,
                    );
                } else {
                    map_unknown.insert(
                        invader_id,
                        invader,
                    );
                }
            }
        }

        if verbose {
            println!("MAP Active : {:?}", map_active);
            println!("MAP Dead : {:?}", map_dead);
            println!("MAP Unknown : {:?}", map_unknown);
        }

        return (map_active, map_dead, map_unknown);
    }

    pub fn get_custom_file(city: &str) -> Custom {
        let path = format!("./maps/{}_custom.json", city).to_string();

        return match InvaderSpotter::read_custom_element_from_file(path) {
            Ok(s)   => s,
            _       => Custom::empty()
        };
    }

    fn read_custom_element_from_file<P: AsRef<Path>>(path: P) -> Result<Custom, Box<Error>> {
        let file = File::open(path)?;
        let c = serde_json::from_reader(file)?;
        Ok(c)
    }

    fn from_html(node: Node, city: &str) -> InvaderSpotter {
        let infos = node.find(Class("normal")).next().unwrap().text();
        // println!("{:?}", infos);

        // Regex to split the informations (warning parsing specific for Paris)
        let caps = InvaderSpotter::get_regex(city).captures(&infos).unwrap();
        let (state, date, source) = InvaderSpotter::get_state_and_update(&caps["state"]);

        // Get the IMG URLs (One or 2)
        let mut imgs = node
            .find(Name("img"))
            .filter_map(|n| n.attr("src"))
            .collect::<Vec<_>>();
        imgs.append(
            &mut node
                .find(Name("a"))
                .filter_map(|n| n.attr("href"))
                .collect::<Vec<_>>(),
        );

        InvaderSpotter {
            city: caps["city"].to_string(),
            city_detail: city.to_string(),
            id: caps["id"].parse::<u16>().ok(),
            status: InvaderSpotter::get_status(&state),
            status_detail: state.to_string(),
            arrondissement: InvaderSpotter::get_arrondissement(city, &caps),
            points: caps["pts"].parse::<u16>().ok(),
            update_time: date,
            source: source,
            links: imgs
                .iter()
                .rev()
                .filter(|path| !path.starts_with("images/spot"))
                .cloned()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            coordinates: Point { x: None, y: None },
        }
    }

    fn get_state_and_update(state: &str) -> (String, Option<NaiveDate>, Option<String>) {
        let re = Regex::new(r"(.+)Date et source : (.+) \((.+)\)").unwrap();

        match re.captures(state) {
            Some(caps) => {
                let date_french: Vec<&str> = caps.get(2).unwrap().as_str().split(" ").collect();
                return (
                    caps.get(1).unwrap().as_str().to_string(),
                    Some(NaiveDate::from_ymd(
                        date_french[1].parse::<i32>().unwrap(),
                        InvaderSpotter::french_month_to_int(date_french[0]),
                        1,
                    )),
                    Some(caps.get(3).unwrap().as_str().to_string()),
                );
            }
            None => {
                return (state.to_string(), None, None);
            }
        }
    }

    fn get_regex(city: &str) -> regex::Regex {
        if city == "paris" {
            return Regex::new(
                r"(?x)
        (?P<city>[A-Z]{2,})
        _
        (?P<id>[0-9?]{2,})
        \s\[
        (?P<pts>[0-9?]{2,})
        \spts\]
        \(
        (?P<arrondissement>.{2,})
        \).+:\s\s
        (?P<state>.{2,})",
            ).unwrap();
        }

        return Regex::new(
            r"(?x)
        (?P<city>[A-Z]{2,})
        _
        (?P<id>[0-9?x]{2,})
        \s\[
        (?P<pts>[0-9?]{2,})
        \spts\].+:\s\s
        (?P<state>.{2,})",
        ).unwrap();
    }

    fn get_arrondissement(city: &str, caps: &regex::Captures) -> Option<String> {
        if city == "paris" {
            return Some(caps["arrondissement"].to_string());
        }
        None
    }

    fn get_status(status_detail: &str) -> InvaderStatus {
        // OK / Non localisé / Détruit / Dégradé / Réactivé / Un peu dégradé
        match status_detail {
            "OK" | "Dégradé" | "Réactivé" | "Un peu dégradé" => InvaderStatus::Active,
            "Non localisé" | "Non visible" => InvaderStatus::Unknown,
            _ => InvaderStatus::Dead,
        }
    }

    fn french_month_to_int(month: &str) -> u32 {
        match month {
            "janvier" => 1,
            "février" => 2,
            "mars" => 3,
            "avril" => 4,
            "mai" => 5,
            "juin" => 6,
            "juillet" => 7,
            "août" => 8,
            "septembre" => 9,
            "octobre" => 10,
            "novembre" => 11,
            "décembre" => 12,
            _ => 0,
        }
    }
}
