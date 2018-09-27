extern crate chrono;

use quick_xml::events::Event;
use quick_xml::Reader;
use regex::Regex;
use self::chrono::prelude::*;
use spotter::InvaderSpotter;
use spotter::Point;
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::Result;
use std::io::prelude::*;
use std::ffi::OsStr;

const COLOR_ACTIVE : &str = "0288D1";
const COLOR_DEAD : &str = "A52714";
const COLOR_UNKNOWN : &str = "757575";
const COLOR_DONE : &str = "67C82F";
const COLOR_INSIDER : &str = "FDDB00";
const COLOR_DIURNAL : &str = "000000";
const COLOR_UNCERTAIN : &str = "FD9C00";

pub fn generate_kml_city(
    map_active: BTreeMap<u16, InvaderSpotter>,
    map_dead: BTreeMap<u16, InvaderSpotter>,
    map_unknonwn: BTreeMap<u16, InvaderSpotter>,
    done_active: BTreeMap<u16, InvaderSpotter>,
    done_dead: BTreeMap<u16, InvaderSpotter>,
    done_unknown: BTreeMap<u16, InvaderSpotter>,
) -> String {
    return generate_kml(map_active, map_dead, map_unknonwn, done_active, done_dead, done_unknown, (0.0, 0.0));
}


pub fn generate_kml_new_city(
    map_active: BTreeMap<u16, InvaderSpotter>,
    map_dead: BTreeMap<u16, InvaderSpotter>,
    map_unknonwn: BTreeMap<u16, InvaderSpotter>,
    coordinates: (f32, f32)
) -> String {
  return generate_kml(map_active, map_dead, map_unknonwn, BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), coordinates);
}

fn generate_kml(
    map_active: BTreeMap<u16, InvaderSpotter>,
    map_dead: BTreeMap<u16, InvaderSpotter>,
    map_unknonwn: BTreeMap<u16, InvaderSpotter>,
    done_active: BTreeMap<u16, InvaderSpotter>,
    done_dead: BTreeMap<u16, InvaderSpotter>,
    done_unknown: BTreeMap<u16, InvaderSpotter>,
    coordinates: (f32, f32)
) -> String {
    let mut ret = "
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<kml xmlns=\"http://www.opengis.net/kml/2.2\">
  <Document>
    <name>Invaders map</name>
    <description/>".to_string();

    ret = ret + &style_map(&format!("1739-{}-nodesc", COLOR_DONE)) + &style_map(&format!("1739-{}-nodesc", COLOR_DEAD)) + &style_map(&format!("1739-{}-nodesc", COLOR_UNKNOWN));
    ret = ret + &style_map(&format!("1899-{}", COLOR_ACTIVE)) + &style_map(&format!("1899-{}", COLOR_UNKNOWN)) + &style_map(&format!("1899-{}", COLOR_DEAD));
    ret = ret + &style_map(&format!("1899-{}", COLOR_INSIDER)) + &style_map(&format!("1899-{}", COLOR_DIURNAL)) + &style_map(&format!("1899-{}", COLOR_UNCERTAIN));

    ret = ret + &generate_kml_folder(map_active, coordinates, "Active");
    ret = ret + &generate_kml_folder(map_dead, coordinates, "Dead");
    ret = ret + &generate_kml_folder(map_unknonwn, coordinates, "Unknown");

    ret = ret + &generate_kml_folder_done(done_active, done_dead, done_unknown);

    ret = ret + &"
  </Document>
</kml>".to_string();

    ret
}

