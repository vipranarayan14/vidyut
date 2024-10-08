/*!
The rules of the Ashtadhyayi.


# About the sutrapatha

The sutrapatha is the core of Paniniya-vyakarana and defines the grammar itself. It consists of
around 4000 formal rules that generate valid Sanskrit expressions if applied with the various
secondary texts of the Paninian school (i.e. the Unadipatha, the Ganapatha, the Dhatupatha, and the
Linganushasanam).

The rules of the sutrapatha enable and disable each other in various complex patterns. Most
commonly, two rules have an utsarga-apavAda relation, where an exception (apavAda) takes priority
over a general principle (utsarga). The sutrapatha uses various other conflict resolution
mechanisms as well. Most notably, the bulk of the tripAdi (8.2 - 8.4) uses an imperative control
flow, meaning that an earlier rule takes priority over a later one.


# About this code

To manage some of the complexity of the sutrapatha, we have tried to group rules according to their
logical function: defining lakaras, adding pratyayas, etc. This is not always possible due to the
inherent complexities of the text. For that reason, we strongly recommend navigating the rules of
the sutrapatha by grepping over our code.
*/
use crate::ac_sandhi;
use crate::angasya;
use crate::ardhadhatuka;
use crate::args::{
    Artha, Dhatu, Krdanta, Lakara, Pada, Pratipadika, Prayoga, Samasa, Subanta, Taddhitanta,
    Tinanta,
};
use crate::atidesha;
use crate::atmanepada;
use crate::core::errors::*;
use crate::core::prakriya_stack::PrakriyaStack;
use crate::core::Prakriya;
use crate::core::Tag;
use crate::core::Term;
use crate::dhatu_karya;
use crate::dvitva;
use crate::it_agama;
use crate::krt;
use crate::la_karya;
use crate::linganushasanam;
use crate::misc;
use crate::pratipadika_karya;
use crate::samasa;
use crate::samjna;
use crate::samprasarana;
use crate::sanadi;
use crate::stritva;
use crate::sup_karya;
use crate::svara;
use crate::taddhita;
use crate::tin_pratyaya;
use crate::tripadi;
use crate::uttarapade;
use crate::vikarana;

/// Adds a dhatu to the prakriya and runs basic follow-up tasks, such as:
///
/// - adding upasargas
/// - replacing initial `R` and `z`  with `n` and `s`, respectively.
/// - recording and removing any it-samjnas
/// - adding any necessary sanAdi-pratyayas.
fn prepare_dhatu(p: &mut Prakriya, dhatu: &Dhatu, is_ardhadhatuka: bool) -> Result<()> {
    match dhatu {
        Dhatu::Mula(m) => {
            dhatu_karya::run(p, m)?;
        }
        Dhatu::Nama(n) => {
            dhatu_karya::try_add_prefixes(p, n.prefixes());
            sanadi::try_create_namadhatu(p, n);
            if !p.terms().last().expect("ok").is_dhatu() {
                println!("{:#?}", p);
                return Err(Error::Abort(p.rule_choices().clone()));
            }
        }
    }

    sanadi::try_add_required(p, is_ardhadhatuka);
    if p.terms().last().expect("ok").is_pratyaya() {
        samjna::run(p);
        run_main_rules(p, None, false)?;
        // Defer tripadi until we add other pratyayas.
    }

    if let Dhatu::Mula(_) = dhatu {
        for s in dhatu.sanadi() {
            // HACK: reset padas for next sanadi.
            p.remove_tag(Tag::Parasmaipada);
            p.remove_tag(Tag::Atmanepada);

            sanadi::try_add_optional(p, *s)?;
            samjna::run(p);
            atmanepada::run(p);
            run_main_rules(p, None, false)?;
            // Defer tripadi until we add other pratyayas.
        }
    }

    p.debug("~~~~~~~~~~~~~~ completed dhatu ~~~~~~~~~~~~~~~~~~");

    Ok(())
}

