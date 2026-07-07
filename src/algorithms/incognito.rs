/*
* Citation: based on
* UTD Anonymization ToolBox
* The University of Texas at Dallas
*/


use std::collections::{HashMap, HashSet};

use polars::{frame::row::Row, prelude::*};

use crate::{
    algorithms::anonymization_algorithm::{AlgorithmError, AnonymizationAlgorithm},
    data::{QIType, QuasiIdentifier, dataset::Dataset, qi::QuasiIdentifiers}, taxonomy::TaxonomyManager,
};

#[derive(Clone)]
pub enum EquivValue {
    Numeric((i64, i64)),
    Categorical(String)
}
impl std::fmt::Display for EquivValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EquivValue::Numeric((lo, hi)) => writeln!(f, "Numeric(({}, {}))", lo, hi),
            EquivValue::Categorical(category) => writeln!(f, "Categorical({})", category),
        }
    }
}
impl std::fmt::Debug for EquivValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { writeln!(f, "{self}") }
}
enum EquivColumn {
    Numeric(StructChunked),
    Categorical(StringChunked)
}
impl EquivValue {
    pub fn as_expr(&self) -> Expr {
        match self {
            EquivValue::Numeric(range) => {
                as_struct(
                    vec![
                        lit(range.0).alias("low"),
                        lit(range.1).alias("high")
                    ]
                )
            },
            EquivValue::Categorical(category) => lit(category.to_owned()),
        }
    }
    pub fn new (any_val: &AnyValue) -> Option<Self> {
        match any_val {
            AnyValue::String(s) => {
                Some(EquivValue::Categorical(s.to_string()))
            }
            AnyValue::StringOwned(s) => {
                Some(EquivValue::Categorical(s.to_string()))
            }
            AnyValue::StructOwned(payload) => {
                let values = &payload.0;

                if let (Some(AnyValue::Int64(low)), Some(AnyValue::Int64(high))) = (values.get(0), values.get(1)) {
                    Some(EquivValue::Numeric((*low, *high)))
                } else {
                    None
                }
            }

            AnyValue::Struct(_row_idx, _arrow_array, _fields) => {
                let owned_val = any_val.clone().to_owned(); 

                if let AnyValue::StructOwned(payload) = owned_val {
                    let values = &payload.0;
                    if let (Some(AnyValue::Int64(low)), Some(AnyValue::Int64(high))) = (values.get(0), values.get(1)) {
                        return Some(EquivValue::Numeric((*low, *high)));
                    }
                }
                None
            }
            _ => None,
        }
    }
}

