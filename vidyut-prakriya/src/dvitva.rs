/// Runs rules that perform `dvitva` (doubling) on the dhAtu.
///
/// TODO: the code here is repetitive and can be consolidated with a bit more thought.
use crate::ac_sandhi;
use crate::core::operators as op;
use crate::core::Rule::{Kashika, Varttika};
use crate::core::Tag as T;
use crate::core::Term;
use crate::core::{Code, Prakriya};
use crate::sounds as al;
use crate::sounds::{s, Set};
use compact_str::CompactString;
use lazy_static::lazy_static;

lazy_static! {
    static ref AC: Set = s("ac");
    static ref YAN: Set = s("yaR");
    static ref HAL: Set = s("hal");
    static ref NDR: Set = s("n d r");
}

/// Finds the character span that should be duplicated in the given text.
fn find_abhyasa_span(text: &CompactString) -> Option<(usize, usize)> {
    let mut start = None;
    let mut end = None;
    for (i, c) in text.chars().enumerate() {
        // Start at first consonant.
        if start.is_none() && HAL.contains(c) {
            // 6.1.3 na ndrAH saMyogAdayaH
            if i < text.len() - 1 {
                let next = text.as_bytes()[i + 1] as char;
                // check for 'b' by varttika on 6.1.3 (ubjijizati)
                if (NDR.contains(c) || c == 'b') && HAL.contains(next) {
                    if c == 'r' && next == 'y' {
                        // yakāraparasya rephasya pratiṣedho na bhavatīti vaktavyam
                    } else {
                        continue;
                    }
                }
            }
            start = Some(i);
        }
        if start.is_some() && AC.contains(c) {
            end = Some(i);
            break;
        }
    }
    if let (Some(start), Some(end)) = (start, end) {
        Some((start, end))
    } else {
        None
    }
}

fn mark_abhyasta(p: &mut Prakriya, i_start: usize, i_end: usize) {
    for i in i_start..=i_end {
        p.set(i, |t| t.add_tag(T::Abhyasta));
    }
}

