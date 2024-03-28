use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use axum_extra::routing::SpaRouter;
use maud::{html, Markup, DOCTYPE};
use serde::Serialize;
use std::{env, net::SocketAddr, sync::Mutex, sync::Arc};

use pronouns::{PronounSet, PronounTrie};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pronouns: Vec<PronounSet> = serde_dhall::from_file("./dhall/package.dhall").parse()?;

    let pron_trie = PronounTrie::build(pronouns);

    let files = SpaRouter::new("/static/css", env!("XESS_PATH"));

    let app = Router::new()
        .route("/.within/health", get(health))
        .route("/api/all", get(all_pronouns_json))
        .route("/api/docs", get(api_docs))
        .route("/api/lookup/*pronoun", get(guess_pronouns_json))
        .route(
            "/api/exact/:nominative/:accusative/:determiner/:possessive/:reflexive",
            get(exact_pronouns_json),
        )
        .route("/pronoun-list", get(all_pronouns))
        .route("/", get(handler))
        .route("/they", get(they))
        .route("/*pronoun", get(guess_pronouns))
        .merge(files)
        .with_state(Arc::new(pron_trie));

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    let server = axum::Server::bind(&addr).serve(app.into_make_service());

    // Prepare some signal for when the server should start shutting down...
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let graceful = server.with_graceful_shutdown(async {
        rx.await.ok();
    });

    let tx = Mutex::new(Some(tx));

    // Await the `server` receiving the signal...
    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}

async fn health() -> String {
    "OK".into()
}

// unused for now; need to figure out how to call this in a Maud template
async fn get_domain() -> String {
    match env::var("PRONOUNS_DOMAIN") {
        Ok(val) => val,
        Err(_e) => "pronouns.within.lgbt".to_string(),
    }
}

async fn all_pronouns_json(
    State(prons): State<Arc<PronounTrie>>
) -> Json<Vec<PronounSet>> {
    Json(prons.gather())
}

async fn exact_pronouns_json(Path(ps): Path<PronounSet>) -> Json<PronounSet> {
    let mut ps = ps.clone();
    ps.singular = true;
    Json(ps)
}

#[derive(Serialize, Debug)]
pub struct Error {
    pub message: String,
}

async fn guess_pronouns_json(
    Path(pronoun): Path<String>,
    State(prons): State<Arc<PronounTrie>>,
) -> Result<(StatusCode, Json<Vec<PronounSet>>), (StatusCode, Json<Error>)> {
    let mut key = url_to_trie_query(pronoun.clone());
    let guessed = prons.guess(&mut key);

    if !guessed.is_empty() {
        Ok((StatusCode::OK, Json(guessed)))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(Error {
                message: format!("can't find {pronoun} in my database"),
            }),
        ))
    }
}

async fn they(prons: State<Arc<PronounTrie>>) -> (StatusCode, Markup) {
    guess_pronouns(Path("they/.../themselves".to_string()), prons).await
}