#[derive(Clone,Default)]
pub struct EquivTable {
    qis: QuasiIdentifiers,
    df: DataFrame,
}
trait AuxTable {
    fn name(&self) -> &str;
    fn dataframe(&self) -> &DataFrame;
    fn drop_table(&mut self);
}
impl EquivTable {
    pub fn remove_eid(&mut self, eid: i64) -> PolarsResult<()> {
        self.df = self
            .df
            .clone()
            .lazy()
            .filter(col("EID").neq(lit(eid)))
            .collect()?;
        Ok(())
    }
    pub fn set_generalization(&mut self, eid: i64, new_vals: &[EquivValue]) -> PolarsResult<()> {
        let mut lf = self.df.clone().lazy();
        for (i, attr) in self.qis.0.iter().enumerate() {
            let col_name = attr.incognito_colname.clone();
            let new_val = new_vals[i].clone();
            lf = lf.with_column(
                when(col("EID").eq(lit(eid)))
                    .then(new_vals[i].as_expr())
                    .otherwise(col(&col_name))
                    .alias(&col_name),
            );
        }
        self.df = lf.collect()?;
        Ok(())
    }
    pub fn get_generalization(&self, eid: i64) -> PolarsResult<Option<Vec<EquivValue>>> {
        let result = self
            .df
            .clone()
            .lazy()
            .filter(col("EID").eq(lit(eid)))
            .collect()?;

        if result.height() == 0 {
            return Ok(None);
        }

        let mut gen_vals = Vec::with_capacity(self.qis.0.len());
        for attr in &self.qis.0 {
            let col_name = attr.incognito_colname.clone();
            match attr.qi_type {
                QIType::Numerical { .. } => {
                    let struct_ = result.column(&col_name)?.struct_()?;
                    let low = struct_.field_by_name("low")?.i64()?.get(0).unwrap().to_owned();
                    let high = struct_.field_by_name("high")?.i64()?.get(0).unwrap().to_owned();
                    gen_vals.push(EquivValue::Numeric((low, high)));
                },
                QIType::Categorical { .. } => {
                    let val = result.column(&col_name)?.str()?.get(0).unwrap().to_owned();
                    gen_vals.push(EquivValue::Categorical(val));
                },
            }
        }
        Ok(Some(gen_vals))
    }
    pub fn get_eid(&self, gen_vals: &[EquivValue]) -> PolarsResult<Option<i64>> {
        if gen_vals.len() != self.qis.0.len() {
            return Err(PolarsError::ShapeMismatch(
                format!(
                    "get_eid: expected {} generalised values, got {}",
                    self.qis.0.len(),
                    gen_vals.len()
                )
                .into(),
            ));
        }

        let mut filter_expr = lit(true);
        for (i, attr) in self.qis.0.iter().enumerate() {
            let col_name = attr.incognito_colname.clone();
            filter_expr = filter_expr
                .and(col(&col_name).eq(gen_vals[i].as_expr()));
        }

        let result = self.df.clone().lazy().filter(filter_expr).collect()?;

        if result.height() == 0 {
            Ok(None)
        } else {
            Ok(result.column("EID")?.i64()?.get(0))
        }
    }


    pub fn dataframe(&self) -> &DataFrame {
        &self.df
    }
}
#[derive(Clone,Default)]
pub struct AnonTable {
    qis: QuasiIdentifiers,
    df: DataFrame,
}
impl AnonTable {
    pub fn is_k_anonymous(&self, qis: QuasiIdentifiers, k: u32) ->Result<bool, PolarsError> {
        let is_k_anon_col = self.df.clone().lazy()
            .with_column(
                len().over(qis.0.iter().map(|x| 
                    col(x.incognito_colname.clone()),
                ).collect::<Vec<_>>())
                .alias("__is_k_anon")
            ).collect()?;
        let series = is_k_anon_col.column("__is_k_anon")?.as_series().unwrap();
        let min_k = series.min::<u32>()?.unwrap();
        Ok(min_k >= k)
    }

    pub fn dataframe(&self) -> &DataFrame {
        &self.df
    }


    pub fn from_remap(
        qis: QuasiIdentifiers,
        source: &AnonTable,
        eid_map: &DataFrame,
    ) -> PolarsResult<Self> {
        let rename_exprs = qis.0.iter().map(|x| col(format!("ATT_{}_right", x.index)).alias(x.incognito_colname.clone())).collect::<Vec<_>>();
        let drop_exprs = [PlSmallStr::from("NEW_EID")].into_iter().chain(qis.0.iter().map(|x| PlSmallStr::from(format!("ATT_{}_right", x.index)))).collect::<Vec<_>>();


        let df = source.df.clone().lazy()
            .join(eid_map.clone().lazy(),
                [col("EID")],
                [col("PREV_EID")],
                JoinArgs::new(JoinType::Inner)
            ).with_column(col("NEW_EID").alias("EID"))
            .with_columns(
                rename_exprs
            ).drop(Selector::ByName { names: drop_exprs.into(), strict: true })
            .collect()?;
        ;
 
        Ok(AnonTable { qis, df })
    }
}



