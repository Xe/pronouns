use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use axum_extra::routing::SpaRouter;
use maud::{html, Markup, Render, DOCTYPE};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Clone, Debug)]
struct PronounTrie {
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
        // TODO
        Vec::new()
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pronouns: Vec<PronounSet> = serde_dhall::from_file("./dhall/package.dhall").parse()?;

    let pron_trie = PronounTrie::build(pronouns);

    let files = SpaRouter::new("/static/css", env!("XESS_PATH"));

    let app = Router::new()
        .route("/api/all", get(all_pronouns_json))
        .route("/api/docs", get(api_docs))
        .route("/api/lookup/*pronoun", get(guess_pronouns_json))
        .route(
            "/api/exact/:nominative/:accusative/:determiner/:possessive/:reflexive",
            get(exact_pronouns_json),
        )
        .route("/pronoun-list", get(all_pronouns))
        .route("/", get(handler))
        .route("/*pronoun", get(guess_pronouns))
        .merge(files)
        .with_state(pron_trie);

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn all_pronouns_json(
    State(prons): State<PronounTrie>
) -> Json<Vec<PronounSet>> {
    let result = prons.guess(&mut Vec::new());

    Json(result)
}

async fn exact_pronouns_json(Path(ps): Path<PronounSet>) -> Json<PronounSet> {
    Json(ps)
}

#[derive(Serialize, Debug)]
pub struct Error {
    pub message: String,
}

async fn guess_pronouns_json(
    Path(pronoun): Path<String>,
    State(prons): State<PronounTrie>,
) -> Result<(StatusCode, Json<Vec<PronounSet>>), (StatusCode, Json<Error>)> {
    let mut key = pronoun.split("/").map(|x| x.to_owned()).collect();
    let guessed = prons.guess(&mut key);

    if !guessed.is_empty() {
        Ok((StatusCode::OK, Json(guessed.clone())))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(Error {
                message: format!("can't find {pronoun} in my database"),
            }),
        ))
    }
}

async fn guess_pronouns(
    Path(pronoun): Path<String>,
    State(prons): State<PronounTrie>,
) -> (StatusCode, Markup) {
    let mut key = pronoun.split("/").map(|x| x.to_owned()).collect();
    let guessed = prons.guess(&mut key);

    // If we have at least one allowed guess, let's just show the first. This means that
    // ambiguities are resolved in alphabetical order. (Note that the API will return all matches,
    // we want a fuzzier/friendlier interface on the web.)
    if let Some(v) = guessed.last() {
        let title = format!("{}/{}", v.nominative, v.accusative);
        return (
            StatusCode::OK,
            base(
                Some(&title),
                html! {
                    (v)
                },
            ),
        );
    }

    let sp = pronoun.split("/").collect::<Vec<&str>>();
    if sp.len() == 5 {
        let ps = PronounSet {
            nominative: sp[0].to_string(),
            accusative: sp[1].to_string(),
            determiner: sp[2].to_string(),
            possessive: sp[3].to_string(),
            reflexive: sp[4].to_string(),
        };

        let title = format!("{}/{}", ps.nominative, ps.accusative);
        return (
            StatusCode::OK,
            base(
                Some(&title),
                html! {
                    (ps)
                },
            ),
        );
    }

    (
        StatusCode::NOT_FOUND,
        base(
            Some(&format!("Can't find that pronoun")),
            html! {
                p {
                    "oopsie whoopsie uwu u did a fucky-wucky a widdle fucko boingo! This service doesn't have pronouns for "
                    (pronoun)
                    " on file. If this is a bug, please contact "
                    a href="https://pony.social/@cadey" { "@cadey@pony.social" }
                    " for help."
                }
            },
        ),
    )
}

async fn all_pronouns(State(prons): State<PronounTrie>) -> Markup {
    let pronouns = prons.guess(&mut Vec::new());
    let dsp = pronouns.iter()
        .map(|v| (format!("{}/{}", v.nominative, v.accusative), v.url()));

    base(
        Some("All pronouns"),
        html! {
            ul {
                @for (title, link) in dsp {
                    li {
                        a href=(link) {(title)}
                    }
                }
            }

            p {
                "If your pronouns are not listed here, you can construct a custom URL like this:"
                    br;br;
                code {
                    pre {
                        "https://pronouns.within.lgbt/subject/object/determiner/possessive/reflexive"
                    }
                }
                "If you want that set added to the website, please contact "
                a href="https://pony.social/@cadey" { "@cadey@pony.social" }
                " to get them added."
            }
        },
    )
}