async fn guess_pronouns(
    Path(pronoun): Path<String>,
    State(prons): State<Arc<PronounTrie>>,
) -> (StatusCode, Markup) {
    let mut key = url_to_trie_query(pronoun.clone());
    let guessed = prons.guess(&mut key);

    if guessed.len() > 1 {
        return (
            StatusCode::BAD_REQUEST,
            base(
                Some("Ambiguous pronouns detected"),
                html! {
                    p {
                        "The pronoun you are looking up ("
                        (pronoun)
                        ") has multiple hits in the database. Please try one of the following options:"
                    }

                    ul {
                        @for hit in guessed {
                            li { a href=(hit.url()) {(hit.title())} }
                        }
                    }
                },
            ),
        );
    }

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

    let sp = pronoun.split('/').collect::<Vec<&str>>();
    if sp.len() == 5 {
        let ps = PronounSet {
            nominative: sp[0].to_string(),
            accusative: sp[1].to_string(),
            determiner: sp[2].to_string(),
            possessive: sp[3].to_string(),
            reflexive: sp[4].to_string(),
            singular: !sp[4].ends_with('s'),
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
            Some("Can't find that pronoun"),
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

async fn all_pronouns(State(prons): State<Arc<PronounTrie>>) -> Markup {
    let pronouns = prons.gather();
    let dsp = pronouns.iter().map(|v| (v.title(), v.url()));

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
                        "https://" ({
                            match env::var("PRONOUN_DOMAIN") {
                                Ok(val) => val,
                                Err(_e) => "pronouns.within.lgbt".to_string(),
                            }
                        }) "/subject/object/determiner/possessive/reflexive"
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
                " URL templates. All results will return JSON-formatted values. Here are the calls offered by this service:"
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
                    dt { "singular" }
                    dd { "This is true if the pronoun should be used in a singular way. This is false if it should be used in a plural way." }
                }
                "PronounSet responses are only returned when the HTTP status is 200."
            }
            h4 { "Example" }
            pre {
                code {
                    "{\n  \"nominative\": \"she\",\n  \"accusative\": \"her\",\n  \"determiner\": \"her\",\n  \"possessive\": \"hers\",\n  \"reflexive\": \"herself\",\n  \"singular\": true\n}"
                }
            }

            h3 { "Error type" }
            p {
                "Sometimes the service may return an error if it can't find what you're asking it. This error type will only contain a field named "
                code { "message" }
                " that contains a human-readable message to explain the failure. This will accompany a non-200 response."
            }
            h4 { "Example" }
            pre {
                code {
                    "{\n  \"message\": \"can't find she/his in my database\"\n}"
                }
            }

            h3 { code { "/api/all" } }
            p {
                "This returns all information on all pronouns in the database in a list of PronounSet values."
            }
            h4 { "Example" }
            pre {
                code {
                    "curl https://" ({
                        match env::var("PRONOUN_DOMAIN") {
                            Ok(val) => val,
                            Err(_e) => "pronouns.within.lgbt".to_string(),
                        }
                    }) "/api/all"
                }
            }

            h3 { code { "/api/lookup/{pronouns*}" } }
            p {
                "This attempts to figure out which pronoun you want and returns information about each PronounSet matching that description. It returns a list of PronounSet's."
                br;br;
                "For example: "
                a href="/api/lookup/she/her" { "/api/lookup/she/her" }
                " will return the same information as "
                a href="/she/her" { "/she/her" }
                "."
            }
            h4 { "Example" }
            pre {
                code {
                    "curl https://" ({
                        match env::var("PRONOUN_DOMAIN") {
                            Ok(val) => val,
                            Err(_e) => "pronouns.within.lgbt".to_string(),
                        }
                    }) "/api/lookup/she"
                    "\n[\n  {\n    \"nominative\": \"she\",\n    \"accusative\": \"her\",\n    \"determiner\": \"her\",\n    \"possessive\": \"hers\",\n    \"reflexive\": \"herself\",\n    \"singular\": true\n  }\n]"
                }
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
            h4 { "Example" }
            pre {
                code {
                    "curl https://" ({
                        match env::var("PRONOUN_DOMAIN") {
                            Ok(val) => val,
                            Err(_e) => "pronouns.within.lgbt".to_string(),
                        }
                    }) "/api/exact/char/char/char/chars/charself"
                    "\n{\n  \"nominative\": \"char\",\n  \"accusative\": \"char\",\n  \"determiner\": \"char\",\n  \"possessive\": \"chars\",\n  \"reflexive\": \"charself\",\n  \"singular\": true\n}"
                }
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
                        "https://" ({
                            match env::var("PRONOUN_DOMAIN") {
                                Ok(val) => val,
                                Err(_e) => "pronouns.within.lgbt".to_string(),
                            }
                        }) "/subject/object/determiner/possessive/reflexive"
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

fn url_to_trie_query(url: String) -> Vec<Option<String>> {
    url.split('/')
        .map(|x| match x {
            "..." | "" => None,
            x => Some(x.to_owned()),
        })
        .collect()
}