/// Tries dvitva on the base ending at index `i`.
fn try_dvitva(rule: Code, p: &mut Prakriya, i_dhatu: usize) -> Option<()> {
    // First, run ac-sandhi (for div -> dudyUzati, etc.)
    ac_sandhi::run_antaranga(p);

    p.maybe_save_sthanivat();
    // Force-save for dhatus that consist of a single vowel.
    if p.has(i_dhatu, |t| t.has_adi(&*AC) && t.has_antya(&*YAN)) {
        p.set(i_dhatu, |t| t.force_save_sthanivat());
    }

    let i_n = p.find_next_where(i_dhatu, |t| {
        !(t.is_agama() && t.has_tag(T::kit) && !t.is_it_agama())
    })?;
    let dhatu = p.get(i_dhatu)?;
    let next = p.pratyaya(i_n)?;

    if dhatu.has_adi(&*AC)
        && next.last().is_pratyaya()
        && next.last().has_u_in(&["san", "Ric", "yaN", "RiN"])
    {
        // Case 1: sanAdi ajAdi dhAtu
        //
        // In this case, the dhatu will likely include at least some of the sanAdi-pratyaya.

        // Case 1a: special case for Irzya~
        if dhatu.has_u("Irzya~") && next.first().has_u("iw") {
            let i_it = next.start();
            let i_pratyaya = next.end();
            let done = p.optional_run(Kashika("6.1.3"), |p| {
                // Irz + [yi] + y + i + sa
                let mut abhyasa = Term::make_text("yi");
                abhyasa.add_tags(&[T::Abhyasa, T::FlagIttva]);
                p.set(i_dhatu, |t| t.set_antya(""));
                p.insert_after(i_dhatu, abhyasa);
                p.insert_after(i_dhatu + 1, Term::make_text("y"));
                mark_abhyasta(p, i_dhatu + 1, i_dhatu + 3);
                p.set(i_dhatu, |t| t.add_tag(T::Dvitva));
            });
            if !done {
                p.run(Varttika("6.1.3.3"), |p| {
                    // Irzy + i + [sa] + sa
                    let mut abhyasa = Term::make_text(&p.get(i_pratyaya).expect("").text);
                    abhyasa.add_tags(&[T::Abhyasa, T::FlagIttva]);
                    if abhyasa.has_adi('s') {
                        abhyasa.add_tag(T::FlagSaAdeshadi);
                    }
                    p.insert_after(i_it, abhyasa);
                    mark_abhyasta(p, i_dhatu + 2, i_dhatu + 3);
                    p.set(i_dhatu, |t| t.add_tag(T::Dvitva));
                });
            }
            return None;
        }

        // Case 1b: other dhatus
        let mut p_text = CompactString::from("");
        for t in p.terms() {
            if t.is_upasarga() || t.is_lupta() {
                continue;
            }
            if !t.sthanivat().is_empty() {
                p_text.push_str(t.sthanivat());
            } else {
                p_text.push_str(&t.text);
            }
        }
        let dhatu = p.get(i_dhatu)?;

        let (start, end) = find_abhyasa_span(&p_text)?;

        // Term up to and including last vowel before abhyasa.
        let mut abhyasa = Term::make_text(&p_text[start..=end]);
        abhyasa.add_tags(&[T::Abhyasa, T::FlagIttva]);

        // For OcicCat, etc.
        //
        // > hrasva eva atra āgamī, na tu tadantaḥ. tena cicchadatuḥ, cicchiduḥ ityatra
        // > tukabhyāsasya grahaṇena na gṛhyate iti halādiḥśeṣeṇa na nivartyate, nāvayavāvayavaḥ
        // > samudāyāvayavo bhavatīti.
        //
        // -- KV on 6.1.73.
        if abhyasa.starts_with("tC") {
            abhyasa.set_adi("");
        }
        // For zatva in 8.3.
        if abhyasa.has_adi('s') && !dhatu.text.contains('s') {
            abhyasa.add_tag(T::FlagSaAdeshadi);
        }

        let dhatu_len = dhatu.len();
        p.set(i_dhatu, |t| t.set_text(&p_text[0..start]));

        if dhatu_len > start {
            // Case 1b1: abhyasa within dhatu ([und] i sa --> un di [d] i sa)
            let i_dhatu_old = i_dhatu;
            let before_abhyasa = Term::make_text(&p_text[..start]);
            p.insert_before(i_dhatu, before_abhyasa);
            p.insert_before(i_dhatu + 1, abhyasa);
            p.set(i_dhatu + 2, |t| t.set_text(&p_text[start..dhatu_len]));

            let i_abhyasa = i_dhatu_old + 1;
            let i_dhatu = i_dhatu_old + 2;
            let dhatu = p.get_mut(i_dhatu)?;
            if dhatu.has_u("UrRuY") && dhatu.has_adi('R') {
                dhatu.set_adi("n");
            }
            p.step(rule);

            p.add_tag_at("6.1.4", i_abhyasa, T::Abhyasa);
            p.run("6.1.5", |p| {
                mark_abhyasta(p, i_dhatu_old, i_dhatu);
                p.set(i_dhatu, |t| t.add_tag(T::Dvitva));
            });
        } else {
            // Case 1b2: abhyasa after dhatu (i sa --> i sa sa)
            p.set(i_dhatu, |t| t.set_text(&p_text[0..start]));
            p.insert_after(i_dhatu, abhyasa);
            p.step(rule);

            p.add_tag_at("6.1.4", i_dhatu + 1, T::Abhyasa);
            p.run("6.1.5", |p| {
                mark_abhyasta(p, i_dhatu, i_dhatu + 2);
                p.set(i_dhatu, |t| t.add_tag(T::Dvitva));
            });
        }
    } else if dhatu.is_ekac() || al::is_hal(dhatu.adi()?) {
        // Case 2: halAdi dhatu
        let mut abhyasa = Term::make_text("");
        abhyasa.set_text(dhatu.sthanivat());

        // See comment elsewhere in this module on 6.1.73 and removal of tuk-Agama.
        if dhatu.starts_with("tC") {
            abhyasa.set_adi("");
        }
        p.insert_before(i_dhatu, abhyasa);
        p.step(rule);

        let i_abhyasa = i_dhatu;
        let i_dhatu = i_dhatu + 1;
        p.add_tag_at("6.1.4", i_abhyasa, T::Abhyasa);

        p.set(i_abhyasa, |t| t.add_tag(T::Abhyasta));
        p.set(i_dhatu, |t| t.add_tags(&[T::Abhyasta, T::Dvitva]));
        if p.has(i_dhatu + 1, |t| t.is_ni_pratyaya()) {
            p.set(i_dhatu + 1, |t| t.add_tag(T::Abhyasta));
        }
        p.step("6.1.5")
    } else {
        // Case 3: ajAdi dhAtu
        //
        // Create 3 terms:
        // 1. the dhatu without the abhyasa
        // 2. the abhyasa
        // 3. the doubled portion
        //
        // 6.1.2 ajAder dvitIyasya
        // 6.1.3 na ndrAH saMyogAdayaH
        let dhatu = p.get(i_dhatu)?;

        let temp = match &dhatu.u {
            Some(u) => u.clone(),
            None => return None,
        };
        let mut third = Term::make_upadesha(&temp);
        third.set_text(&dhatu.sthanivat()[1..]);

        // 6.1.3 na ndrAH saMyogAdayaH
        while third.is_samyogadi() && NDR.contains(third.adi()?) {
            third.set_adi("");
        }
        third.add_tags(&[T::Dhatu]);

        let abhyasa = Term::make_text(&third.text);
        p.set(i_dhatu, |t| t.text.truncate(t.len() - abhyasa.len()));
        if p.has(i_dhatu, |t| t.has_u("UrRuY")) {
            third.set_adi("n");
        }

        p.insert_after(i_dhatu, abhyasa);
        p.insert_after(i_dhatu + 1, third);
        p.step(rule);
        p.add_tag_at("6.1.4", i_dhatu + 1, T::Abhyasa);

        p.set(i_dhatu, |t| t.add_tag(T::Abhyasta));
        p.set(i_dhatu + 1, |t| t.add_tag(T::Abhyasta));
        p.set(i_dhatu + 2, |t| t.add_tags(&[T::Abhyasta, T::Dvitva]));
        if p.has(i_dhatu + 3, |t| t.is_ni_pratyaya()) {
            p.set(i_dhatu + 3, |t| t.add_tag(T::Abhyasta));
        }
        p.step("6.1.5");
    }

    Some(())
}