/// Adds the basic terms necessary to create a krdanta.
fn prepare_krdanta(p: &mut Prakriya, args: &Krdanta) -> Result<()> {
    // If defined, set the meaning condition that this prakriya must follow.
    if let Some(artha) = args.artha() {
        p.set_artha(Artha::Krt(artha));
    }

    if let Some(upapada) = args.upapada() {
        prepare_pratipadika(p, upapada.pratipadika())?;

        let mut su = Term::make_text("");
        su.add_tags(&[Tag::Pratyaya, Tag::Vibhakti, Tag::Sup, Tag::Pada]);
        p.push(su);
        samjna::run(p);
    }

    let krt = args.krt();
    prepare_dhatu(p, args.dhatu(), krt.is_ardhadhatuka())?;
    if let Some(la) = args.lakara() {
        p.add_tag(Tag::Kartari);
        add_lakara_and_decide_pada(p, la);
    }
    let added = krt::run(p, args);
    if !added {
        println!("{:#?}", p);
        return Err(Error::Abort(p.rule_choices().clone()));
    }

    if args.upapada().is_some() {
        let i_last = p.terms().len() - 1;
        p.add_tag_at("2.2.19", i_last, Tag::Samasa)
    }

    linganushasanam::run(p);
    stritva::run(p);
    samjna::run(p);

    Ok(())
}

fn prepare_pratipadika(p: &mut Prakriya, pratipadika: &Pratipadika) -> Result<()> {
    use Pratipadika as Prati;
    match pratipadika {
        Pratipadika::Krdanta(k) if k.require().is_some() => {
            let mut stack = PrakriyaStack::new(false, false, false);
            stack.find_all(|p| derive_krdanta(p, k));

            let mut added = false;
            if let Some(s) = k.require() {
                for temp_p in stack.prakriyas() {
                    if *s == temp_p.text() {
                        p.extend(temp_p.terms());
                        added = true;
                    }
                    break;
                }
            }
            if !added {
                return Err(Error::Abort(p.rule_choices().clone()));
            }
        }
        Pratipadika::Taddhitanta(t) if t.require().is_some() => {
            let mut stack = PrakriyaStack::new(false, false, false);
            stack.find_all(|p| derive_taddhitanta(p, t));

            let mut added = false;
            if let Some(s) = t.require() {
                for temp_p in stack.prakriyas() {
                    if *s == temp_p.text() {
                        p.extend(temp_p.terms());
                        added = true;
                    }
                    break;
                }
            }
            if !added {
                return Err(Error::Abort(p.rule_choices().clone()));
            }
        }
        Prati::Basic(basic) => pratipadika_karya::add_basic(p, basic),
        Prati::Krdanta(krdanta) => prepare_krdanta(p, krdanta)?,
        Prati::Taddhitanta(taddhitanta) => prepare_taddhitanta(p, taddhitanta)?,
        Prati::Samasa(samasa) => prepare_samasa(p, samasa)?,
    }

    samjna::try_decide_pratipadika(p);

    Ok(())
}

/// Adds the basic terms necessary to create a krdanta.
fn prepare_taddhitanta(p: &mut Prakriya, args: &Taddhitanta) -> Result<()> {
    let taddhita = args.taddhita();

    prepare_pratipadika(p, args.pratipadika())?;
    samjna::run(p);

    // If defined, set the meaning condition that this prakriya must follow.
    //
    // Set `artha` *after* `prepare_pratipadika` to avoid clobbering `artha` when dealing with
    // nested taddhitantas.
    if let Some(artha) = args.artha() {
        p.set_artha(Artha::Taddhita(artha));
    }

    let added = taddhita::run(p, taddhita);
    if !added {
        if cfg!(debug_assertions) {
            println!("{:?}: {:#?}", args.taddhita(), p);
        }
        return Err(Error::Abort(p.rule_choices().clone()));
    }

    angasya::run_before_stritva(p);
    linganushasanam::run(p);
    stritva::run(p);
    samjna::run(p);

    Ok(())
}