async fn api_docs() -> Markup {
    base(
        Some("API Documentation"),
        html! {
            p {
                "This service offers API calls for looking up pronoun information. All URLs are offered as "
                a href="https://www.rfc-editor.org/rfc/rfc6570" { "RFC 6570" }
                " URL templates. Here are the calls offered by this service:"
            }

            h3 { "PronounSet type" }

            p {
                "The core datatype of the API is the PronounSet. It contains information on all the grammatical cases for each pronoun set. It always has five fields that are as follows:"
                dl {
                    dt { "nominative" }
                    dd { "The nominative case or subject form of a pronoun. This is the case that is used when the person or object being referred to is the subject of the sentence." }
                    dt { "accusative" }
                    dd { "The accusative case or object form of a pronoun. This is the case that is used when the person or object being referred to is the object of the sentence." }
                    dt { "dependent" }
                    dd { "The dependent possessive case. This is the case that is used when the person or object being referred to is the owner of an object or possesses it somehow. This is used as an adjective." }
                    dt { "possessive" }
                    dd { "The possessive case. This is the case that is used when the pronoun replaces a noun or a noun phrase." }
                    dt { "reflexive" }
                    dd { "The reflexive case. This is the case used when one is referring to themselves." }
                }
            }

            h3 { code { "/api/all" } }
            p {
                "This returns all information on all pronouns in the database in a list of PronounSet values."
            }

            h3 { code { "/api/lookup/{pronouns*}" } }
            p {
                "This attempts to figure out which pronoun you want and returns information about that PronounSet."
                br;br;
                "For example: "
                a href="/api/lookup/she/her" { "/api/lookup/she/her" }
                " will return the same information as "
                a href="/she/her" { "/she/her" }
                "."
            }

            h3 { code { "/api/exact/{nom}/{acc}/{det}/{pos}/{ref}" } }
            p {
                "This route will give you a PronounSet based on the exact set of pronouns that you give it."
                br;br;
                "For example: "
                a href="/api/exact/char/char/char/chars/charself" { "/api/exact/char/char/char/chars/charself" }
                " will return the same information as "
                a href="/char/char/char/chars/charself" { "/char/char/char/chars/charself" }
                "."
            }
        },
    )
}

async fn handler() -> Markup {
    base(
        None,
        html! {
            p {
                "Hello, this is a service that lets you demonstrate how various third-person pronouns are used. It will list all of the grammatical forms for each pronoun set."
            }

            a href="/pronoun-list" { "All the pronouns in the database" }
            br;
            a href="/api/docs" { "API Documentation" }

            p {
                "If your pronouns are not listed here, you can construct a custom URL like this:"
                br;br;
                code {
                    pre {
                        "https://pronouns.within.lgbt/subject/object/determiner/possessive/reflexive"
                    }
                }
                "This is a bit verbose, but it will work."
            }
        },
    )
}

fn base(title: Option<&str>, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                @if let Some(title) = title {
                    title { (format!("{title} - Pronouns")) }
                } @else {
                    title { "Pronouns" }
                }
                link rel="stylesheet" href="/static/css/xess.css";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
            }

            body #top {
                main {
                    nav {
                        a href="/" {"Pronouns"}
                        " - "
                        a href="/pronoun-list" {"All Pronouns"}
                        " - "
                        a href="/api/docs" {"API Documentation"}
                    }

                    @if let Some(title) = title {
                        h1 { (title) }
                    } @else {
                        h1 { "Pronouns" }
                    }

                    (body)

                    footer {
                        p {
                            "From "
                            a href="https://xeiaso.net" { "Within" }
                            "."
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub struct PronounSet {
    nominative: String,
    accusative: String,
    determiner: String,
    possessive: String,
    reflexive: String,
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
    fn url(&self) -> String {
        format!("/{}/{}/{}/{}/{}", self.nominative, self.accusative, self.determiner, self.possessive, self.reflexive)
    }
}
