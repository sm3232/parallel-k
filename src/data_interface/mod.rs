use polars::prelude::*;
pub fn replace_qmarks_with_none(df: &mut DataFrame) -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..df.width() {
        if let Some(can) = df[i].dtype().can_cast_to(&DataType::String) && can {
            df.try_apply_at_idx(i, |series| {
                let cast = series.str()?;
                cast.set(&cast.equal("?"), None)
            })?;
        }
    };
    Ok(())
}