fn prepare_samasa(p: &mut Prakriya, args: &Samasa) -> Result<()> {
    use crate::core::Tag as T;
    use crate::it_samjna;

    for pada in args.padas() {
        let pratipadika = pada.pratipadika();
        prepare_pratipadika(p, pratipadika)?;

        if pada.is_avyaya() {
            let i = p.terms().len() - 1;
            p.set(i, |t| t.add_tag(T::Avyaya));
            if p.has(i, |t| t.has_u("naY")) {
                it_samjna::run(p, i).expect("ok");
            }
        }
        p.push(make_sup_pratyaya(pada.vibhakti()));
    }

    // Remove the trailing sup-pratyaya.
    p.terms_mut().pop();

    samjna::run(p);

    let added = samasa::run(p, args);
    if !added {
        return Err(Error::Abort(p.rule_choices().clone()));
    }

    pratipadika_karya::run_napumsaka_rules(p);
    taddhita::run_for_samasas(p);

    if args.stri() {
        p.add_tag(T::Stri);
        stritva::run(p);
        p.remove_tag(T::Stri);
    }

    Ok(())
}

/// Adds a lakara to the prakriya and decides which pada it is allowed to use.
///
/// Scope: tinantas and certain krdantas (gacCat, jagmivas, gamyamAna, ...)
fn add_lakara_and_decide_pada(p: &mut Prakriya, lakara: Lakara) {
    la_karya::run(p, lakara);
    // Constraints:
    // - must come after `la_karya`, which affects the choice of cakz -> KyAY.
    ardhadhatuka::dhatu_adesha_before_pada(p, lakara);
    // Constraints:
    // - must come after `dhatu_adesha_before_pada` to allow parasmaipada for cakz -> KyAY.
    atmanepada::run(p);
    // Try adding am-pratyaya and the corresponding dhatu before tin-adesha, since doing so affects
    // which tin-pratyaya we use with bhU and kR.
    vikarana::try_add_am_pratyaya_for_lit(p);
}

/// Scope: all prakriyas
fn run_main_rules(p: &mut Prakriya, lakara: Option<Lakara>, is_ardhadhatuka: bool) -> Result<()> {
    p.debug("==== Tin-siddhi ====");
    // Do lit-siddhi and AzIrlin-siddhi first to support the valAdi vArttika for aj -> vi.
    let is_lit_or_ashirlin = matches!(lakara, Some(Lakara::Lit) | Some(Lakara::AshirLin));
    if let Some(lakara) = lakara {
        if is_lit_or_ashirlin {
            tin_pratyaya::try_general_siddhi(p, lakara);
            tin_pratyaya::try_siddhi_for_jhi(p, lakara);
        }
    }

    p.debug("==== Vikaranas ====");
    ardhadhatuka::run_before_vikarana(p, lakara, is_ardhadhatuka);
    vikarana::run(p)?;
    samjna::run(p);

    if let Some(lakara) = lakara {
        if !is_lit_or_ashirlin {
            tin_pratyaya::try_general_siddhi(p, lakara);
        }
    }

    // Constraints:
    // - should run before atidesha rules because of Rittva.
    // - should also run for subantas.
    angasya::try_add_or_remove_nit(p);

    p.debug("==== Dhatu tasks ====");
    {
        // Needed transitively for dhatu-samprasarana.
        angasya::try_pratyaya_adesha(p);
        // Must run before it-Agama.
        angasya::try_cinvat_for_bhave_and_karmani_prayoga(p);

        // Must run before it_agama rules since it affects how those rules are applied.
        atidesha::run_before_it_agama(p);
        // Depends on jha_adesha since it conditions on the first sound.
        it_agama::run_before_attva(p);
        // Should come before atidesha rules for ju[hve --> hU]zati (san is kit).
        samprasarana::run_for_dhatu_before_atidesha(p);
        // Depends on it_agama for certain rules.
        atidesha::run_before_attva(p);

        // Samprasarana of the dhatu is conditioned on several other operations, which we must execute
        // first:
        //
        // 1. jha_adesha (affects it-Agama).
        // 2. it_agama (affects kit-Nit)
        // 3. atidesha (for kit-Nit)
        samprasarana::run_for_dhatu_after_atidesha(p);
        // Ad-Adeza and other special tasks for Ardhadhatuka
        ardhadhatuka::run_before_dvitva(p);

        // Now finish it_agama and atidesha
        it_agama::run_after_attva(p);
        atidesha::run_after_attva(p);
    }

    // Must follow tin-siddhi and it-Agama, which could change the first sound of the pratyaya.
    ardhadhatuka::try_add_am_agama(p);

    p.debug("==== Dvitva (dvirvacane 'ci) ====");
    dvitva::try_dvirvacane_aci(p);
    let used_dvirvacane_aci = p.find_last_where(Term::is_abhyasta).is_some();
    if used_dvirvacane_aci {
        samprasarana::run_for_abhyasa(p);
    }

    // If Ji causes dvitva, that dvitva will be performed in `try_dvirvacane_aci` above.
    // So by this point, it's safe to replace Ji. (See 3.4.109, which replaces Ji if it follows a
    // term called `abhyasta`.)
    if let Some(lakara) = lakara {
        if !is_lit_or_ashirlin {
            tin_pratyaya::try_siddhi_for_jhi(p, lakara);
        }
    }

    // Samasa rules.
    // TODO: can these be put somewhere more sensible?
    uttarapade::run(p);
    samasa::try_sup_luk(p);
    misc::run_pad_adi(p);

    angasya::maybe_do_jha_adesha(p);

    ac_sandhi::try_sup_sandhi_before_angasya(p);
    angasya::run_before_dvitva(p);

    // After guna
    ardhadhatuka::try_aa_adesha_for_sedhayati(p);

    p.debug("==== Dvitva (default) ====");
    dvitva::run(p);
    if !used_dvirvacane_aci {
        samprasarana::run_for_abhyasa(p);
    }

    p.debug("==== After dvitva ====");
    angasya::run_after_dvitva(p);
    uttarapade::run_after_guna_and_bhasya(p);

    ac_sandhi::try_sup_sandhi_after_angasya(p);
    ac_sandhi::run_common(p);

    if p.use_svaras() {
        p.debug("==== Svaras ====");
        svara::run(p);
    }

    // Run tripadi rules separately.

    Ok(())
}

