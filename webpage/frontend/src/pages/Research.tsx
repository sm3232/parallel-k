import '../styles/pages.css';
import '../styles/researchpage.css';

const Research = () => {
    return (
        <div className="page_container">
            <div className="top_section">
				<h1>Research Overview</h1>

				<div className='top_section_background'></div>
			</div>

            <div className="page_sub_section">
                <h2>Project Background</h2>
                <div className="research_background_container">
                    <div className="text_background">Privacy-Preserving Data Publishing (PPDP) is a field concerned with releasing datasets to the public or third
                        parties in a way that protects the privacy of individuals whose records appear in the data. A foundational
                        privacy model in this field is k-anonymity, which guarantees that every record in a published dataset is
                        indistinguishable from at least k-1 other records based on a set of quasi-identifier (QI) attributes, QI are
                        attributes that, while not directly identifying, can be combined to re-identify individuals (e.g., age, zip code, gender).
                    </div>

                    <div className="text_background">
                        Two well-established algorithms for achieving k-anonymity are k-Mondrian and k-Incognito. This research
                        project proposes modifications to both algorithms that improve data utility while preserving the k-anonymity
                        guarantee, and introduces a parallel execution framework to evaluate anonymization quality across a range of k
                        values simultaneously.
                    </div>
                </div>

                <h2>Motivation For Modification</h2>
                <div className="motivation_for_modification_container">
                    <div className="text_background">
                        Standard k-Mondrian and k-Incognito treat all quasi-identifier attributes with generalization, replacing specific
                        values with broader ranges or categories. While effective, this approach can result in significant information
                        loss, particularly for numerical attributes where range generalization discards distributional information.
                    </div>

                    <div className="text_background">
                        This project proposes a hybrid anonymization strategy that applies different techniques depending on the type
                        of quasi-identifier attribute:
                    </div>

                    <div className="sub_bullet">
                        1.  Numerical QIs are anonymized using a taxonomy tree to form equivalence classes, after which the
                            range label of each node is replaced with the mean of the actual values of all records in that
                            equivalence class. This preserves the grouping structure of the taxonomy tree while improving data
                            utility by replacing uninformative ranges with a more representative numerical value.
                    </div>
                    <div className="sub_bullet">
                        2.  Categorical QIs are anonymized using tree-based generalization, where each value is replaced with an
                            ancestor node in a predefined taxonomy tree. The lowest common ancestor (LCA) of all values within
                            a partition is used, ensuring the minimum necessary generalization is applied.
                    </div>

                    <div className="text_background">
                        These modifications are applied to both k-Mondrian and k-Incognito, preserving each algorithm&#39;s core
                        partitioning and grouping logic while improving the quality of the anonymized output. Importantly, both
                        algorithms adopt a methodologically consistent anonymization philosophy: equivalence classes are formed first
                        using taxonomy tree structures, and numerical QI values are then replaced with the mean of each class. The key
                        difference between the two algorithms lies solely in how equivalence classes are formed, recursive partitioning
                        in k-Mondrian versus lattice-based search in k-Incognito.
                    </div>
                </div>

                <h2>Modified Algorithms</h2>
                <div className="modified_algorithms_container">
                    <br></br>
                    <div className="sub_bullet">
                        k-Mondrian is like a real estate agent dividing a city map into neighbourhoods by drawing
                        straight lines. Fast, practical, but the lines are drawn one at a time without seeing the full
                        picture.
                    </div>
                    <div className="sub_bullet">
                        k-Incognito is like a city planner who considers all possible ways to draw the neighbourhood
                        boundaries and picks the one that best balances privacy and detail. It&#39;s slower but more
                        deliberate and globally optimal.
                    </div>

                    <h4>1. Modified K-Mondrian</h4>
                    <div className="text_background">
                        k-Mondrian is a top-down, recursive partitioning algorithm. It divides the dataset into increasingly smaller
                        partitions along QI dimensions until no further split can be made without violating k-anonymity.
                    </div>

                    <div className="text_background">
                        In the modified version, the partitioning phase remains unchanged; the dataset is split recursively using the
                        median of numerical QIs or taxonomy tree branches of categorical QIs as cut points. The key change occurs in
                        the anonymization phase: once final partitions are established, they serve as the equivalence classes. For each
                        equivalence class, numerical QI values are replaced with the mean of all values in that class for the respective
                        attribute, discarding the range that would have been used in standard Mondrian. Categorical QI values are
                        replaced with the lowest common ancestor (LCA) of all values in that class according to the predefined
                        taxonomy tree.
                    </div>

                    <h4>2. Modified K-Incognito</h4>
                    <div className="text_background">
                        k-Incognito is a bottom-up lattice search algorithm. It searches over a generalization lattice, a structured space
                        of all possible generalization combinations, to find the least-generalizing configuration that satisfies k-
                        anonymity.
                    </div>

                    <div className="text_background">
                        In the modified version, the lattice is constructed over both numerical and categorical QIs, using their
                        respective predefined taxonomy trees. For numerical QIs, the taxonomy tree is defined with range-based nodes
                        as usual, the ranges serve purely as a structural mechanism to drive the lattice search and determine
                        generalization levels. The lattice search proceeds as normal, identifying the optimal generalization combination
                        that satisfies k-anonymity and produces equivalence classes across all QIs.
                    </div>

                    <div className="text_background">
                        Once equivalence classes are formed, the range labels on numerical QI nodes are discarded and replaced
                        with the mean of the actual values of all records in that equivalence class for each numerical attribute.
                        Categorical QI values are replaced with the taxonomy tree ancestor node dictated by the selected generalization
                        level. The published dataset therefore contains mean values for numerical QIs and taxonomy node labels for
                        categorical QIs, never raw ranges.
                    </div>

                    <div className="text_background">
                        This design keeps the lattice search fully intact and tractable, as the taxonomy tree structure for numerical QIs
                        remains static throughout the search. The mean replacement is applied purely as a post-processing step after
                        equivalence classes are finalized, and does not affect the grouping decisions made during the search.
                    </div>
                </div>

                <h2>Parallel Execution Framework</h2>
                <div className="execution_framework_container">
                    <div className="text_background">
                        A central component of this research is the evaluation of anonymization quality across multiple values of k
                        simultaneously. Rather than running each algorithm once for a single k value, the framework will execute the
                        modified k-Mondrian and modified k-Incognito algorithms in parallel for a range of k values.
                    </div>

                    <div className="text_background">
                        Each parallel execution produces an independently anonymized version of the dataset for a specific k value.
                        This allows the research to systematically study the trade-off between privacy and information loss as k
                        increases, a relationship that is well-known in theory but less studied empirically under hybrid anonymization
                        strategies. 
                    </div>

                    <div className="text_background">
                        The parallel framework will:
                    </div>

                    <div className="sub_bullet">
                        1. Accept a dataset and a defined range of k values as input.
                    </div>

                    <div className="sub_bullet">
                        2. Spawn a parallel process for each k value, running both modified algorithms independently.
                    </div>

                    <div className="sub_bullet">
                        3. Each process produces two anonymized datasets for its assigned k value: one from modified k-
                        Mondrian, one from modified k-Incognito.
                    </div>

                    <div className="sub_bullet">
                        4. Record privacy and information loss metrics for each anonymized version upon completion.
                    </div>
                </div>
            </div>
        </div>
    )
}

export default Research;