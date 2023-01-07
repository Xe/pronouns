use maud::{html, Markup, Render};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub struct PronounSet {
    pub nominative: String,
    pub accusative: String,
    pub determiner: String,
    pub possessive: String,
    pub reflexive: String,
}

#[derive(Clone, Debug)]
pub struct PronounTrie {
    inner: String,
    left: Option<Box<PronounTrie>>,
    right: Option<Box<PronounTrie>>,
    next: Option<Box<PronounTrie>>,
}

impl PronounTrie {
    // Build a trie out of a vector of pronouns.
    pub fn build(pronouns: Vec<PronounSet>) -> Self {
        let mut base: Option<Box<PronounTrie>> = None;

        for pronoun in pronouns {
            let key = vec![
                pronoun.nominative,
                pronoun.accusative,
                pronoun.determiner,
                pronoun.possessive,
                pronoun.reflexive,
            ];

            Self::insert_to_child(&mut base, key)
        }

        return *base.unwrap()
    }

    pub fn guess(&self, key: &mut Vec<String>) -> Vec<PronounSet> {
        let mut strings = self.guess_strings(key);

        strings.drain(..).filter_map(|x| if x.len() == 5 {
            Some(PronounSet {
                nominative: x[0].clone(),
                accusative: x[1].clone(),
                determiner: x[2].clone(),
                possessive: x[3].clone(),
                reflexive:  x[4].clone(),
            })
        } else {
            None
        }).collect()
    }

    fn guess_strings(&self, key: &mut Vec<String>) -> Vec<Vec<String>> {
        let car = key.get(0).clone();

        let mut result = Vec::new();

        let search_left = car.is_none() || car.unwrap() < &self.inner;
        let search_right = car.is_none() || car.unwrap() > &self.inner;
        let search_down = car.is_none() || car.unwrap() == &self.inner;

        if search_left {
            if let Some(left) = self.left.as_ref() {
                result.extend(left.guess_strings(key))
            }
        }

        if search_right {
            if let Some(right) = self.right.as_ref() {
                result.extend(right.guess_strings(key))
            }
        }

        if search_down {
            if let Some(next) = self.next.as_ref() {
                if !key.is_empty() {
                    key.remove(0);
                }
                let mut basket = next.guess_strings(key);

                let basket = basket.drain(..).map(|x| {
                    let mut y = vec![self.inner.clone()];
                    y.extend(x);
                    y
                });

                result.extend(basket.collect::<Vec<Vec<String>>>());
            } else {
                result.extend(vec![vec![self.inner.clone()]]);
            }
        }

        result
    }

    // Get all strings in the set.
    pub fn gather(&self) -> Vec<PronounSet> {
        self.guess(&mut Vec::new())
    }

    fn new(inner: String) -> Self {
        Self {
            inner,
            left: None,
            right: None,
            next: None,
        }
    }

    fn insert(&mut self, mut key: Vec<String>) {
        let car = &key[0];

        if car < &self.inner {
            Self::insert_to_child(&mut self.left, key);
        } else if car > &self.inner {
            Self::insert_to_child(&mut self.right, key);
        } else {
            key.remove(0);
            Self::insert_to_child(&mut self.next, key);
        }
    }

    fn insert_to_child(s: &mut Option<Box<Self>>, mut v: Vec<String>) {
        match s {
            None => {
                let car = v[0].clone();
                v.remove(0);
                let cons = v;

                let mut child = Self::new(car);

                if !cons.is_empty() {
                    Self::insert_to_child(&mut child.next, cons);
                }

                s.replace(Box::new(child));
            },
            Some(t) => t.insert(v),
        };
    }
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
                li { em{(titlecase::titlecase(&self.nominative))} " went to the park." }
                li { "I went with " i{(self.accusative)} "." }
                li { em{(titlecase::titlecase(&self.nominative))} " brought " em{(self.determiner)} " frisbee." }
                li { "At least I think it was " em{(self.possessive)} "." }
                li { em{(titlecase::titlecase(&self.nominative))} " threw the frisbee to " em{(self.reflexive)} "." }
            }
        }
    }
}

impl PronounSet {
    pub fn url(&self) -> String {
        format!("/{}/{}/{}/{}/{}", self.nominative, self.accusative, self.determiner, self.possessive, self.reflexive)
    }
}
