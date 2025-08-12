//! Creates a very large list of tinantas. This list includes all combinations of:
//!
//! - Around 2000 dhatus from our dhatupatha
//! - 2 prayogas
//! - 5 sanAdi combinations (none, nic, san, yan, yan-luk)
//! - 10 lakaras
//! - 3 purushas
//! - 3 vacanas
//!
//! These combinations produce around 2000 x 2 x 5 x 10 x 3 x 3 = 1.8 million tinantas.
use clap::Parser;
use serde::Serialize;
use std::error::Error;
use std::io;
use vidyut_lipi::{Lipika, Scheme};
use vidyut_prakriya::args::{DhatuPada, Lakara, Prayoga, Purusha, Sanadi, Tinanta, Vacana};
use vidyut_prakriya::{Dhatupatha, Vyakarana};

/// Command line arguments.
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// If set, the output scheme to use. Supported options are: slp1, devanagari, iast.
    ///
    /// (Default: slp1)
    #[arg(long)]
    output_scheme: Option<String>,
}

#[derive(Debug, Serialize)]
struct Row<'a> {
    form: String,
    stem: String,
    suffix: String,
    dhatu: &'a str,
    gana_num: &'static str,
    entry_num: u16,
    // code: String,
    sanadis: String,
    prayoga: String,
    padi: String,
    lakara: String,
    purusha: String,
    vacana: String,
}

// fn create_pada_string(mut padas: Vec<String>, output_scheme: Scheme) -> String {
//     padas.sort();
//     let mut lipika = Lipika::new();
//     if output_scheme != Scheme::Slp1 {
//         for s in padas.iter_mut() {
//             *s = lipika.transliterate(&s, Scheme::Slp1, output_scheme);
//         }
//     }
//     padas.join("|")
// }

fn transliterate_to_wx(input_str: String) -> String {
    let mut lipika = Lipika::new();

    // Transliterate to wx scheme
    let transliterated_str = lipika.transliterate(&input_str, Scheme::Slp1, Scheme::Wx);

    // Remove accent svara marks "\" and "^"
    let cleaned_str = transliterated_str.replace("\\", "").replace("^", "");
    cleaned_str

    // transliterated_str
}

fn run(dhatupatha: Dhatupatha, _args: Args) -> Result<(), Box<dyn Error>> {
    let sanadi_choices = vec![
        vec![],
        vec![Sanadi::san],
        vec![Sanadi::Ric],
        vec![Sanadi::yaN],
        vec![Sanadi::yaNluk],
    ];

    let mut wtr = csv::Writer::from_writer(io::stdout());
    let v = Vyakarana::builder().log_steps(false).build();

    // let output_scheme = match args.output_scheme {
    //     Some(x) => match x.as_str() {
    //         "devanagari" => Scheme::Devanagari,
    //         "iast" => Scheme::Iast,
    //         "slp1" => Scheme::Slp1,
    //         // We should handle this with an error, but it's easier to default to SLP1.
    //         _ => Scheme::Slp1,
    //     },
    //     None => Scheme::Wx,
    // };

    // for entry in dhatupatha.into_iter().take(20) {
    for entry in dhatupatha {
        let dhatu = entry.dhatu();

        // if entry.code() != "01.1159" {
        //     continue;
        // }

        for sanadis in &sanadi_choices {
            for prayoga in &[Prayoga::Kartari, Prayoga::Karmani] {
                for dhatupada in DhatuPada::iter() {
                    for lakara in Lakara::iter() {
                        for purusha in Purusha::iter() {
                            for vacana in Vacana::iter() {
                                let tinanta = Tinanta::builder()
                                    .dhatu(dhatu.clone().with_sanadi(&sanadis))
                                    .prayoga(*prayoga)
                                    .pada(*dhatupada)
                                    .lakara(*lakara)
                                    .purusha(*purusha)
                                    .vacana(*vacana)
                                    .build()?;

                                let prakriyas = v.derive_tinantas(&tinanta);
                                if prakriyas.is_empty() {
                                    continue;
                                }

                                let forms: Vec<_> = prakriyas.iter().map(|p| p.text2()).collect();

                                for form in forms {
                                    // Form
                                    let form_wx = transliterate_to_wx(form);

                                    let (stem, suffix) =
                                        form_wx.split_once("+").unwrap_or(("", ""));

                                    // Dhatu
                                    let dhatu_text = &dhatu.upadesha().expect("ok");
                                    let dhatu_wx = transliterate_to_wx(dhatu_text.to_string());

                                    // Gana
                                    let gana = dhatu.gana().expect("ok");

                                    // Sanadi
                                    let sanadis_text = sanadis
                                        .iter()
                                        .map(|s| s.as_str().to_owned()) // Convert to `String` to own the data
                                        .collect::<Vec<_>>()
                                        .join(":");

                                    let sanadis_wx = transliterate_to_wx(sanadis_text.to_string());

                                    // Lakara
                                    let lakara_wx =
                                        transliterate_to_wx(lakara.as_str().to_string());

                                    // Padi
                                    let dhatupada_wx =
                                        transliterate_to_wx(dhatupada.as_str().to_string());

                                    // Prayoga
                                    let prayoga_wx =
                                        transliterate_to_wx(prayoga.as_str().to_string());

                                    // Purusha
                                    let purusha_wx =
                                        transliterate_to_wx(purusha.as_str().to_string());

                                    // Vacana
                                    let vacana_wx =
                                        transliterate_to_wx(vacana.as_str().to_string());

                                    // Create the row
                                    let row = Row {
                                        form: form_wx.to_owned(),
                                        stem: stem.to_owned(),
                                        suffix: suffix.to_owned(),
                                        dhatu: dhatu_wx.as_str(),
                                        gana_num: gana.as_str(),
                                        entry_num: entry.number(),
                                        sanadis: sanadis_text,
                                        lakara: lakara.as_str().to_string(),
                                        padi: dhatupada.as_str().to_string(),
                                        prayoga: prayoga.as_str().to_string(),
                                        purusha: purusha.as_str().to_string(),
                                        vacana: vacana.as_str().to_string(),
                                    };

                                    wtr.serialize(row)?;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    wtr.flush()?;
    Ok(())
}

fn main() {
    let args = Args::parse();

    let dhatus = match Dhatupatha::from_path("data/dhatupatha.tsv") {
        Ok(res) => res,
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        }
    };

    match run(dhatus, args) {
        Ok(()) => (),
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
