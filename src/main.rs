mod flashed;
mod kml;
mod map;
mod spotter;
mod collab;

#[macro_use]
extern crate structopt;
#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate select;
extern crate serde_json;
extern crate quick_xml;

use flashed::InvadersFlashed;
use spotter::InvaderSpotter;
use collab::InvadersCollab;
use std::fs::File;
use std::io::prelude::*;
use structopt::StructOpt;
use structopt::clap::AppSettings;

use std::io::BufReader;
use std::collections::{BTreeMap};

#[derive(StructOpt, Debug)]
#[structopt(raw(global_setting = "AppSettings::AllowNegativeNumbers"))]
struct Options {
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
    #[structopt(subcommand)]
    cmd: Opt,
}

#[derive(StructOpt, Debug)]
#[structopt(raw(global_setting = "AppSettings::AllowNegativeNumbers"))]
enum Opt {
    #[structopt(name = "newcity")]
    /// Generate a KML for a new city
    NewCity {
        #[structopt(help = "Name of city in Spotter")]
        city: String,
        #[structopt(help = "Latitude", default_value = "0.0")]
        lat: f32,
        #[structopt(help = "Longitude", default_value = "0.0")]
        long: f32,
    },
    #[structopt(name = "update")]
    /// Update a city
    UpdateCity {
        #[structopt(help = "Name of city in Spotter")]
        city: String,
        #[structopt(help = "Player (Optional)")]
        player: Option<String>,
    },
    // Generate collab map
    #[structopt(name = "collab")]
    Collab {
        #[structopt(help = "Players")]
        players: Vec<String>,
    },
    // Transform a .kml to a .map file
    #[structopt(name = "kml2map")]
    KmlToMap,
    // Calculate remaining points across the world
    #[structopt(name = "calc")]
    Calc {
        #[structopt(help = "Player (Optional")]
        player: Option<String>,
    },
}

