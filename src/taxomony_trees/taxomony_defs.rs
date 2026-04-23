use crate::taxonomy::CategoricalHierarchy;

//  From HTML we were given
//  name: "workclass",
//     tree: {
//       label: "*",
//       children: [
//         { label: "Employed", children: [
//           { label: "Private-Sector", children: [
//             { label: "Private" }
//           ]},
//           { label: "Self-Employed", children: [
//             { label: "Self-emp-not-inc" },
//             { label: "Self-emp-inc" }
//           ]},
//           { label: "Government", children: [
//             { label: "Federal-gov" },
//             { label: "Local-gov" },
//             { label: "State-gov" }
//           ]}
//         ]},
//         { label: "Non-Earning", children: [
//           { label: "Without-pay" },
//           { label: "Never-worked" }
//         ]},
//         { label: "Unknown", children: [
//           { label: "?" }
//         ]}
//       ]
//     }

pub fn workclass_hierarchy() -> CategoricalHierarchy {
    CategoricalHierarchy::new("*").with_children(vec![
        CategoricalHierarchy::new("Employed").with_children(vec![
            CategoricalHierarchy::new("Private-Sector").with_children(vec![
                CategoricalHierarchy::new("Private"),
            ]),

            CategoricalHierarchy::new("Self-Employed").with_children(vec![
                CategoricalHierarchy::new("Self-emp-not-inc"),
                CategoricalHierarchy::new("Self-emp-inc")
            ]),

            CategoricalHierarchy::new("Governemnt").with_children(vec![
                CategoricalHierarchy::new("Federal-gov"),
                CategoricalHierarchy::new("Local-gov"),
                CategoricalHierarchy::new("State-gov"),
            ]),

        ]),
        // Not 100% sure about these ones, since they don't show up 
        // at all in the csv data we were given. so gonna just leave them commented out
        CategoricalHierarchy::new("Non-Earning").with_children(vec![
            CategoricalHierarchy::("Without-pay"),
            CategoricalHierarchy::("Never-worked")
        ]),
        CategoricalHierarchy::new("Unknown").with_children(vec![
            CategoricalHierarchy::("?")
        ])
    ])
}

// {
//     name: "education",
//     tree: {
//       label: "*",
//       children: [
//         { label: "Pre-College", children: [
//           { label: "Primary", children: [
//             { label: "Preschool" },
//             { label: "1st-4th" },
//             { label: "5th-6th" }
//           ]},
//           { label: "Middle-School", children: [
//             { label: "7th-8th" },
//             { label: "9th" }
//           ]},
//           { label: "High-School", children: [
//             { label: "10th" },
//             { label: "11th" },
//             { label: "12th" },
//             { label: "HS-grad" }
//           ]}
//         ]},
//         { label: "College", children: [
//           { label: "Undergraduate", children: [
//             { label: "Some-college" },
//             { label: "Assoc-voc" },
//             { label: "Assoc-acdm" },
//             { label: "Bachelors" }
//           ]}
//         ]},
//         { label: "Post-Graduate", children: [
//           { label: "Graduate", children: [
//             { label: "Masters" },
//             { label: "Prof-school" },
//             { label: "Doctorate" }
//           ]}
//         ]}
//       ]
//     }
//   },

pub fn education_hierarchy() -> CategoricalHierarchy {
    CategoricalHierarchy::new("*").with_children(vec![
        CategoricalHierarchy::new("Pre-College").with_children(vec![
            CategoricalHierarchy::new("Primary").with_children(vec![
                CategoricalHierarchy::new("Preschool"),
                CategoricalHierarchy::new("1st-4th"),
                CategoricalHierarchy::new("5th-6th"),
            ]),

            CategoricalHierarchy::new("Middle-School").with_children(vec![
                CategoricalHierarchy::new("7th-8th"),
                CategoricalHierarchy::new("9th")
            ]),

            CategoricalHierarchy::new("High-School").with_children(vec![
                CategoricalHierarchy::new("10th"),
                CategoricalHierarchy::new("11th"),
                CategoricalHierarchy::new("12th"),
                CategoricalHierarchy::new("HS-grad"),
            ]),
        ]),

        CategoricalHierarchy::new("College").with_children(vec![
            CategoricalHierarchy::new("Undergraduate").with_children(vec![
                CategoricalHierarchy::new("Some-college"),
                CategoricalHierarchy::new("Assoc-voc"),
                CategoricalHierarchy::new("Assoc-acdm"),
                CategoricalHierarchy::new("Bachelors"),
            ])
        ]),

        CategoricalHierarchy::new("Post-Graduate").with_children(vec![
            CategoricalHierarchy::new("Graduate").with_children(vec![
                CategoricalHierarchy::new("Masters"),
                CategoricalHierarchy::new("Prof-school"),
                CategoricalHierarchy::new("Assoc-acdm"),
                CategoricalHierarchy::new("Doctorate"),
            ])
        ])
    ])
}

