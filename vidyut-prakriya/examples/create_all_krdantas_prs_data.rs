//! Creates a very large list of krdantas. This list includes all combinations of:
//!
//! - Around 2000 dhatus from our dhatupatha
//! - 5 sanAdi combinations (none, nic, san, yan, yan-luk)
//! - Around 120 krt-pratyayas from the `Krt` enum, including variants like sya-Satf and sya-SAnac.
//!
//! These combinations produce around 2000 x 5 x 120 = 1.2 million krdantas.
//!
//! Usage:
//!
//!     cargo run --release --example create_all_krdantas -- --output-scheme Devanagari
use clap::Parser;
use serde::Serialize;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;

use vidyut_lipi::{Lipika, Scheme};
use vidyut_prakriya::args::{BaseKrt, Krdanta, Krt, Lakara, Prayoga, Vacana, Vibhakti, Subanta, Linga};
use vidyut_prakriya::Vyakarana;
use vidyut_kosha::{Kosha, entries::DhatuEntry, entries::KrdantaEntry, entries::PratipadikaEntry, entries::SubantaEntry};

mod src_utils;
use src_utils::find_src_root;

/// Command line arguments.
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// If set, the output scheme to use.
    ///
    /// Any scheme name accepted by `vidyut-prakriya` is valid. Examples: `Devanagari`, `Iso15919`,
    /// `Slp1`.
    ///
    /// (Default: `Slp1`)
    #[arg(long)]
    output_scheme: Option<String>,
}

#[derive(Debug, Serialize)]
// struct Row<'a> {
struct Row {
    krt_pratipadika: String,
    krt_first_form: String,
    gender: String,
    is_avyaya: bool,
    krt_suffix: String,

    dhatu: String,
    // dhatu_without_prefixes: String,
    // muladhatu: &'a str,
    muladhatu: String,
    // gana: &'static str,
    gana: String,
    prefixes: String,
    sanadi: String,
    prayoga: Option<Prayoga>,
    lakara: Option<Lakara>,
}

fn create_output_string(
    lipika: &mut Lipika,
    mut items: Vec<String>,
    output_scheme: Scheme,
) -> String {
    items.sort();
    items.dedup();
    if output_scheme != Scheme::Slp1 {
        for s in items.iter_mut() {
            *s = lipika.transliterate(&s, Scheme::Slp1, output_scheme);
        }
    }
    items.join("|")
}

// fn wx(lipika: &mut Lipika) -> (input_str: String) -> String {   
//     lipika.transliterate(&input_str, Scheme::Slp1, Scheme::Wx)
// }


// fn run<'a>(dhatus: Vec<DhatuEntry<'a>>, args: Args) -> Result<(), Box<dyn Error>> {
//     let sanadi_choices = vec![
//         vec![],
//         vec![Sanadi::san],
//         vec![Sanadi::Ric],
//         vec![Sanadi::yaN],
//         vec![Sanadi::yaNluk],
//     ];

//     let v = Vyakarana::builder().log_steps(false).build();
//     let mut lipika = Lipika::new();
//     let mut wtr = csv::Writer::from_writer(io::stdout());

//     let output_scheme: Scheme = match args.output_scheme {
//         Some(s) => s.parse()?,
//         None => Scheme::Slp1,
//     };

//     let sat_prayoga_lakara = &[
//         (Some(Prayoga::Kartari), Some(Lakara::Lat)),
//         (Some(Prayoga::Kartari), Some(Lakara::Lrt)),
//         (Some(Prayoga::Karmani), Some(Lakara::Lat)),
//         (Some(Prayoga::Karmani), Some(Lakara::Lrt)),
//     ];

//         for entry in &dhatus {
//             // let dhatu = entry.dhatu().clone().with_sanadi(&sanadis);
//             let dhatu = entry.dhatu().clone();
            

//             // println!(
//             //     "Processing dhatu: {}, prefixes: {}, sanadi: {}",
//             //     dhatu.aupadeshika().expect("ok"),
//             //     dhatu.prefixes().join("-"),
//             //     dhatu.sanadi().iter().map(|x| x.as_str()).collect::<Vec<_>>().join("-")
//             // );