fn main() {
    // println!("{:?}", Opt::from_args());
    // println!("{:?}", Options::from_args());

    let command = Options::from_args();

    println!("
           ▄▓▒▒▒▓                   ▄▓▒▒▒▌        
         ▄▓▓▌░░░░                 ▓▓▓▌▄▄▄▌        
        ▐▓▓▓▓▓▓▓▓░░░▐          ▄▓▌░░░▓▓▓▀         
        ▐▀▀▀▀▓▓▓▓░░░▐        ▐▓▓▓▌░░░▀            
           ▄▌░░░░░░░░░░░░░░░░░░░░░░░░░░░░▌        
         ▄▓▓▌░░░▄▄▄▄▄░░░░░░░░░░░░▄▄▄▄▄░░░▌        
      ▄▓▓░░░░░░▐▓▓▓▓▓░░░░░░░░░░░░▓▓▓▓▓░░░░░░░     
    ▄▓▓▓▓░░░░░░▐▓▓▓▓▓░░░░░░░░░░░░▓▓▓▓▓░░░░░░░▄▄▄▄▄
  ▄▓▌░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░▐
▐▓▓▓▌░░▐▓▓▓▓▌░░░░░░░░░░░░░░░░░░░░░░░░░░░░▒▓▓▓▓░░░▐
▓▓▓▓▌░░▐▓▓▓▓▌░░░░░░░░░░░░░░░░░░░░░░░░░░░░▓▓▓▓▓░░░▐
▓▓▓▓▌░░▐▐▓▓▓▌░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░▌▓▓▓▓░░░▐
▓▓▓▓▌░░▐▐▓▓▓▌░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░▌▓▓▓▓░░░▐
▓▓▓▓▓▓▓▀▐▓▓▓▓▓▓▓▌▄▄▄▄▄▄▄▄   ▄▄▄▄▄▄▀▀▀▀▓▓▓▀▓▓▓▓▓▓▓▀
▓▓▓▓▓▀  ▐▓▓▓▓▓▓▓▌░░░░░░░░ ▄▓▓▌░░░░░░░▐▓▀  ▓▓▓▓▀   
            ▓▓▓▓▓▓▓▓▓▓▓▀ ▐▓▓▓▓▓▓▓▓▓▓▓▀            
            ▐▓▓▓▓▓▓▓▓▀   ▐▓▓▓▓▓▓▓▓▓                                                                                               
    ", );

    match command.cmd {
        Opt::NewCity { lat, long, city } => generate_new_city(&city, (lat, long), command.verbose),
        Opt::UpdateCity { city, player } => update_city(&city, player, command.verbose),
        Opt::Collab { players } => generate_collab(players, command.verbose),
        Opt::KmlToMap => kml_to_map(command.verbose),
        Opt::Calc { player } => calc(player, command.verbose),
    }
}

fn generate_new_city(city: &str, coordinates: (f32, f32), verbose: bool) {
    // Get status from spotter
    let (map_active, map_dead, map_unknown) = InvaderSpotter::get_invaders_by_status(city, verbose);

    // Generate KML for a new City
    let kml = kml::generate_kml_new_city(map_active, map_dead, map_unknown, coordinates);

    // Generate the Output in the kml folder
    let mut file = File::create("./kml/".to_owned() + city + ".kml").unwrap();
    file.write_all(kml.as_bytes()).ok();
}

fn update_city(city: &str, player: Option<String>, v: bool) {
    // Get map with coordinates by status
    let (map_active, map_dead, map_unknown) = map::map_by_status(city, v);

    // Split each status by already flashed or not
    let player = InvadersFlashed::get_player(player);

    // Get invaders already flashed by the player
    let already_flashed = InvadersFlashed::read_already_flashed(city, player.clone(), v);

    // Split TODO and Done by status
    let (a_todo, a_done) = InvadersFlashed::split_done_todo(map_active, &already_flashed);
    let (d_todo, d_done) = InvadersFlashed::split_done_todo(map_dead, &already_flashed);
    let (u_todo, u_done) = InvadersFlashed::split_done_todo(map_unknown, &already_flashed);

    // Generate a KML file for update city
    let kml = kml::generate_kml_city(
        a_todo, d_todo, u_todo,
        a_done, d_done, u_done,
    );

    // Generate the Output in the kml folder
    let mut file = File::create("./kml/".to_owned() + city + "_" + &player + ".kml").unwrap();
    file.write_all(kml.as_bytes()).ok();
}

fn generate_collab(players: Vec<String>, verbose: bool) {
    if verbose { println!("Players {:?}", players); }
    let city = "paris";

    // Get map active with coordinates
    let map_active = map::map_active(city, verbose);
    let collab = InvadersCollab::generate(map_active.clone(), players.clone(), city, verbose); 

    let kml = kml::generate_kml_folder_collab(map_active, collab.tree, players.clone());

    // Generate the Output in the kml folder
    let mut file =
        File::create("./kml/".to_owned() + city + "_" + &(players.join("_")) + ".kml").unwrap();
    file.write_all(kml.as_bytes()).ok();
}

fn kml_to_map(v: bool) {
    kml::kml_to_map(v);
}

fn calc(player: Option<String>, v: bool) {
    let file = File::open("src/list_spotter.list");
    if file.is_ok() {
        println!("{0: <11} | {1: <29} | {2: <16} |", 
            "", "----------- Points ----------", "-------- Invaders -------");
        println!("{0: <11} | {1: <5} | {2: <5} | {3: <5} | {4: <5} | {5: <4} | {6: <4} | {7: <4} | {8: <4} |", 
            "city", "act.", "dead", "unk.", "done", "act.", "dead", "unk.", "done");

        for l in BufReader::new(file.unwrap()).lines() {
            let city = l.unwrap();

            // Get map with coordinates by status
            let (map_active, map_dead, map_unknown) = InvaderSpotter::get_invaders_by_status(&city, v);

            let already_flashed = match player.clone() {
                Some(p) => {
                    // Split each status by already flashed or not
                    let player = InvadersFlashed::get_player(Some(p));

                    // Get invaders already flashed by the player
                    InvadersFlashed::read_already_flashed(&city, player.clone(), v)
                },
                _       => vec![]
            };

            // Split TODO and Done by status
            let (a_todo, a_done) = InvadersFlashed::split_done_todo(map_active, &already_flashed);
            let (d_todo, d_done) = InvadersFlashed::split_done_todo(map_dead, &already_flashed);
            let (u_todo, u_done) = InvadersFlashed::split_done_todo(map_unknown, &already_flashed);

            let mut map_done = BTreeMap::new();
            for (key, value) in a_done.into_iter() {
                map_done.insert(key, value);
            }
            for (key, value) in d_done.into_iter() {
                map_done.insert(key, value);
            }
            for (key, value) in u_done.into_iter() {
                map_done.insert(key, value);
            }

            let active : u16 = a_todo.values().cloned().fold(0, |mut sum, si| {sum += si.points.unwrap_or(0); sum});
            let dead : u16 = d_todo.values().cloned().fold(0, |mut sum, si| {sum += si.points.unwrap_or(0); sum});
            let unknown : u16 = u_todo.values().cloned().fold(0, |mut sum, si| {sum += si.points.unwrap_or(0); sum});
            let done : u16 = map_done.values().cloned().fold(0, |mut sum, si| {sum += si.points.unwrap_or(0); sum});


            println!("{0: <11} | {1: <5} | {2: <5} | {3: <5} | {4: <5} | {5: <4} | {6: <4} | {7: <4} | {8: <4} |", 
                city, active, dead, unknown, done, a_todo.len(), d_todo.len(), u_todo.len(), map_done.len());
        }
    }
}

