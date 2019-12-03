use clap::{App, AppSettings, Arg};
use conllx::io::{ReadSentence, WriteSentence};
use conllx::token::Features;
use std::io::BufWriter;
use stdinout::{Input, OrExit, Output};

static INPUT: &str = "INPUT";
static DEFAULT_CLAP_SETTINGS: &[AppSettings] = &[
    AppSettings::DontCollapseArgsInUsage,
    AppSettings::UnifiedHelpMessage,
];
static OUTPUT: &str = "OUTPUT";

fn main() {
    let matches = App::new("depth")
        .settings(DEFAULT_CLAP_SETTINGS)
        .arg(Arg::with_name(INPUT).help("Input").index(1))
        .arg(Arg::with_name(OUTPUT).help("Output").index(2))
        .get_matches();
    let input = matches.value_of(INPUT).map(ToOwned::to_owned);
    let input = Input::from(input);
    let input = conllx::io::Reader::new(input.buf_read().or_exit("Failed opening input", 1));
    let output = matches.value_of(OUTPUT).map(ToOwned::to_owned);
    let output = Output::from(output);
    let output = output.write().or_exit("Failed opening output", 1);

    let mut output = conllx::io::Writer::new(BufWriter::new(output));

    for sent in input.sentences() {
        let mut sent = sent.unwrap();
        let mut g = sent.dep_graph_mut();

        for idx in 1..g.len() {
            let mut feat = "".to_string();

            if g[idx].token().unwrap().pos().unwrap().starts_with("VERB") {
                let mut first_nmod = true;
                let mut first_advmod = true;
                for triple in g.dependents(idx) {
                    let rel = triple.relation().expect("no rel");
                    if rel.starts_with("nmod") && first_nmod {
                        feat += &format!("|{}:1", "nmod");
                        first_nmod = false;
                    }

                    if rel.starts_with("advmod") && first_advmod {
                        feat += &format!("|{}:1", "advmod");
                        first_advmod = false;
                    }
                }
            }

            let feats = g[idx]
                .token()
                .unwrap()
                .features()
                .unwrap()
                .as_str()
                .to_string()
                + &feat;
            g[idx]
                .token_mut()
                .or_exit("Token missing", 1)
                .set_features(Some(Features::from_string(feats)));
        }
        output
            .write_sentence(&sent)
            .or_exit("Failed writing sent.", 1);
    }
}
