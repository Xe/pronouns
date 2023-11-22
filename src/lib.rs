use heck::ToTitleCase;
use maud::{html, Markup, Render};
use serde::{Deserialize, Serialize};

mod trie;

pub use trie::PronounTrie;

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub struct PronounSet {
    pub nominative: String,
    pub accusative: String,
    pub determiner: String,
    pub possessive: String,
    pub reflexive: String,
    #[serde(default)]
    pub singular: bool,
}

impl Render for PronounSet {
    fn render(&self) -> Markup {
        html! {
            table {
                tr {
                    th { "Subject" }
                    td {(self.nominative)}
                }
                tr {
                    th { "Object" }
                    td {(self.accusative)}
                }
                tr {
                    th { "Dependent Possessive" }
                    td {(self.determiner)}
                }
                tr {
                    th { "Independent Possessive" }
                    td {(self.possessive)}
                }
                tr {
                    th { "Reflexive" }
                    td {(self.reflexive)}
                }
            }
            p {"Here are some example sentences with these pronouns:"}
            ul {
                li { em{(self.nominative.to_title_case())} " went to the park." }
                li { "I went with " i{(self.accusative)} "." }
                li { em{(self.nominative.to_title_case())} " brought " em{(self.determiner)} " frisbee." }
                li { "At least I think it was " em{(self.possessive)} "." }
                li {
                    em{(self.nominative.to_title_case())}
                    " threw the frisbee "
                    @if self.singular {
                        "to"
                    } @else {
                        "between"
                    }
                    " "
                    em{(self.reflexive)}
                    "."
                }
            }
            p {
                "This pronoun should be inflected as a "
                @if self.singular {
                    "singular"
                } @else {
                    "plural"
                }
                " pronoun."
            }
        }
    }
}

impl PronounSet {
    pub fn url(&self) -> String {
        format!(
            "/{}/{}/{}/{}/{}",
            self.nominative, self.accusative, self.determiner, self.possessive, self.reflexive
        )
    }

    pub fn title(&self) -> String {
        format!("{}/{}", self.nominative, self.accusative)
    }

    pub fn plural(&self) -> bool {
        self.determiner.ends_with('s')
    }
}