/// Derives a single dhatu from the given conditions.
pub fn derive_dhatu(mut prakriya: Prakriya, args: &Dhatu) -> Result<Prakriya> {
    let p = &mut prakriya;
    prepare_dhatu(p, args, false)?;
    run_main_rules(p, None, false)?;
    tripadi::run(p);

    Ok(prakriya)
}

/// Derives a single tinanta from the given conditions.
pub fn derive_tinanta(mut prakriya: Prakriya, args: &Tinanta) -> Result<Prakriya> {
    let p = &mut prakriya;
    let prayoga = args.prayoga();
    let lakara = args.lakara();
    let purusha = args.purusha();
    let vacana = args.vacana();
    p.add_tags(&[prayoga.as_tag(), purusha.as_tag(), vacana.as_tag()]);
    p.set_lakara(lakara);

    // Prayogas other than kartari will never be sarvadhatuka, since yak-vikarana is not
    // sarvadhatuka.
    let is_ardhadhatuka = match prayoga {
        Prayoga::Kartari => lakara.is_ardhadhatuka(),
        _ => true,
    };

    prepare_dhatu(p, args.dhatu(), is_ardhadhatuka)?;
    add_lakara_and_decide_pada(p, lakara);
    tin_pratyaya::adesha(p, purusha, vacana);
    samjna::run(p);
    run_main_rules(p, Some(lakara), is_ardhadhatuka)?;
    tripadi::run(p);

    Ok(prakriya)
}

/// Derives a single subanta from the given conditions.
pub fn derive_subanta(mut prakriya: Prakriya, args: &Subanta) -> Result<Prakriya> {
    let p = &mut prakriya;
    prepare_pratipadika(p, args.pratipadika())?;

    p.add_tag(args.linga().as_tag());
    pratipadika_karya::run_napumsaka_rules(p);

    sup_karya::run(p, args.linga(), args.vibhakti(), args.vacana());
    samjna::run(p);

    samasa::run_rules_for_avyayibhava(p);

    // Add strI-pratyayas. This should be done after adding the sup-pratyaya so that we satisfy the
    // following constraints:
    //
    // - su~ must be added before sup-luk (7.1.23)
    // - sup-luk must be checked before changing adas to ada (7.2.102)
    // - ada must be in place before running stritva (4.1.4)
    angasya::run_before_stritva(p);
    stritva::run(p);

    run_main_rules(p, None, false)?;
    tripadi::run(p);

    Ok(prakriya)
}