//   {
//     name: "marital-status",
//     tree: {
//       label: "*",
//       children: [
//         { label: "Married", children: [
//           { label: "Married-civ-spouse" },
//           { label: "Married-AF-spouse" },
//           { label: "Married-spouse-absent" }
//         ]},
//         { label: "Previously-Married", children: [
//           { label: "Divorced" },
//           { label: "Separated" },
//           { label: "Widowed" }
//         ]},
//         { label: "Single", children: [
//           { label: "Never-married" }
//         ]}
//       ]
//     }
//   },
//   {
//     name: "occupation",
//     tree: {
//       label: "*",
//       children: [
//         { label: "White-Collar", children: [
//           { label: "Exec-managerial" },
//           { label: "Prof-specialty" },
//           { label: "Adm-clerical" },
//           { label: "Sales" },
//           { label: "Tech-support" }
//         ]},
//         { label: "Blue-Collar", children: [
//           { label: "Craft-repair" },
//           { label: "Machine-op-inspct" },
//           { label: "Handlers-cleaners" },
//           { label: "Transport-moving" },
//           { label: "Farming-fishing" }
//         ]},
//         { label: "Service", children: [
//           { label: "Other-service" },
//           { label: "Priv-house-serv" },
//           { label: "Protective-serv" }
//         ]},
//         { label: "Military", children: [
//           { label: "Armed-Forces" }
//         ]},
//         { label: "Unknown", children: [
//           { label: "?" }
//         ]}
//       ]
//     }
//   },
//   {
//     name: "relationship",
//     tree: {
//       label: "*",
//       children: [
//         { label: "Spouse", children: [
//           { label: "Husband" },
//           { label: "Wife" }
//         ]},
//         { label: "Child", children: [
//           { label: "Own-child" }
//         ]},
//         { label: "Unrelated", children: [
//           { label: "Not-in-family" },
//           { label: "Other-relative" },
//           { label: "Unmarried" }
//         ]}
//       ]
//     }
//   },
//   {
//     name: "race",
//     tree: {
//       label: "*",
//       children: [
//         { label: "White", children: [
//           { label: "White" }
//         ]},
//         { label: "Non-White", children: [
//           { label: "Black" },
//           { label: "Asian-Pac-Islander" },
//           { label: "Amer-Indian-Eskimo" },
//           { label: "Other" }
//         ]}
//       ]
//     }
//   },
//   {
//     name: "sex",
//     tree: {
//       label: "*",
//       children: [
//         { label: "Male" },
//         { label: "Female" }
//       ]
//     }
//   },
//   {
//     name: "native-country",
//     tree: {
//       label: "*",
//       children: [
//         { label: "Americas", children: [
//           { label: "North-America", children: [
//             { label: "United-States" },
//             { label: "Canada" }
//           ]},
//           { label: "Central-America", children: [
//             { label: "Mexico" },
//             { label: "Guatemala" },
//             { label: "Honduras" },
//             { label: "El-Salvador" },
//             { label: "Nicaragua" }
//           ]},
//           { label: "Caribbean", children: [
//             { label: "Cuba" },
//             { label: "Jamaica" },
//             { label: "Haiti" },
//             { label: "Dominican-Republic" },
//             { label: "Puerto-Rico" },
//             { label: "Trinadad&Tobago" },
//             { label: "Outlying-US" }
//           ]},
//           { label: "South-America", children: [
//             { label: "Columbia" },
//             { label: "Peru" },
//             { label: "Ecuador" }
//           ]}
//         ]},
//         { label: "Europe", children: [
//           { label: "Western-Europe", children: [
//             { label: "Germany" },
//             { label: "England" },
//             { label: "Italy" },
//             { label: "France" },
//             { label: "Portugal" },
//             { label: "Greece" },
//             { label: "Ireland" },
//             { label: "Scotland" },
//             { label: "Holand-Netherlands" }
//           ]},
//           { label: "Eastern-Europe", children: [
//             { label: "Poland" },
//             { label: "Yugoslavia" },
//             { label: "Hungary" }
//           ]}
//         ]},
//         { label: "Asia", children: [
//           { label: "East-Asia", children: [
//             { label: "China" },
//             { label: "Japan" },
//             { label: "Taiwan" },
//             { label: "Hong" }
//           ]},
//           { label: "South-Asia", children: [
//             { label: "India" }
//           ]},
//           { label: "SE-Asia", children: [
//             { label: "Philippines" },
//             { label: "Vietnam" },
//             { label: "Cambodia" },
//             { label: "Laos" },
//             { label: "Thailand" }
//           ]},
//           { label: "Middle-East", children: [
//             { label: "Iran" }
//           ]}
//         ]},
//         { label: "Other", children: [
//           { label: "South" }
//         ]},
//         { label: "Unknown", children: [
//           { label: "?" }
//         ]}
//       ]
//     }
//   }