//             for krt in BaseKrt::iter() {
//                 let prayoga_lakara: &[(Option<Prayoga>, Option<Lakara>)] = match krt {
//                     BaseKrt::Satf | BaseKrt::SAnac => sat_prayoga_lakara,
//                     _ => &[(None, None)],
//                 };

//                 for (prayoga, lakara) in prayoga_lakara.iter().copied() {
//                     let mut builder = Krdanta::builder().dhatu(dhatu.clone()).krt(krt);
//                     if let (Some(prayoga), Some(lakara)) = (prayoga, lakara) {
//                         builder = builder.prayoga(prayoga).lakara(lakara);
//                     }

//                     let krdanta = builder.build()?;

//                     let krt_pradipadika_prakriyas = v.derive_krdantas(&krdanta);

//                     if krt_pradipadika_prakriyas.is_empty() {
//                         continue;
//                     }

//                     let krt_pratipadika_entry = PratipadikaEntry::Krdanta(KrdantaEntry::new(entry.clone(), Krt::from(krt), prayoga, lakara));

//                     let lingas =  krt_pratipadika_entry.lingas();

//                     for linga in lingas {

//                         let subanta_entry = SubantaEntry::new(krt_pratipadika_entry.clone(), linga.clone(), Vibhakti::Prathama, Vacana::Eka);

//                         let subanta = Subanta::from(&subanta_entry);

//                         let subanta_prakriyas = v.derive_subantas(&subanta); 



//                         // let krt_pratipadika = Pratipadika::from(&krdanta).with_linga(linga);

//                         // let pratipadika_prakriyas = v.derive_pratipadikas(&krt_pratipadika);

//                         // if pratipadika_prakriyas.is_empty() {
//                         //     println!("No pratipadika prakriyas for dhatu: {}, {}, krt: {}, linga: {:?}", dhatu.aupadeshika().expect("ok"), dhatu.prefixes().join("-"), krt, linga);
//                         //     continue;
//                         // }

//                         // let muladhatu_text = &dhatu.aupadeshika().expect("ok");
//                         // let krdantas: Vec<_> = prakriyas.iter().map(|p| p.text2()).collect();
//                         // let krdantas = create_output_string(&mut lipika, krdantas, output_scheme);
//                         // let sanadi_text: Vec<_> = sanadis.iter().map(|x| x.as_str()).collect();
//                         // let sanadi_text = sanadi_text.join("-");
//                         // let prefixes = dhatu.prefixes().join("+");
//                         // let dhatu_text = entry.clean_text();
//                         // let gana_text = dhatu.gana().expect("ok").as_str();

//                         // let row = Row {
//                         //     krt_pratipadika: krdantas,
//                         //     krt_first_form: "".to_string(),
//                         //     gender: "".to_string(),
//                         //     krt_suffix: krt,
//                         //     prefixes: prefixes,
//                         //     sanadi: sanadi_text,
//                         //     dhatu: dhatu_text,
//                         //     muladhatu: muladhatu_text,
//                         //     gana: gana_text,
//                         //     prayoga: prayoga,
//                         //     lakara: lakara,
//                         // };

//                         // wtr.serialize(row)?;
//                     // }

//                     // let krt_pradipadika = Pratipadika::from(&krdanta);

//                     // let pratipadika_prakriyas = v.derive_pratipadikas(&krt_pradipadika);

//                     // // let prakriyas = v.derive_krdantas(&krdanta);
//                     // if pratipadika_prakriyas.is_empty() {
//                     //     println!("No pratipadika prakriyas for dhatu: {}, {}, krt: {}", dhatu.aupadeshika().expect("ok"), dhatu.prefixes().join("-"), krt);
//                     //     continue;
//                     // }

                    

//                     let muladhatu_text = &dhatu.aupadeshika().expect("ok");
//                     let krt_pratipadikas: Vec<_> = krt_pradipadika_prakriyas.iter().map(|p| p.text2()).collect();
//                     let krt_pratipadikas_text = create_output_string(&mut lipika, krt_pratipadikas, output_scheme);
//                     let krt_subantas: Vec<_> = subanta_prakriyas.iter().map(|p| p.text2()).collect();
//                     let krt_subantas_text = create_output_string(&mut lipika, krt_subantas, output_scheme);
//                     let sanadi_text = dhatu.sanadi().iter().map(|x| x.as_str()).collect::<Vec<_>>().join("-");
//                     let prefixes = dhatu.prefixes().join("+");
//                     let dhatu_text = entry.clean_text();
//                     let gana_text = dhatu.gana().expect("ok").as_str();