#[derive(Clone)]
pub struct LatticeEntry {
    pub attr: QuasiIdentifier,
    pub root: HashMap<QuasiIdentifier, usize>,
    pub anon_table: AnonTable,
    pub equi_table: EquivTable,
}
impl std::fmt::Display for LatticeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut qis: Vec<_> = self.root.keys().collect();
        qis.sort_by_key(|x| x.column_name.to_owned());

        let mut ret = self.root.get(qis[0]).unwrap().to_string();
        for i in &qis[1..] {
            ret += &("_".to_owned() + (&self.root.get(i).unwrap().to_string()));
        }
        write!(f, "{ret}")
    }
}
impl std::fmt::Debug for LatticeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LatticeEntry").field("attr", &self.attr).field("root", &self.root).finish()
    }
}

impl LatticeEntry {
    pub fn from_root(root: HashMap<QuasiIdentifier, usize>, qis: &QuasiIdentifiers) -> Self {
        Self {
            root: root.clone(),
            attr: qis.0[0].clone(),
            anon_table: AnonTable::default(),
            equi_table: EquivTable::default()
        }
    }
    pub fn from_parent(parent: &Self, attr: QuasiIdentifier) -> Self {
        let mut me = Self {
            attr,
            root: parent.root.clone(),
            anon_table: AnonTable::default(),
            equi_table: EquivTable::default()
        };
        *me.root.get_mut(&me.attr).unwrap() += 1;
        me
    }

    pub fn height_at(&self, attr: QuasiIdentifier) -> usize { *self.root.get(&attr).unwrap() }

    pub fn generalizes_to(&self, other: &Self) -> bool {
        assert!(self.root.len() == other.root.len());
        for k in self.root.keys() {
            if self.root.get(k).unwrap() > other.root.get(k).unwrap() {
                return false;
            }
        }
        true
    }

    pub fn parents_name(&self) -> String{
        let mut parent_root = self.root.clone();
        *parent_root.get_mut(&self.attr).unwrap() -= 1;

        let mut qis: Vec<_> = parent_root.keys().collect();
        qis.sort_by_key(|x| x.column_name.to_owned());

        let mut ret = parent_root.get(qis[0]).unwrap().to_string();
        for i in &qis[1..] {
            ret += &("_".to_owned() + (&parent_root.get(i).unwrap().to_string()));
        }
        ret
    }
    pub fn set_tables(&mut self, anon: AnonTable, equi: EquivTable) {
        self.anon_table = anon;
        self.equi_table = equi;
    }
}

pub struct LatticeManager {
    dgh_depths: HashMap<QuasiIdentifier, usize>,
    qis: QuasiIdentifiers,
    roots: Vec<Option<LatticeEntry>>,
    next_index: usize,
    last_returned: Option<LatticeEntry>,
    successful_entries: Vec<LatticeEntry>
}

impl LatticeManager {
    pub fn new(super_root: LatticeEntry, dgh_depths: HashMap<QuasiIdentifier, usize>, qis: QuasiIdentifiers) -> Self {
        Self {
            dgh_depths,
            qis,
            roots: vec![Some(super_root.clone())],
            next_index: 0,
            last_returned: None,
            successful_entries: Vec::default()
        }
    }

    pub fn has_next(&mut self) -> bool {
        if self.next_index < self.roots.len() {
            true
        } else {
            self.next_index = 0;
            let mut new_roots = Vec::default();
            for i in 0..self.roots.len() {
                if self.roots[i].is_none() {
                    continue;
                }
                let curr = self.roots[i].as_ref().unwrap();
                for k in curr.attr.index..self.dgh_depths.len() {
                    let qi = self.qis.0.iter().find(|x| x.index == k).unwrap().clone();
                    let new = LatticeEntry::from_parent(curr, qi.clone());
                    if new.height_at(qi.clone()) <= *self.dgh_depths.get(&qi.clone()).unwrap() {
                        new_roots.push(Some(new));
                    }
                }
            }
            if new_roots.is_empty() {
                false
            } else {
                // Eventually this new_Roots.len() number goes down, and we reach 0. At that point
                // all combinations of generalizations have been considered.
                println!("Completed {} iterations, expanding with {} more", self.roots.len(), new_roots.len());
                self.roots = new_roots;
                true
            }
        }
    }

