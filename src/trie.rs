use super::PronounSet;

#[derive(Clone, Debug)]
pub struct PronounTrie {
    inner: String,
    left:  Option<Box<PronounTrie>>,
    right: Option<Box<PronounTrie>>,
    next:  Option<Box<PronounTrie>>,

    /// If this node terminates a PronounSet, store whether or not the pronoun is singular.
    singular: Option<bool>,
}

impl PronounTrie {
    /// Build a trie out of a vector of pronouns.
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

            Self::insert_to_child(&mut base, key, pronoun.singular);
        }

        *base.expect("non-empty input list")
    }

    /// Take a vector of optional strings and return a list of matching pronouns. If None is passed
    /// as one of the key element,s it may match any string.
    pub fn guess(&self, key: &mut Vec<Option<String>>) -> Vec<PronounSet> {
        // Expand wildcards to fill length 5. Not the prettiest code.
        let expansion = 5 - key.len();
        for (i, word) in key.iter().enumerate() {
            if word.is_none() {
                for _ in 0..expansion {
                    key.insert(i, None);
                }
                break;
            }
        }

        let mut strings = self.guess_strings(key);

        strings.drain(..).filter_map(|(x, singular)| if x.len() == 5 {
            Some(PronounSet {
                nominative: x[0].clone(),
                accusative: x[1].clone(),
                determiner: x[2].clone(),
                possessive: x[3].clone(),
                reflexive:  x[4].clone(),
                singular:   singular.unwrap_or(false),
            })
        } else {
            None
        }).collect()
    }

    /// Get all strings in the set.
    pub fn gather(&self) -> Vec<PronounSet> {
        self.guess(&mut Vec::new())
    }

    fn new(inner: String) -> Self {
        Self {
            inner,
            left: None,
            right: None,
            next: None,
            singular: None,
        }
    }

    fn insert(&mut self, mut key: Vec<String>, singular: bool) {
        let car = &key[0];

        if car < &self.inner {
            Self::insert_to_child(&mut self.left, key, singular);
        } else if car > &self.inner {
            Self::insert_to_child(&mut self.right, key, singular);
        } else {
            // We found where to insert, advance the key.
            key.remove(0);
            Self::insert_to_child(&mut self.next, key, singular);
        }
    }

    fn insert_to_child(s: &mut Option<Box<Self>>, mut v: Vec<String>, singular: bool) {
        match s {
            None => {
                let car = v[0].clone();
                v.remove(0);
                let cons = v;

                let mut child = Self::new(car);

                if !cons.is_empty() {
                    Self::insert_to_child(&mut child.next, cons, singular);
                } else {
                    child.singular = Some(singular);
                }

                s.replace(Box::new(child));
            },
            Some(t) => t.insert(v, singular),
        };
    }

    fn guess_strings(&self, key: &mut Vec<Option<String>>) -> Vec<(Vec<String>, Option<bool>)> {
        let car = key.get(0).map(|x| x.as_ref()).clone().flatten();

        let wildcard = car.is_none();

        let mut result = Vec::new();

        let search_left = wildcard || car.unwrap() < &self.inner;
        let search_right = wildcard || car.unwrap() > &self.inner;
        let search_down = wildcard || car.unwrap() == &self.inner;

        if search_left {
            if let Some(left) = self.left.as_ref() {
                result.extend(left.guess_strings(key));
            }
        }

        if search_right {
            if let Some(right) = self.right.as_ref() {
                result.extend(right.guess_strings(key));
            }
        }

        if search_down {
            if let Some(next) = self.next.as_ref() {
                if !key.is_empty() {
                    key.remove(0);
                }

                let mut basket = next.guess_strings(key);

                let basket = basket.drain(..).map(|(x, singular)| {
                    let mut y = vec![self.inner.clone()];
                    y.extend(x);
                    (y, singular)
                });

                result.extend(basket.collect::<Vec<(Vec<String>, Option<bool>)>>());
            } else {
                result.extend(vec![(vec![self.inner.clone()], self.singular)]);
            }
        }

        result
    }
}