//                     // if linga is Linga.Stri then krt_pratipadikas should be krt_subantas
//                     // and the `H` should be removed from the krt_subantas
//                     let krt_pratipadikas_text = if *linga == Linga::Stri {
//                         krt_subantas_text.replace("H", "")
//                     } else {
//                         krt_pratipadikas_text
//                     };



//                     let row = Row {
//                         krt_pratipadika: krt_pratipadikas_text,
//                         krt_first_form: krt_subantas_text,
//                         gender: linga.to_string(),
//                         krt_suffix: krt,
//                         prefixes: prefixes,
//                         sanadi: sanadi_text,
//                         prayoga: prayoga,
//                         lakara: lakara,
//                         dhatu: dhatu_text,
//                         muladhatu: muladhatu_text,
//                         gana: gana_text,
//                     };

//                     wtr.serialize(row)?;
//                 }
//                 }
//             }
//         }
//         wtr.flush()?;
//         Ok(())
// }

fn lingas_for_krt(krt_suffix: &BaseKrt, pratipadika_entry: &PratipadikaEntry) -> Vec<Linga> {
    match krt_suffix {
        BaseKrt::lyuw => vec![Linga::Napumsaka],
        BaseKrt::GaY => vec![Linga::Pum],
        // The `|` operator matches multiple variants, equivalent to Python's `in` list check
        BaseKrt::ktin | BaseKrt::a => vec![Linga::Stri], 
        _ => {

            if pratipadika_entry.is_avyaya() {
                return vec![Linga::Pum];
            }

            if !pratipadika_entry.lingas().is_empty() {
                pratipadika_entry.lingas().to_vec()
            } else {
                vec![Linga::Pum, Linga::Stri, Linga::Napumsaka]
            }
        }
    }
}

