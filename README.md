Install Rust

cargo run -- Basel’


cross build --release --target x86_64-pc-windows-gnu

[![Build Status](https://travis-ci.com/dcuenot/spotter.svg?branch=master)](https://travis-ci.com/dcuenot/spotter)

Travis is used to package the application
Retry with tags

# Features
* Generate shell for a new city
* Update map for a city
* Generate a collaborative map
* Transform a .kml to .map
* Calculate remaining points to do around the different cities in the world

## Generate new city
`./invaders newcity <city> <latitude> <longitude>`

## Update a city
`./invaders update <city> <player>`

## List already flashed
Create a Json file in the folder `flashed` with the same format as `cdams.json`
Update directly the file on Gitlab


## Add custom information on the map (only for Paris)
### Custom reactivations
Very useful when Instagram is quicker than Spotter.

Add the id of the invader in the block **reactivations** in the file `paris_custom.json` in the folder `maps`

### Insiders
_Feature coming soon_

Add the id of the invader in the block **insiders** in the file `paris_custom.json` in the folder `maps`

### Diurnals
_Feature coming soon_

Add the id of the invader in the block **diurnals** in the file `paris_custom.json` in the folder `maps`



# Configuration Rust