    pub fn get_successful(&self) -> Vec<LatticeEntry> {
        self.successful_entries.clone()
    }

    pub fn next_entry(&mut self) -> LatticeEntry {
        self.last_returned = self.roots[self.next_index].clone();
        self.last_returned.clone().unwrap()
    }

    pub fn set_result_ok(&mut self, anon: AnonTable, equi: EquivTable) {
        self.last_returned.as_mut().unwrap().set_tables(anon, equi);
        self.successful_entries.push(self.last_returned.as_mut().unwrap().clone());
        self.roots[self.next_index] = None;
        self.next_index += 1;
    }
    pub fn set_result_bad(&mut self) {
        self.next_index += 1;
    }
}



#[derive(Default)]
pub struct Incognito {}

impl AnonymizationAlgorithm for Incognito {
    fn name(&self) -> &str {
        "Incognito"
    }

    fn anonymize(
        &self,
        k: u32,
        dataset: Dataset,
        quasi_identifiers: QuasiIdentifiers,
    ) -> Result<Dataset, AlgorithmError> {
        let df_col_names: Vec<_> = dataset.df.get_column_names_owned().iter().map(|x| x.to_string()).collect();
        let mut resolve_id = "resolve_id".to_owned();
        while df_col_names.contains(&resolve_id) {
            resolve_id += "_";
        }

        let df = dataset.df.with_row_index(resolve_id.clone().into(), None)?;

        let aux = df.lazy().with_columns(
            dataset.taxonomies.numerical_taxonomies.iter().map(|(name, tax)| as_struct(vec![
                col(name.to_owned()).alias("low"),
                col(name.to_owned()).alias("high"),
            ]).alias(name.to_owned())).collect::<Vec<_>>()
        ).collect()?;

        let mut dgh_depths: HashMap<QuasiIdentifier, usize> = HashMap::default();
        let mut root: HashMap<QuasiIdentifier, usize> = HashMap::default();
        for qi in quasi_identifiers.0.iter() {
            dgh_depths.insert(qi.clone(), dataset.taxonomies.get_qi_height(qi.column_name.to_owned()));
            root.insert(qi.clone(), 0);
        }
        let super_root: LatticeEntry = LatticeEntry::from_root(root, &quasi_identifiers);


        let att_col_names: Vec<String> = quasi_identifiers.0.iter()
            .map(|attr| attr.incognito_colname.clone())
            .collect();
        let att_cols: Vec<Expr> = att_col_names.iter().map(col).collect();
 
        let select_exprs: Vec<Expr> = quasi_identifiers.0.iter()
            .map(|attr| col(attr.column_name.to_owned()).alias(attr.incognito_colname.clone()))
            .collect();
 
        let qi_df = aux.clone()
            .lazy()
            .select(
                std::iter::once(col(resolve_id.clone()))
                    .chain(select_exprs)
                    .collect::<Vec<_>>()
            )
            .collect()?;
        let resolve_df = aux
            .lazy()
            .select([
                all().exclude_cols(
                    quasi_identifiers.0.iter().map(|attr| attr.column_name.to_owned())
                ).into()
            ]).collect()?;
 
        let curr_equi_df = qi_df
            .clone()
            .lazy()
            .select(att_cols.clone())
            .unique_stable(Some(Selector::ByName { names: att_col_names.clone().iter().map(|x| PlSmallStr::from_string(x.to_owned())).collect::<Vec<_>>().into() , strict: true }), UniqueKeepStrategy::First)
            .collect()?
            .with_row_index("EID".into(), Some(1))?
            .lazy()
            .select(
                std::iter::once(col("EID").cast(DataType::Int64))
                    .chain(att_cols.clone())
                    .collect::<Vec<_>>()
            )
            .collect()?;
 
        let anon_df = qi_df
            .lazy()
            .join(
                curr_equi_df.clone().lazy(),
                att_cols.clone(),
                att_cols.clone(),
                JoinArgs::new(JoinType::Inner),
            )
            .select(
                [col(resolve_id.clone()), col("EID")]
                    .into_iter()
                    .chain(att_cols.clone())
                    .collect::<Vec<_>>()
            )
            .collect()?;
 

        let equi = EquivTable {
            qis: quasi_identifiers.clone(),
            df: curr_equi_df,
        };

        let anon = AnonTable {
            qis: quasi_identifiers.clone(),
            df: anon_df
        };


        let mut result = Self::anonymize_impl(
            anon,
            equi,
            LatticeManager::new(super_root.clone(), dgh_depths, quasi_identifiers.clone()),
            dataset.taxonomies.clone(),
            quasi_identifiers.clone(),
            k
        )?;


        result = result.lazy().with_columns(
            quasi_identifiers.0.iter().map(|x| match x.qi_type {
                QIType::Numerical { .. } => {
                    concat_str([
                        col(x.incognito_colname.clone()).struct_().field_by_name("low"),
                        col(x.incognito_colname.clone()).struct_().field_by_name("high"),
                    ], "-", true).alias(x.incognito_colname.clone())
                },
                QIType::Categorical { .. } => {
                    col(x.incognito_colname.clone())
                },
            }).collect::<Vec<_>>()
        ).collect()?;

        let renames = quasi_identifiers.0.into_iter().map(|x| {
            (x.incognito_colname.clone(), PlSmallStr::from(x.column_name.to_owned()))
        }).collect::<Vec<_>>();
        let renames_refs: Vec<_> = renames.iter().map(|(str, plsm)| (str.as_str(), plsm.clone())).collect();
        result.rename_many(renames_refs.into_iter())?;

        result = result.left_join(&resolve_df, [resolve_id.clone()], [resolve_id.clone()])?.drop_many(["EID", &resolve_id.clone()]);

        Ok(Dataset::from_anonymized(result, dataset.qis, dataset.taxonomies))
    }
}