fn generate_kml_folder(map: BTreeMap<u16, InvaderSpotter>, coordinates: (f32, f32), label: &str) -> String {
  let mut ret = "".to_string();
  
  let total : u16 = map.values().cloned().fold(0, |mut sum, si| {sum += si.points.unwrap_or(0); sum});
  println!("Total {} : {} pts / {} invaders", label, total, map.len());
  ret = ret + &format!("
  <Folder>
    <visibility>0</visibility>
    <name>{} - {} pts</name>", label, total).to_string();
  for (_, value) in map.into_iter() {
      ret = ret + &inv_to_kml(value.clone(), &define_icon_custom(label, &value.city_detail, value.id.unwrap_or(0)), coordinates);
  }
  ret = ret + "</Folder>";

  return ret;
}


pub fn generate_kml_folder_collab(map: BTreeMap<u16, InvaderSpotter>, todo_collab: BTreeMap<Vec<String>, Vec<u16>>, players: Vec<String>) -> String {
  let mut ret = "
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<kml xmlns=\"http://www.opengis.net/kml/2.2\">
  <Document>
    <name>Carte collaborative</name>
    <description/>".to_string();

  ret = ret + &style_map("1899-e6194b") + &style_map("1899-3cb44b") + &style_map("1899-ffe119") + &style_map("1899-4363d8") + &style_map("1899-f58231");
  ret = ret + &style_map("1899-911eb4") + &style_map("1899-46f0f0") + &style_map("1899-f032e6") + &style_map("1899-008080") + &style_map("1899-aa6e28") + &style_map("1899-e6beff");

  for (group, invaders) in todo_collab.into_iter() {
    ret = ret + &format!("
    <Folder>
      <open>0</open>
      <name>{}</name>", group.clone().join(", ")).to_string();
    for i in invaders.into_iter() {
      // println!("{} = {:?}", i, group.clone());
      ret = ret + &inv_to_kml(map.get(&i).unwrap().clone(), &define_icon_nb(group.clone(), &players), (0.0, 0.0));
    }
    ret = ret + "</Folder>";
  }

  ret = ret + &"
  </Document>
</kml>".to_string();

  return ret;
}

pub fn kml_to_map(_v: bool) {
  let paths = fs::read_dir("./maps/").unwrap();

    let names = paths
        .filter_map(Result::ok)
        .filter(|d| d.path().extension().unwrap_or(OsStr::new("")).to_str() == Some("kml") )
        .map(|s| s.file_name().to_string_lossy().into_owned())
        .collect::<Vec<String>>();

    // println!("{:?}", names);
    for file_kml in names {
        // println!("{:?}", file_kml);
        let mut reader = Reader::from_file("./maps/".to_owned() + &file_kml).unwrap();
        reader.trim_text(true);

        let mut in_placemark = false;
        let mut capture = false;
        let mut txt = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"Placemark" => in_placemark = true,
                    b"name" | b"coordinates" => capture = if in_placemark { true } else { false },
                    _ => (),
                },
                Ok(Event::End(ref e)) => match e.name() {
                    b"Placemark" => in_placemark = false,
                    b"name" | b"coordinates" => capture = false,
                    _ => (),
                },
                Ok(Event::Text(e)) => if capture {
                    txt.push(e.unescape_and_decode(&reader).unwrap());
                },
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (), // There are several other `Event`s we do not consider here
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
        }

        if _v {
          println!("{:?}", txt);
        }
        
        let mut tree = BTreeMap::new();
        while let Some(coord) = txt.pop() {
            let id = txt.pop().unwrap();

            let regex = Regex::new(r"(.+)_(?P<id>[0-9?]{2,})").unwrap();
            let long_lat = Regex::new(r"(?P<long>[0-9\.?-]{2,}),(?P<lat>[0-9\.?-]{2,}),").unwrap();

            let coordinates = match long_lat.captures(&coord) {
                Some(c) => c,
                None => panic!("Issue on parsing \"{}\"", &coord),
            };

            match regex.captures(&id) {
                Some(c) => {
                  tree.insert(
                    c["id"].parse::<u16>().ok().unwrap(),
                    (
                        id.clone(),
                        coordinates["long"].parse::<f32>().ok().unwrap(),
                        coordinates["lat"].parse::<f32>().ok().unwrap(),
                    ),
                  );
                },
                _ => println!("Issue on parsing \"{}\"", &id),
            };
        }

        // println!("{:?}", tree);

        let mut csv = "".to_string();
        for (i, (name, long, lat)) in tree {
            csv = csv
                + &i.to_string()
                + ";"
                + &name
                + ";"
                + &long.to_string()
                + ";"
                + &lat.to_string()
                + ";
";
        }

        let filename = file_kml.split('.').collect::<Vec<&str>>()[0].to_string();
        fs::remove_file("./maps/".to_owned() + &file_kml).ok();

        let mut file = File::create("./maps/".to_owned() + &filename + ".map").unwrap();
        file.write_all(csv.as_bytes()).ok();
    }
}

fn define_icon_nb(raf: Vec<String>, players: &Vec<String>) -> String {
  let colors = vec!["ffffff", "e6194b", "3cb44b", "ffe119", "4363d8", "f58231", "911eb4", "46f0f0"];

  let mut nb = 0;
  for i in 0..players.len() {
    if raf.contains(players.get(i).unwrap()) {
      nb = nb + u32::pow(2, i as u32);
    }
  }

  // '#e6194b', '#3cb44b', '#ffe119', '#4363d8', '#f58231', '#911eb4', '#46f0f0', '#f032e6', '#bcf60c', '#fabebe', '#008080', '#e6beff', '#9a6324', '#fffac8', '#800000', '#aaffc3', '#808000', '#ffd8b1', '#000075', '#808080', '#ffffff', '#000000'
  format!("#icon-1899-{}", colors.get(nb as usize).unwrap())
}

