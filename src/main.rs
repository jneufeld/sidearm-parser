mod parse;

use parse::{Action, Game};

use clap::{App, Arg};

// Hand parser for text files at:
// http://web.archive.org/web/20110205042259/http://www.outflopped.com/questions/286/obfuscated-datamined-hand-histories
fn main() {
    let matches = App::new("hand-reader")
        .version("0.1.0")
        .author("Jonathan Neufeld <jneufeld@alumni.ubc.ca>")
        .about("Parses poker hands from replayer format to JSON")
        .arg(
            Arg::with_name("input")
                .required(true)
                .short("i")
                .long("input")
                .takes_value(true)
                .help("Input file"),
        )
        .arg(
            Arg::with_name("output")
                .required(false)
                .default_value("output.json")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("Output file"),
        )
        .arg(
            Arg::with_name("stats")
                .required(false)
                .short("s")
                .long("stats")
                .takes_value(false)
                .help("Prints hand statistics"),
        )
        .arg(
            Arg::with_name("debug")
                .required(false)
                .short("d")
                .long("debug")
                .takes_value(false)
                .help("Prints debug info"),
        )
        .get_matches();

    let input = matches.value_of("input").unwrap();
    let output = matches.value_of("output").unwrap();
    let stats = matches.is_present("stats");
    let debug = matches.is_present("debug");

    let hands = parse::parse(input);
    let json = serde_json::to_value(&hands).unwrap();
    let json = serde_json::to_string(&json).unwrap();

    std::fs::write(output, json).unwrap();

    if debug {
        println!("{:#?}", &hands);
    }

    if stats {
        println!("Hands: {}", hands.len());

        let mut flops = 0;
        let mut turns = 0;
        let mut rivers = 0;
        let mut shows = 0;

        let mut unknown = 0;
        let mut nlh = 0;
        let mut nlh_he = 0;

        for hand in hands {
            match hand.game {
                Game::Unknown(_) => unknown += 1,
                Game::NoLimitHoldemHeadsUp => nlh_he += 1,
                Game::NoLimitHoldem => {
                    nlh += 1;

                    for action in hand.actions {
                        match action {
                            Action::Flop(_, _, _) => flops += 1,
                            Action::Turn(_) => turns += 1,
                            Action::River(_) => rivers += 1,
                            Action::Show(_, _, _) => {
                                shows += 1;
                                break;
                            }
                            _ => (),
                        }
                    }
                }
            };
        }

        println!("NLH: {}\nNLH HE: {}\nUnknown: {}", nlh, nlh_he, unknown);
        println!(
            "Flops: {}\nTurns: {}\nRivers: {}\nShowdowns: {}",
            flops, turns, rivers, shows
        );
    }
}