/// Derives a single krdanta from the given conditions.
pub fn derive_krdanta(mut prakriya: Prakriya, args: &Krdanta) -> Result<Prakriya> {
    let p = &mut prakriya;
    prepare_krdanta(p, args)?;
    run_main_rules(p, None, true)?;
    tripadi::run(p);

    Ok(prakriya)
}

pub fn derive_taddhitanta(mut prakriya: Prakriya, args: &Taddhitanta) -> Result<Prakriya> {
    let p = &mut prakriya;
    prepare_taddhitanta(p, args)?;

    run_main_rules(p, None, false)?;
    tripadi::run(p);

    Ok(prakriya)
}

pub fn derive_stryanta(mut prakriya: Prakriya, pratipadika: &Pratipadika) -> Result<Prakriya> {
    let p = &mut prakriya;
    prepare_pratipadika(p, pratipadika)?;

    p.add_tag(Tag::Stri);

    stritva::run(p);
    samjna::run(p);
    run_main_rules(p, None, false)?;
    tripadi::run(p);

    Ok(prakriya)
}

/// Creates a dummy sup-pratyaya.
///
/// Scope: samasas
fn make_sup_pratyaya(vibhakti: crate::args::Vibhakti) -> Term {
    use crate::args::Vibhakti::*;
    use crate::core::Tag as T;
    let (u, vibhakti) = match vibhakti {
        Prathama | Sambodhana => ("su~", T::V1),
        Dvitiya => ("am", T::V2),
        Trtiya => ("wA", T::V3),
        Caturthi => ("Ne", T::V4),
        Panchami => ("Nasi~", T::V5),
        Sasthi => ("Nas", T::V6),
        Saptami => ("Ni", T::V7),
    };

    let mut su = Term::make_upadesha(u);
    su.add_tags(&[T::Pratyaya, T::Sup, T::Vibhakti, T::Pada, vibhakti]);
    su
}

pub fn derive_samasa(mut prakriya: Prakriya, args: &Samasa) -> Result<Prakriya> {
    use crate::args::SamasaType;

    let p = &mut prakriya;
    prepare_samasa(p, args)?;

    if args.samasa_type() == SamasaType::Avyayibhava {
        samjna::run(p);
        samasa::run_rules_for_avyayibhava(p);
    }

    samjna::try_decide_pratipadika(p);

    run_main_rules(p, None, false)?;
    tripadi::run(p);

    Ok(prakriya)
}

pub fn derive_vakya(mut prakriya: Prakriya, padas: &[Pada]) -> Result<Prakriya> {
    for pada in padas {
        match pada {
            Pada::Subanta(s) => {
                let mut stack = PrakriyaStack::new(false, false, false);
                stack.find_all(|p| derive_subanta(p, s));

                if let Some(p) = stack.prakriyas().first() {
                    prakriya.extend(p.terms());
                }
            }
            Pada::Tinanta(t) => {
                let mut stack = PrakriyaStack::new(false, false, false);
                stack.find_all(|p| derive_tinanta(p, t));

                if let Some(p) = stack.prakriyas().first() {
                    prakriya.extend(p.terms());
                }
            }
            Pada::Dummy(s) => {
                let mut pada = Term::make_upadesha(s);
                pada.add_tags(&[Tag::Pada]);
                prakriya.push(pada);
            }
            Pada::Nipata(s) => {
                let mut pada = Term::make_upadesha(s);
                pada.add_tags(&[Tag::Pada, Tag::Avyaya, Tag::Nipata]);
                if pada.has_antya('N') || pada.has_antya('Y') {
                    pada.set_antya("");
                }
                prakriya.push(pada);
            }
        }
    }

    let p = &mut prakriya;
    samjna::try_pragrhya_rules(p);
    run_main_rules(p, None, false)?;
    tripadi::run(p);

    Ok(prakriya)
}