fn run<'a>(dhatus: Vec<DhatuEntry<'a>>, args: Args) -> Result<(), Box<dyn Error>> {
    // ... sanadi_choices, output_scheme setup ...
    
    let v = Vyakarana::builder().log_steps(false).build();
    let mut lipika = Lipika::new();

    let mut wx = |s: &str| lipika.transliterate(s, Scheme::Slp1, Scheme::Wx);


    // // Lock stdout for much faster I/O in tight loops
    // let stdout = io::stdout();
    // let stdout_locked = stdout.lock();
    // let mut wtr = csv::Writer::from_writer(stdout_locked);

    // Write to file
    let file = File::create("krdantas2.csv")?;
    // Wrap the file in a BufWriter for fast, buffered I/O
    let buf_writer = BufWriter::new(file);
    // Pass the buffered file writer to the CSV writer
    let mut wtr = csv::Writer::from_writer(buf_writer);

    // let output_scheme: Scheme = match args.output_scheme {
    //     Some(s) => s.parse()?,
    //     None => Scheme::Slp1,
    // };

    let sat_prayoga_lakara = &[
        (Some(Prayoga::Kartari), Some(Lakara::Lat)),
        (Some(Prayoga::Kartari), Some(Lakara::Lrt)),
        (Some(Prayoga::Karmani), Some(Lakara::Lat)),
        (Some(Prayoga::Karmani), Some(Lakara::Lrt)),
    ];

    for entry in &dhatus {
        let dhatu = entry.dhatu().clone();
        
        
        let muladhatu_text = dhatu.aupadeshika().expect("ok");
        // let muladhatu_wo_prefixes = entry.dhatu().clone().with_prefixes(&[] as &[&str]);
        // let muladhatu_wo_prefixes_text = muladhatu_wo_prefixes.aupadeshika().expect("ok");        
        let sanadi_text = dhatu.sanadi().iter().map(|x| x.as_str()).collect::<Vec<_>>().join("-");
        let prefixes = dhatu.prefixes().join("+");
        let dhatu_text = entry.clean_text();
        let gana_text = dhatu.gana().expect("ok").as_str();

        for krt in BaseKrt::iter() {
            let prayoga_lakara: &[(Option<Prayoga>, Option<Lakara>)] = match krt {
                BaseKrt::Satf | BaseKrt::SAnac => sat_prayoga_lakara,
                _ => &[(None, None)],
            };

            for (prayoga, lakara) in prayoga_lakara.iter().copied() {
                let mut builder = Krdanta::builder().dhatu(dhatu.clone()).krt(krt);
                if let (Some(prayoga), Some(lakara)) = (prayoga, lakara) {
                    builder = builder.prayoga(prayoga).lakara(lakara);
                }

                let krdanta = builder.build()?;
                let krt_pradipadika_prakriyas = v.derive_krdantas(&krdanta);

                if krt_pradipadika_prakriyas.is_empty() {
                    eprintln!("No krdanta prakriyas for dhatu: {}, krt: {}, prefixes: {}, sanadi: {}", muladhatu_text, krt, prefixes, sanadi_text);
                    continue;
                }

                // 2. Pre-calculate base pratipadika text OUTSIDE the Linga loop
                let krt_pratipadikas: Vec<_> = krt_pradipadika_prakriyas.iter().map(|p| p.text2()).collect();
                

                let krt_pratipadika_entry = PratipadikaEntry::Krdanta(KrdantaEntry::new(entry.clone(), Krt::from(krt), prayoga, lakara));
                let lingas = lingas_for_krt(&krt, &krt_pratipadika_entry);

                for linga in lingas {
                    let subanta_entry = SubantaEntry::new(krt_pratipadika_entry.clone(), linga.clone(), Vibhakti::Prathama, Vacana::Eka);
                    let subanta = Subanta::from(&subanta_entry);
                    let subanta_prakriyas = v.derive_subantas(&subanta); 

                    let krt_subantas: Vec<_> = subanta_prakriyas.iter().map(|p| p.text2()).collect();
                    

                    // 3. Optimize visarga removal on each string element for Stri
                    let krt_pratipadikas = if linga == Linga::Stri {
                        krt_subantas
                            .iter()
                            .map(|s| s.strip_suffix("H").unwrap_or(s).to_string())
                            .collect::<Vec<String>>()
                    } else {
                        krt_pratipadikas.clone()
                    };

                    let krt_pratipadikas_text = krt_pratipadikas.join("|");
                    let krt_subantas_text = krt_subantas.join("|");

                    let row = Row {
                        // krt_pratipadika: wx(&krt_pratipadikas_text),
                        krt_pratipadika: krt_pratipadikas_text,
                        // krt_first_form: wx(&krt_subantas_text),
                        krt_first_form: krt_subantas_text,
                        gender: wx(&linga.as_str()),
                        is_avyaya: krt_pratipadika_entry.is_avyaya(),
                        krt_suffix: wx(&krt.as_str()),
                        dhatu: wx(&dhatu_text),
                        // dhatu_without_prefixes: wx(&muladhatu_wo_prefixes_text),
                        muladhatu: wx(&muladhatu_text),
                        gana: wx(&gana_text),
                        prefixes: wx(&prefixes),       // Clone pre-calculated strings
                        sanadi: wx(&sanadi_text),      // instead of re-creating them
                        prayoga: prayoga,
                        lakara: lakara,
                    };

                    wtr.serialize(row)?;
                }
            }
        }
    }
    wtr.flush()?;
    Ok(())
}


fn main() {
    let args = Args::parse();

    let source_root = find_src_root(); // Find the toplevel .git directory
    assert!(
        source_root.is_some(),
        "Could not find toplevel .git directory"
    );
    // let dhatupatha_path = source_root
    //     .unwrap()
    //     .as_path()
    //     .join("vidyut-prakriya/data/dhatupatha.tsv");

    let kosha = Kosha::new(source_root
        .unwrap()
        .as_path()
        .join("data/build/vidyut-0.2.0/kosha2")).expect("Failed to load kosha");

    // print first dhatu from kosha
    // let first_dhatu = kosha.dhatus().next().unwrap();
    // println!("First dhatu from kosha: {}", first_dhatu.clean_text());

    let dhatus: Vec<_> = kosha.dhatus().collect();

    // let dhatus = match Dhatupatha::from_path(dhatupatha_path.as_path()) {
    //     Ok(res) => res,
    //     Err(err) => {
    //         println!("{}", err);
    //         std::process::exit(1);
    //     }
    // };

    match run(dhatus, args) {
        Ok(()) => (),
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