/// Runs dvitva at the given index.
///
/// - `i` should point to a dhatu.
fn run_at_index(p: &mut Prakriya, i: usize) -> Option<()> {
    let dhatu = p.get_mut(i)?;
    debug_assert!(dhatu.is_dhatu());

    let jaksh_adi = &[
        "jakza~", "jAgf", "daridrA", "cakAsf~", "SAsu~", "dIDIN", "vevIN",
    ];
    if p.has(i, |t| t.has_u_in(jaksh_adi)) {
        // These are termed abhyasta, but they can still undergo dvitva because
        // the rules below are conditioned specifically on "anabhyAsasya" ("not having an abhyasa")
        // from 6.1.8.
        p.add_tag_at("6.1.6", i, T::Abhyasta);
    }

    // Use a view to include `iw`-Agama. Skip vu~k and other dhatu-agamas.
    let i_n = p.find_next_where(i, |t| {
        !(t.is_agama() && t.has_tag(T::kit) && !t.is_it_agama())
    })?;
    let n = p.pratyaya(i_n)?;
    if n.has_lakshana("li~w") {
        let dhatu = p.get(i)?;
        // kAshikA:
        //   dayateḥ iti dīṅo grahaṇaṃ na tu daya dāne ityasya.
        //   digyādeśena dvirvacanasya bādhanam iṣyate.
        if dhatu.has_u("de\\N") {
            p.run_at("7.4.9", i, op::text("digi"));
        } else {
            try_dvitva("6.1.8", p, i);
        }
    } else if p
        .find_next_where(i, |t| t.has_u_in(&["san", "yaN"]) && !t.is_unadi())
        .is_some()
    {
        try_dvitva("6.1.9", p, i);
    } else if n.has_tag(T::Slu) {
        try_dvitva("6.1.10", p, i);
    } else if p.find_next_where(i, |t| t.has_u("caN")).is_some() {
        // `last()` to avoid `it`.
        try_dvitva("6.1.11", p, i);
    }

    Some(())
}

/// Runs dvitva rule only if the pratyaya that causes dvitva starts with a vowel.
///
/// For more details, see rule 1.1.59 ("dvirvacane 'ci").
pub fn try_dvirvacane_aci(p: &mut Prakriya) -> Option<()> {
    p.debug("try_dvirvacane_aci");
    // Select !pratyaya to avoid sanAdi, which are also labeled as Dhatu.
    let filter = |t: &Term| t.is_dhatu() && !t.has_tag_in(&[T::Dvitva, T::Pratyaya]);

    // Loop for cases like jihriyAmbaBUva, where dvitva occurs twice.
    let mut num_loops = 0;
    let mut i = p.find_first_where(filter)?;
    loop {
        let i_n = p.find_next_where(i, |t| !t.is_empty())?;

        // Run only if the next term starts with a vowel.
        // Check for `Ji` as well, which starts with a vowel.
        // Exclude it_agama so that we can derive `aririzati` etc.
        let n = p.get(i_n)?;
        if (n.has_adi(&*AC) && !n.is_it_agama()) || n.has_text("Ji") {
            run_at_index(p, i);
        }

        num_loops += 1;
        if num_loops > 10 {
            panic!("Infinite loop {:?}", p.terms());
        }

        i = p.find_next_where(i, filter)?;
    }
}

pub fn run(p: &mut Prakriya) -> Option<()> {
    // Select !pratyaya to avoid sanAdi, which are also labeled as Dhatu.
    let filter = |t: &Term| t.is_dhatu() && !t.has_tag_in(&[T::Dvitva, T::Pratyaya]);

    // Loop for cases like jihriyAmbaBUva, where dvitva occurs twice.
    let mut num_loops = 0;
    let mut i = p.find_first_where(filter)?;
    loop {
        run_at_index(p, i);

        num_loops += 1;
        if num_loops > 10 {
            panic!("Infinite loop {:?}", p.terms());
        }

        i = p.find_next_where(i, filter)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_abhyasa_span_examples() {
        let find = |t| find_abhyasa_span(&CompactString::from(t));
        assert_eq!(find("kIrza"), Some((0, 1)));
        assert_eq!(find("undiza"), Some((2, 3)));
        assert_eq!(find("arya"), Some((1, 3)));
    }
}
