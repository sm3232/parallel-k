use std::fs::File;

use polars::prelude::*;
use parallel_k::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut df = CsvReadOptions::default()
        .with_has_header(true)
        .with_parse_options(CsvParseOptions::default()
            .with_try_parse_dates(true)
        )
        .try_into_reader_with_file_path(Some("adult.csv".into()))?
        .finish()?;

    unsafe { std::env::set_var("POLARS_FMT_MAX_COLS", df.width().to_string()); }

    data_interface::replace_qmarks_with_none(&mut df)?;

    let null_count = df.null_count();
    println!("{null_count}");

    df = df.drop_nulls::<String>(None)?;

    let null_count = df.null_count();
    println!("{null_count}");

    df = df.unique_stable(None, UniqueKeepStrategy::First, None)?;
    println!("{df}");

    df = df.lazy().with_column(
        col("income").replace(
            lit(Series::new("from".into(), &["<=50K", ">50K"])),
            lit(Series::new("to".into(), &[0i64, 1i64])),
        ).cast(DataType::Int64)
    ).collect()?;

    println!("{df}");

    let mut outf = File::create("adult_clean.csv").expect("could not create file");
    CsvWriter::new(&mut outf)
        .include_header(true)
        .with_separator(b',')
        .finish(&mut df)?;
    Ok(())
}