impl Incognito {
    fn apply_generalization(
        last_equi: &EquivTable,
        qis: &QuasiIdentifiers,
        root: &LatticeEntry,
        taxonomy_man: &TaxonomyManager,
    ) -> PolarsResult<DataFrame> {
        let df = last_equi.dataframe();
        let n_rows = df.height();

        let old_eid: Vec<i64> = df.column("EID")?.i64()?.into_iter().map(|v| v.unwrap()).collect();
        let mut out_columns: Vec<Column> = vec![Column::new("PREV_EID".into(), old_eid)];

        for attr in &qis.0 {
            let col_name = attr.incognito_colname.clone();
            let levels = root.height_at(attr.clone());
            let raw = df.column(&col_name)?;

            let generalized: Column = match attr.qi_type {
                QIType::Numerical { .. } => {
                    let tax = taxonomy_man.numerical_taxonomies.get(&attr.column_name).unwrap();
                    let struct_ = raw.struct_()?;
                    let lo = struct_.field_by_name("low")?;
                    let hi = struct_.field_by_name("high")?;
                    let lo = lo.i64()?;
                    let hi = hi.i64()?;

                    let (lows, highs): (Vec<i64>, Vec<i64>) = lo
                        .into_iter()
                        .zip(hi.into_iter())
                        .map(|(l, h)| {
                            let mut range = (l.unwrap(), h.unwrap());
                            for level in 0..levels {
                                range = tax.generalize(range, level);
                            }
                            range
                        })
                        .unzip();

                    StructChunked::from_series(
                        col_name.clone().into(),
                        lows.len(),
                        [
                            Series::new("low".into(), lows),
                            Series::new("high".into(), highs),
                        ]
                            .iter(),
                    )?
                        .into_column()
                }
                QIType::Categorical { .. } => {
                    let tax = taxonomy_man.categorical_taxonomies.get(&attr.column_name).unwrap();
                    let strs = raw.str()?;

                    let values: Vec<String> = strs
                        .into_iter()
                        .map(|s| {
                            let mut category = s.unwrap().to_owned();
                            for _ in 0..levels {
                                category = tax.generalize(category);
                            }
                            category
                        })
                        .collect();

                    Series::new(col_name.clone().into(), values).into_column()
                }
            };

            out_columns.push(generalized);
        }

        DataFrame::new(n_rows, out_columns)
    }

