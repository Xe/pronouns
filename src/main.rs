use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use axum_extra::routing::SpaRouter;
use maud::{html, Markup, Render, DOCTYPE};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pronouns: Vec<PronounSet> = serde_dhall::from_file("./dhall/package.dhall").parse()?;

    let mut pron_map: HashMap<String, PronounSet> = HashMap::new();

    for pronoun in pronouns {
        pron_map.insert(
            format!(
                "{}/{}/{}/{}/{}",
                pronoun.nominative,
                pronoun.accusative,
                pronoun.determiner,
                pronoun.possessive,
                pronoun.reflexive
            ),
            pronoun,
        );
    }

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
        .with_state(pron_map);

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn all_pronouns_json(
    State(prons): State<HashMap<String, PronounSet>>,
) -> Json<Vec<PronounSet>> {
    let mut result: Vec<PronounSet> = vec![];

    for (_, v) in &prons {
        result.push(v.clone())
    }

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
    State(prons): State<HashMap<String, PronounSet>>,
) -> Result<(StatusCode, Json<PronounSet>), (StatusCode, Json<Error>)> {
    if pronoun == "they/.../themselves" {
        if let Some(v) = prons.get("they/them/their/theirs/themselves") {
            return Ok((StatusCode::OK, Json(v.clone())));
        }
    }

    for (k, v) in &prons {
        if k.starts_with(&pronoun) {
            return Ok((StatusCode::OK, Json(v.clone())));
        }
    }

    Err((
        StatusCode::NOT_FOUND,
        Json(Error {
            message: format!("can't find {pronoun} in my database"),
        }),
    ))
}

async fn guess_pronouns(
    Path(pronoun): Path<String>,
    State(prons): State<HashMap<String, PronounSet>>,
) -> (StatusCode, Markup) {
    if pronoun == "they/.../themselves" {
        if let Some(v) = prons.get("they/them/their/theirs/themselves") {
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
    }

    for (k, v) in &prons {
        if k.starts_with(&pronoun) {
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

async fn all_pronouns(State(prons): State<HashMap<String, PronounSet>>) -> Markup {
    let mut pronouns: Vec<(String, String)> = Vec::new();

    for (k, v) in &prons {
        pronouns.push((
            format!("{}/{}", v.nominative, v.accusative),
            format!("/{k}"),
        ));
    }

    pronouns.sort();

    base(
        Some("All pronouns"),
        html! {
            ul {
                @for (title, link) in &pronouns {
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

#[derive(Clone, Deserialize, Serialize, Default)]
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