fn generate_kml_folder_done(active: BTreeMap<u16, InvaderSpotter>, dead: BTreeMap<u16, InvaderSpotter>, unknown: BTreeMap<u16, InvaderSpotter>) -> String {
  let mut ret = "".to_string();
  
  let tot_active : u16 = active.values().cloned().fold(0, |mut sum, si| {sum += si.points.unwrap_or(0); sum});
  let tot_dead : u16 = dead.values().cloned().fold(0, |mut sum, si| {sum += si.points.unwrap_or(0); sum});
  let tot_unknown : u16 = unknown.values().cloned().fold(0, |mut sum, si| {sum += si.points.unwrap_or(0); sum});
  println!("Total Done : {} pts / {} invaders", tot_active + tot_dead + tot_unknown, active.len() + dead.len() + unknown.len());

  ret = ret + &format!("
  <Folder>
    <visibility>0</visibility>  
    <name>Done - {} pts</name>", tot_active + tot_dead + tot_unknown).to_string();
  for (_, value) in active.into_iter() {
      ret = ret + &inv_to_kml(value, &define_icon("Done-Active"), (0.0, 0.0));
  }
  for (_, value) in dead.into_iter() {
      ret = ret + &inv_to_kml(value, &define_icon("Done-Dead"), (0.0, 0.0));
  }
  for (_, value) in unknown.into_iter() {
      ret = ret + &inv_to_kml(value, &define_icon("Done-Unknown"), (0.0, 0.0));
  }
  ret = ret + "</Folder>";

  return ret;
}


fn define_icon(label: &str) -> String {
  return define_icon_custom(label, "", 0);
}

fn define_icon_custom(label: &str, city: &str, id: u16) -> String {
  match label {
    "Active" => {
      let custom = InvaderSpotter::get_custom_file(city);
      // println!("{:?}", custom);
      if custom.uncertains != None && custom.uncertains.unwrap().contains(&id) {
        format!("#icon-1899-{}", COLOR_UNCERTAIN)
      } else if custom.diurnals != None && custom.diurnals.unwrap().contains(&id) {
        format!("#icon-1899-{}", COLOR_DIURNAL)
      } else if custom.insiders != None && custom.insiders.unwrap().contains(&id) {
        format!("#icon-1899-{}", COLOR_INSIDER)
      } else {
        format!("#icon-1899-{}", COLOR_ACTIVE)
      } 
    }
    "Dead" => { format!("#icon-1899-{}", COLOR_DEAD) } 
    "Done-Active" => { format!("#icon-1739-{}-nodesc", COLOR_DONE) } 
    "Done-Dead" => { format!("#icon-1739-{}-nodesc", COLOR_DEAD) } 
    "Done-Unknown" => { format!("#icon-1739-{}-nodesc", COLOR_UNKNOWN) } 
    _ => { format!("#icon-1899-{}", COLOR_UNKNOWN) } // Matches every string value
  }
}

fn inv_to_kml(invader: InvaderSpotter, icon: &str, (mut lat, mut long): (f32, f32)) -> String {
    if invader.coordinates != Point::null() {
      lat = invader.coordinates.x.unwrap_or(0.0);
      long = invader.coordinates.y.unwrap_or(0.0);
    }

    // long, lat, 0
    format!("
    <Placemark>
        <name>{}_{} [{} pts]</name>
        <description><![CDATA[{} / {} / {}<br/>https://www.instagram.com/explore/tags/{}_{}/]]></description>
        <styleUrl>{}</styleUrl>
        <ExtendedData>
          <Data name=\"gx_media_links\">
            <value>{}</value>
          </Data>
        </ExtendedData>
        <Point>
          <coordinates>
            {}, {}, 0
          </coordinates>
        </Point>
      </Placemark>
    ", 
    invader.city, 
    format!("{:0>2}", invader.id.unwrap_or(9999)), 
    invader.points.unwrap_or(0),
    invader.status_detail,
    invader.update_time.clone().unwrap_or(NaiveDate::from_ymd(1970,1,1)),
    invader.source.clone().unwrap_or("".to_string()),
    invader.city, 
    format!("{:0>2}", invader.id.unwrap_or(9999)), 
    icon,
    invader.links.iter().map(|s| "http://invader.spotter.free.fr/".to_string() + &s).collect::<Vec<String>>().join(" "),
    long,
    lat
    ).to_string()
}


fn style_map(color: &str) -> String {
  format!("<Style id=\"icon-{}-normal\">
      <IconStyle>
        <color>ff1427a5</color>
        <scale>1</scale>
        <Icon>
          <href>http://www.gstatic.com/mapspro/images/stock/503-wht-blank_maps.png</href>
        </Icon>
        <hotSpot x=\"32\" xunits=\"pixels\" y=\"64\" yunits=\"insetPixels\"/>
      </IconStyle>
      <LabelStyle>
        <scale>0</scale>
      </LabelStyle>
    </Style>
    <Style id=\"icon-{}-highlight\">
      <IconStyle>
        <color>ff1427a5</color>
        <scale>1</scale>
        <Icon>
          <href>http://www.gstatic.com/mapspro/images/stock/503-wht-blank_maps.png</href>
        </Icon>
        <hotSpot x=\"32\" xunits=\"pixels\" y=\"64\" yunits=\"insetPixels\"/>
      </IconStyle>
      <LabelStyle>
        <scale>1</scale>
      </LabelStyle>
    </Style>
    <StyleMap id=\"icon-{}\">
      <Pair>
        <key>normal</key>
        <styleUrl>#icon-{}-normal</styleUrl>
      </Pair>
      <Pair>
        <key>highlight</key>
        <styleUrl>#icon-{}-highlight</styleUrl>
      </Pair>
    </StyleMap>", color, color, color, color, color)
}