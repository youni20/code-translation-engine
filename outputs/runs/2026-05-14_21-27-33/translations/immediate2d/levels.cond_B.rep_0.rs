const LEVEL_WIDTH: usize = 16;
const LEVEL_HEIGHT: usize = 12;

#[derive(Debug, Clone, Copy)]
enum Tile {
    Floor,
    Wall,
    Player,
    Coin,
    Exit,
    HorizontalBug,
    VerticalBug,
    Trigger(u8),
    PointOfInterest(char),
    Unknown(char),
}

#[derive(Debug)]
struct Level {
    tiles: [[Tile; LEVEL_WIDTH]; LEVEL_HEIGHT],
    triggers: String,
}

impl Level {
    fn parse_level_string(level_str: &str) -> Self {
        let mut tiles = [[Tile::Floor; LEVEL_WIDTH]; LEVEL_HEIGHT];
        let mut index = 0;
        let mut triggers = String::new();

        for (i, character) in level_str.chars().enumerate() {
            if i < LEVEL_WIDTH * LEVEL_HEIGHT {
                let x = index % LEVEL_WIDTH;
                let y = index / LEVEL_WIDTH;
                tiles[y][x] = match character {
                    ' ' => Tile::Floor,
                    '#' => Tile::Wall,
                    '@' => Tile::Player,
                    '$' => Tile::Coin,
                    '!' => Tile::Exit,
                    '-' => Tile::HorizontalBug,
                    '|' => Tile::VerticalBug,
                    '0'..='9' => Tile::Trigger(character.to_digit(10).unwrap() as u8),
                    'A'..='Z' | 'a'..='z' => Tile::PointOfInterest(character),
                    _ => Tile::Unknown(character),
                };
                index += 1;
            } else {
                triggers.push(character);
            }
        }

        Self { tiles, triggers }
    }
}

const LEVEL_LIST: &[&str] = &[
    "                \
     \n                \
     \n     ######     \
     \n     #@ #$#     \
     \n     ## # #     \
     \n     #$   #     \
     \n     #### #     \
     \n     #!## #     \
     \n     #    #     \
     \n     ######     \
     \n                \
     \n                ",

    "                \
     \n                \
     \n    ########    \
     \n    #@   #!#    \
     \n    #### A #    \
     \n    #    ###    \
     \n    # ## #$#    \
     \n    #  # # #    \
     \n    # 1#   #    \
     \n    ########    \
     \n                \
     \n                \
     \n1:A .",

    "                \
     \n   ##########   \
     \n   # $#-   1#   \
     \n   # #### ###   \
     \n   #      # #   \
     \n   # ## # # #   \
     \n   #  #@#  |#   \
     \n   ## ##### #   \
     \n   #aa#!#   #   \
     \n   #aa#   #$#   \
     \n   ##########   \
     \n                \
     \n1:a$.",

    "      #####     \
     \n      #  |#     \
     \n      #2# #     \
     \n      #1#a#     \
     \n      # # #     \
     \n      #@#|#     \
     \n      ###|#     \
     \n        #$#     \
     \n        # #     \
     \n        #|##    \
     \n        # !#    \
     \n        ####    \
     \n~:2 .\
     \n1:a#22.\
     \n2:a 11.",

    "################\
     \n#   #1####   #a#\
     \n# @ # ####   # #\
     \n#  ## #### |## #\
     \n#     ####  |  #\
     \n### #$###### # #\
     \n#-   |####     #\
     \n# # # #### # # #\
     \n#-  # ####   # #\
     \n# # # #### # # #\
     \n#   # ####!- # #\
     \n################\
     \n1:a@.",
];

fn main() {
    for &level_str in LEVEL_LIST {
        let level = Level::parse_level_string(level_str);
        println!("{:?}", level);
    }
}