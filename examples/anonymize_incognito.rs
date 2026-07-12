use std::fs::File;

use parallel_k::{
    algorithms::{anonymization_algorithm::AlgorithmError, incognito::Incognito},
    anonymize::Anonymizer,
    data::{
        dataset::Dataset,
        qi::{QIType, QuasiIdentifiers},
    },
};

use polars::prelude::*;

#[path = "./util/util.rs"]
mod util;


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let df = util::clean_adult_csv("adult.csv".into())?;

    unsafe { std::env::set_var("POLARS_FMT_MAX_COLS", df.width().to_string()); }
    unsafe { std::env::set_var("POLARS_MAX_THREADS", "1"); }

    let k = 100;
    let qis = QuasiIdentifiers::from_column_names(&[
        ("age", QIType::Numerical{ leaf_bucket_size: 1 }),
        ("workclass", QIType::Categorical{ path_to_json_hierarchy: "taxonomies/workclass.json".into() }),
        ("education", QIType::Categorical{ path_to_json_hierarchy: "taxonomies/education.json".into() }),
        ("marital-status", QIType::Categorical{ path_to_json_hierarchy: "taxonomies/marital-status.json".into() }),
        ("occupation", QIType::Categorical{ path_to_json_hierarchy: "taxonomies/occupation.json".into() }),
        ("relationship", QIType::Categorical{ path_to_json_hierarchy: "taxonomies/relationship.json".into() }),
        ("race", QIType::Categorical{ path_to_json_hierarchy: "taxonomies/race.json".into() }),
        ("gender", QIType::Categorical{ path_to_json_hierarchy: "taxonomies/gender.json".into() }),
    ]);

    let qi_colnames: Vec<_> = qis.0.iter().map(|x| x.column_name.as_str()).collect();

    let dataset = Dataset::build(&df, &qis)?;
    let anonymizer = Anonymizer::new(dataset, qis.clone());

    let mut result_maybe = anonymizer.run(&Incognito::default(), k);

    match result_maybe {
        Ok(mut result) => {
            result.anonymized_dataset.df.rechunk_mut();

            println!(
                "[Incognito k=100] {} rows -> {} anonymized, {} suppressed ({:.2?})",
                result.rows_original,
                result.anonymized_dataset.df.height(),
                result.rows_suppressed,
                result.duration,
            );

            println!("{}", result.anonymized_dataset.df.head(Some(15)));

            let verified = util::verify_k_anonymity(
                &result.anonymized_dataset.df,
                &qi_colnames,
                k as usize
            );
            println!("k=100 anonymity satisfied: {verified}");

            let mut out = result.anonymized_dataset.df;
            let mut file = File::create("output.csv")?;
            CsvWriter::new(&mut file)
                .include_header(true)
                .with_separator(b',')
                .finish(&mut out)?;
            println!("wrote output.csv ({} rows)", out.height());

            Ok(())
        },
        Err(err) => {
            Err(err)
        },
    }
}