    fn generalize(qis: QuasiIdentifiers, root: LatticeEntry, last_equi: &EquivTable, last_anon: &AnonTable, taxonomy_man: &TaxonomyManager, lattice_man: &mut LatticeManager, k: u32) -> Result<(), PolarsError> {
        // [PREV_EID, ATT_0, ATT_1, ..., ATT_N]
        let generalized = Self::apply_generalization(last_equi, &qis, &root, taxonomy_man)?;

        let qi_col_names: Vec<String> = qis
            .0
            .iter()
            .map(|attr| attr.incognito_colname.clone())
            .collect();

        let qi_cols: Vec<Expr> = qi_col_names.iter().map(col).collect();
 
        // [PREV_EID, NEW_EID, ATT_0, ATT_1, ..., ATT_N]
        let eid_map = generalized
            .clone()
            .lazy()
            .group_by([col("PREV_EID")]).agg([col(PlSmallStr::from_static("*")).item(false)])
            .with_row_index(PlSmallStr::from_str("NEW_EID"), Some(1))
            .collect()?;
 
        let curr_anon = AnonTable::from_remap(
            qis.clone(),
            last_anon,
            &eid_map,
        )?;
 

        let curr_equi_df = generalized.clone().lazy().join(eid_map.lazy(), [col("PREV_EID")], [col("PREV_EID")], JoinArgs::new(JoinType::Inner))
            .lazy()
            .rename(["NEW_EID"], ["EID"], true)
            .drop(Selector::ByName { names: ["PREV_EID".into()].into(), strict: true })
            .collect()?;

        let curr_equi = EquivTable {
            qis: qis.clone(),
            df: curr_equi_df,
        };
 
        if curr_anon.is_k_anonymous(qis.clone(), k)? {
            lattice_man.set_result_ok(curr_anon, curr_equi);
        } else {
            lattice_man.set_result_bad();
        }
        Ok(())
    }

    pub fn anonymize_impl(
        mut anon: AnonTable,
        mut equi: EquivTable,
        mut lattice_man: LatticeManager,
        taxonomy_man: TaxonomyManager,
        qis: QuasiIdentifiers,
        k: u32
    ) -> Result<DataFrame, AlgorithmError> {

        lattice_man.next_entry();

        if anon.is_k_anonymous(qis.clone(), k)? {
            lattice_man.set_result_ok(anon.clone(), equi.clone());
        } else {
            lattice_man.set_result_bad();
        }

        let mut seen: HashSet<String> = HashSet::default();

        while lattice_man.has_next() {
            let curr_root = lattice_man.next_entry();
            if seen.insert(curr_root.to_string()) {
                Self::generalize(qis.clone(), curr_root, &equi, &anon, &taxonomy_man, &mut lattice_man, k)?;
            } else {
                println!("Skipping {}", curr_root);
                lattice_man.set_result_bad();
            }
        }

        let successful = lattice_man.get_successful();
        if successful.is_empty() {
            return Err("no successful anonymizations found.".into());
        }
        let selection = successful
            .iter()
            .min_by_key(|entry| {
                entry.anon_table.df.clone().lazy().group_by([col("EID")]).agg([len()]).max().collect().unwrap().column("len").unwrap().u32().unwrap().first().unwrap()
            })
            .unwrap();
        let mut result = selection.anon_table.dataframe().clone();


        Ok(result)
    }
}
