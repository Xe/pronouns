use super::PronounSet;

/// An intermediate pronoun is either complete and stores an ordered Vec of nominative,
/// accusative, &c strings, and a boolean plurality indicator. If it is incomplete, the plurality
/// indicator is None and Vec<String> has fewer than 5 elements.
type IntPron = (Vec<String>, Option<bool>);

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

            Self::insert(&mut base, key, pronoun.singular);
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
                singular:   singular.unwrap(),
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

    fn insert(s: &mut Option<Box<Self>>, mut key: Vec<String>, singular: bool) {
        match s {
            None => {
                let car = key[0].clone();
                key.remove(0);
                let cons = key;

                let mut child = Self::new(car);

                if !cons.is_empty() {
                    Self::insert(&mut child.next, cons, singular);
                } else {
                    child.singular = Some(singular);
                }

                s.replace(Box::new(child));
            },
            Some(s) => {
                let car = &key[0];

                let s = if car < &s.inner {
                    &mut s.left
                } else if car > &s.inner {
                    &mut s.right
                } else {
                    // We found where to insert, advance the key.
                    key.remove(0);
                    &mut s.next
                };

                Self::insert(s, key, singular);
            },
        };
    }

    fn guess_strings(&self, key: &mut Vec<Option<String>>) -> Vec<IntPron> {
        let car = key.get(0).and_then(|x| x.as_ref());

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

        if search_right {
            if let Some(right) = self.right.as_ref() {
                result.extend(right.guess_strings(key));
            }
        }

        result
    }
}
