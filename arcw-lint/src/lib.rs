/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod fetch;
pub mod lints;
pub mod preamble;
pub mod reporters;
pub mod tree;

use annotate_snippets::snippet::{Annotation, AnnotationType, Slice, Snippet};

use comrak::arena_tree::Node;
use comrak::nodes::Ast;
use comrak::{Arena, ComrakExtensionOptions, ComrakOptions};

use crate::lints::{Context, Error as LintError, FetchContext, InnerContext, Lint, LintExt as _};
use crate::preamble::Preamble;
use crate::reporters::Reporter;

use educe::Educe;

use snafu::{ensure, ResultExt, Snafu};

use std::cell::RefCell;
use std::collections::hash_map::{self, HashMap};
use std::path::{Path, PathBuf};

#[derive(Snafu, Debug)]
#[non_exhaustive]
pub enum Error {
    Lint {
        #[snafu(backtrace)]
        source: LintError,
        origin: Option<PathBuf>,
    },
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    SliceFetched {
        lint: String,
        origin: Option<PathBuf>,
    },
}

pub fn default_lints() -> impl Iterator<Item = (&'static str, Box<dyn Lint>)> {
    use lints::preamble::regex;
    use lints::{markdown, preamble};

    [
        //
        // Preamble
        //
        ("preamble-no-dup", preamble::NoDuplicates.boxed()),
        ("preamble-trim", preamble::Trim.boxed()),
        ("preamble-arc", preamble::Uint("arc").boxed()),
        ("preamble-author", preamble::Author("author").boxed()),
        ("preamble-re-title", preamble::Regex {
            name: "title",
            mode: regex::Mode::Excludes,
            pattern: r"(?i)standar\w*\b",
            message: "preamble header `title` should not contain `standard` (or similar words.)",
        }.boxed()),
        ("preamble-re-title-arc", preamble::Regex {
            name: "title",
            mode: regex::Mode::Excludes,
            pattern: r"(?i)ARC[\s-]*[0-9]+",
            message: "preamble header `title` should not contain `ARC-`",
        }.boxed()),
        (
            "preamble-re-title-arc-dash",
            preamble::Regex {
                name: "title",
                mode: regex::Mode::Excludes,
                pattern: r"(?i)ARC[\s]*[0-9]+",
                message: "preamble header `title` should not contain `ARC`",
            }.boxed(),
        ),
        (
            "preamble-re-description-arc-dash",
            preamble::Regex {
                name: "description",
                mode: regex::Mode::Excludes,
                pattern: r"(?i)ARC[\s]*[0-9]+",
                message: "proposals must be referenced with the form `ARC-N` (not `ARCN` or `ARC N`)",
            }.boxed(),
        ),
        ("preamble-re-description-arc", preamble::Regex {
            name: "description",
            mode: regex::Mode::Excludes,
            pattern: r"(?i)ARC[\s-]*[0-9]+",
            message: "proposals must be referenced with the form `ARC-N` (not `arc-N`)",
        }.boxed()),
        ("preamble-re-description", preamble::Regex {
            name: "description",
            mode: regex::Mode::Excludes,
            pattern: r"standar\w*\b",
            message: "preamble header `description` should not contain `standard` (or similar words.)",
        }.boxed()),
        (
            "preamble-discussions-to",
            preamble::Url("discussions-to").boxed(),
        ),
        (
            "preamble-re-discussions-to",
            preamble::Regex {
                name: "discussions-to",
                mode: regex::Mode::Includes,
                pattern: "^https://github.com/algorandfoundation/ARCs/issues/",
                message: concat!(
                    "preamble header `discussions-to` should ",
                    "point to a thread on algorandfoundation/ARCs/issues"
                ),
            }.boxed(),
        ),
        ("preamble-list-author", preamble::List("author").boxed()),
        ("preamble-list-extends", preamble::List("extends").boxed()),
        (
            "preamble-uint-extends",
            preamble::UintList("extends").boxed(),
        ),
        ("preamble-list-extended-by", preamble::List("extended-by").boxed()),
        (
            "preamble-uint-extended-by",
            preamble::UintList("extended-by").boxed(),
        ),
        ("preamble-list-replaces", preamble::List("replaces").boxed()),
        (
            "preamble-uint-replaces",
            preamble::UintList("replaces").boxed(),
        ),
        ("preamble-list-superseded-by", preamble::List("superseded-by").boxed()),
        (
            "preamble-uint-superseded-by",
            preamble::UintList("superseded-by").boxed(),
        ),
        ("preamble-list-requires", preamble::List("requires").boxed()),
        (
            "preamble-uint-requires",
            preamble::UintList("requires").boxed(),
        ),
        (
            "preamble-len-title",
            preamble::Length {
                name: "title",
                min: Some(2),
                max: Some(44),
            }
            .boxed(),
        ),
        (
            "preamble-len-description",
            preamble::Length {
                name: "description",
                min: Some(2),
                max: Some(140),
            }
            .boxed(),
        ),
        (
            "preamble-len-withdrawal-reason",
            preamble::Length {
                name: "withdrawal-reason",
                min: Some(2),
                max: Some(140),
            }
            .boxed(),
        ),
        (
            "preamble-req",
            preamble::Required(&[
                "arc",
                "title",
                "description",
                "author",
                "status",
                "type",
                "created",
            ])
            .boxed(),
        ),
        (
            "preamble-order",
            preamble::Order(&[
                "arc",
                "title",
                "description",
                "author",
                "discussions-to",
                "status",
                "last-call-deadline",
                "idle-since",
                "type",
                "category",
                "sub-category",
                "created",
                "requires",
                "withdrawal-reason",
                "extended-by",
                "extends",
                "replaces",
                "superseded-by",
            ])
            .boxed(),
        ),
        (
            "preamble-enum-sub-category",
            preamble::OneOf {
                name: "sub-category",
                values: &[
                    "General",
                    "Asa",
                    "Application",
                    "Explorer",
                    "Wallet"
                ],
            }
            .boxed(),
        ),
        ("preamble-date-created", preamble::Date("created").boxed()),
        (
            "preamble-req-last-call-deadline",
            preamble::RequiredIfEq {
                when: "status",
                equals: "Last Call",
                then: "last-call-deadline",
            }
            .boxed(),
        ),
        (
            "preamble-date-last-call-deadline",
            preamble::Date("last-call-deadline").boxed(),
        ),
        (
            "preamble-req-idle-since",
            preamble::RequiredIfEq {
                when: "status",
                equals: "Idle",
                then: "idle-since",
            }
            .boxed(),
        ),
        (
            "preamble-date-dle-since",
            preamble::Date("idle-since").boxed(),
        ),
        (
            "preamble-req-category",
            preamble::RequiredIfEq {
                when: "type",
                equals: "Standards Track",
                then: "category",
            }
            .boxed(),
        ),
        (
            "preamble-req-withdrawal-reason",
            preamble::RequiredIfEq {
                when: "status",
                equals: "Withdrawn",
                then: "withdrawal-reason",
            }
            .boxed(),
        ),
        (
            "preamble-enum-status",
            preamble::OneOf {
                name: "status",
                values: &[
                    "Draft",
                    "Review",
                    "Last Call",
                    "Idle",
                    "Final",
                    "Stagnant",
                    "Withdrawn",
                    "Deprecated",
                    "Living",
                ],
            }
            .boxed(),
        ),
        (
            "preamble-enum-type",
            preamble::OneOf {
                name: "type",
                values: &["Standards Track", "Meta", "Informational"],
            }
            .boxed(),
        ),
        (
            "preamble-enum-category",
            preamble::OneOf {
                name: "category",
                values: &["Core", "Networking", "Interface", "ARC"],
            }
            .boxed(),
        ),
        (
            "preamble-requires-status",
            preamble::RequiresStatus {
                requires: "requires",
                status: "status",
                flow: &[
                    &["Draft", "Stagnant"],
                    &["Review"],
                    &["Last Call"],
                    &["Final", "Withdrawn", "Living","Deprecated", "Idle"],
                ]
            }.boxed(),
        ),
        (
            "preamble-requires-ref-title",
            preamble::RequireReferenced {
                name: "title",
                requires: "requires",
            }.boxed(),
        ),
        (
            "preamble-requires-ref-description",
            preamble::RequireReferenced {
                name: "description",
                requires: "requires",
            }.boxed(),
        ),
        (
            "preamble-file-name",
            preamble::FileName {
                name: "arc",
                prefix: "arc-",
                suffix: ".md",
            }.boxed(),
        ),
        //
        // Markdown
        //
        (
            "markdown-req-section",
            markdown::SectionRequired(&[
                "Abstract",
                "Specification",
                "Rationale",
                "Security Considerations",
                "Copyright",
            ])
            .boxed(),
        ),
        (
            "markdown-order-section",
            markdown::SectionOrder(&[
                "Abstract",
                "Motivation",
                "Specification",
                "Rationale",
                "Backwards Compatibility",
                "Test Cases",
                "Reference Implementation",
                "Security Considerations",
                "Copyright",
            ])
            .boxed(),
        ),
        (
            "markdown-re-arc-not-arc",
            markdown::Regex {
                mode: markdown::regex::Mode::Excludes,
                pattern: r"arc[\s-]*[0-9]+",
                message: "proposals must be referenced with the form `ARC-N` (not `arc-N`)",
            }.boxed(),
        ),
        (
            "markdown-re-arc-dash",
            markdown::Regex {
                mode: markdown::regex::Mode::Excludes,
                pattern: r"(?i)ARC[\s]*[0-9]+",
                message: "proposals must be referenced with the form `ARC-N` (not `ARCN` or `ARC N`)",
            }.boxed(),
        ),
        (
            "markdown-link-first",
            markdown::LinkFirst(r"(?i)ARC-[0-9]+").boxed(),
        ),
        ("markdown-rel-links", markdown::RelativeLinks.boxed()),
        (
            "markdown-link-status",
            markdown::LinkStatus {
                status: "status",
                flow: &[
                    &["Draft", "Stagnant"],
                    &["Review"],
                    &["Last Call"],
                    &["Final", "Withdrawn", "Living","Deprecated", "Idle"],
                ]
            }.boxed(),
        )
    ]
    .into_iter()
}

#[derive(Debug)]
enum Source<'a> {
    String {
        origin: Option<&'a str>,
        src: &'a str,
    },
    File(&'a Path),
}

impl<'a> Source<'a> {
    fn origin(&self) -> Option<&Path> {
        match self {
            Self::String {
                origin: Some(s), ..
            } => Some(Path::new(s)),
            Self::File(p) => Some(p),
            _ => None,
        }
    }

    fn is_string(&self) -> bool {
        matches!(self, Self::String { .. })
    }

    async fn fetch(&self, fetch: &dyn fetch::Fetch) -> Result<String, Error> {
        match self {
            Self::File(f) => fetch
                .fetch(f.to_path_buf())
                .await
                .with_context(|_| IoSnafu { path: f.to_owned() })
                .map_err(Into::into),
            Self::String { src, .. } => Ok((*src).to_owned()),
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
#[must_use]
pub struct Linter<'a, R> {
    lints: HashMap<&'a str, Box<dyn Lint>>,
    sources: Vec<Source<'a>>,

    #[educe(Debug(ignore))]
    reporter: R,

    #[educe(Debug(ignore))]
    fetch: Box<dyn fetch::Fetch>,
}

impl<'a, R> Default for Linter<'a, R>
where
    R: Default,
{
    fn default() -> Self {
        Self::new(R::default())
    }
}

impl<'a, R> Linter<'a, R> {
    pub fn new(reporter: R) -> Self {
        Self {
            reporter,
            sources: Default::default(),
            lints: default_lints().collect(),
            fetch: Box::new(fetch::DefaultFetch::default()),
        }
    }

    pub fn add_lint<T>(mut self, slug: &'a str, lint: T) -> Self
    where
        T: 'static + Lint,
    {
        if self.lints.insert(slug, lint.boxed()).is_some() {
            panic!("duplicate slug: {}", slug);
        }

        self
    }

    pub fn remove_lint(mut self, slug: &str) -> Self {
        if self.lints.remove(slug).is_none() {
            panic!("no lint with the slug: {}", slug);
        }

        self
    }

    pub fn clear_lints(mut self) -> Self {
        self.lints.clear();
        self
    }

    pub fn set_fetch<F>(mut self, fetch: F) -> Self
    where
        F: 'static + fetch::Fetch,
    {
        self.fetch = Box::new(fetch);
        self
    }
}

impl<'a, R> Linter<'a, R>
where
    R: Reporter,
{
    pub fn check_slice(mut self, origin: Option<&'a str>, src: &'a str) -> Self {
        self.sources.push(Source::String { origin, src });
        self
    }

    pub fn check_file(mut self, path: &'a Path) -> Self {
        self.sources.push(Source::File(path));
        self
    }

    pub async fn run(self) -> Result<R, Error> {
        if self.lints.is_empty() {
            panic!("no lints activated");
        }

        if self.sources.is_empty() {
            panic!("no sources given");
        }

        let mut to_check = Vec::with_capacity(self.sources.len());
        let mut fetched_arcs = HashMap::new();

        for source in self.sources {
            let source_origin = source.origin().map(Path::to_path_buf);
            let source_content = source.fetch(&*self.fetch).await?;

            to_check.push((source_origin, source_content));

            let (source_origin, source_content) = to_check.last().unwrap();
            let display_origin = source_origin.as_deref().map(Path::to_string_lossy);
            let display_origin = display_origin.as_deref();

            let arena = Arena::new();
            let inner = match process(&reporters::Null, &arena, display_origin, source_content)? {
                Some(i) => i,
                None => continue,
            };

            for (slug, lint) in &self.lints {
                let context = FetchContext {
                    body: inner.body,
                    preamble: &inner.preamble,
                    arcs: Default::default(),
                };

                lint.find_resources(&context).with_context(|_| LintSnafu {
                    origin: source_origin.clone(),
                })?;

                let arcs = context.arcs.into_inner();

                // For now, string sources shouldn't be allowed to fetch external
                // resources. The origin field isn't guaranteed to be a file/URL,
                // and even if it was, we wouldn't know which of those to interpret
                // it as.
                ensure!(
                    arcs.is_empty() || !source.is_string(),
                    SliceFetchedSnafu {
                        lint: *slug,
                        origin: source_origin.clone(),
                    }
                );

                for arc in arcs.into_iter() {
                    let root = match source {
                        Source::File(p) => p.parent().unwrap_or_else(|| Path::new(".")),
                        _ => unreachable!(),
                    };

                    let path = root.join(arc);

                    let entry = match fetched_arcs.entry(path) {
                        hash_map::Entry::Occupied(_) => continue,
                        hash_map::Entry::Vacant(v) => v,
                    };

                    let content = Source::File(entry.key()).fetch(&*self.fetch).await;
                    entry.insert(content);
                }
            }
        }

        let resources_arena = Arena::new();
        let mut parsed_arcs = HashMap::new();

        for (origin, result) in &fetched_arcs {
            let source = match result {
                Ok(o) => o,
                Err(e) => {
                    parsed_arcs.insert(origin.as_path(), Err(e));
                    continue;
                }
            };

            let inner = match process(&self.reporter, &resources_arena, None, source)? {
                Some(s) => s,
                None => return Ok(self.reporter),
            };
            parsed_arcs.insert(origin.as_path(), Ok(inner));
        }

        let mut lints: Vec<_> = self.lints.iter().collect();
        lints.sort_by_key(|l| l.0);

        for (origin, source) in &to_check {
            let display_origin = origin.as_ref().map(|p| p.to_string_lossy().into_owned());
            let display_origin = display_origin.as_deref();

            let arena = Arena::new();
            let inner = match process(&self.reporter, &arena, display_origin, source)? {
                Some(i) => i,
                None => continue,
            };

            let context = Context {
                inner,
                reporter: &self.reporter,
                arcs: &parsed_arcs,
            };

            for (slug, lint) in &lints {
                lint.lint(slug, &context).with_context(|_| LintSnafu {
                    origin: origin.clone(),
                })?;
            }
        }

        Ok(self.reporter)
    }
}

fn process<'r, 'a>(
    reporter: &'r dyn Reporter,
    arena: &'a Arena<Node<'a, RefCell<Ast>>>,
    origin: Option<&'a str>,
    source: &'a str,
) -> Result<Option<InnerContext<'a>>, Error> {
    let (preamble_source, body_source) = match Preamble::split(source) {
        Ok(v) => v,
        Err(preamble::SplitError::MissingStart { .. })
        | Err(preamble::SplitError::LeadingGarbage { .. }) => {
            let mut footer = Vec::new();
            if source.as_bytes().get(3) == Some(&b'\r') {
                footer.push(Annotation {
                    id: None,
                    label: Some(
                        "found a carriage return (CR), use Unix-style line endings (LF) instead",
                    ),
                    annotation_type: AnnotationType::Help,
                });
            }
            reporter
                .report(Snippet {
                    title: Some(Annotation {
                        id: None,
                        label: Some("first line must be `---` exactly"),
                        annotation_type: AnnotationType::Error,
                    }),
                    slices: vec![Slice {
                        fold: false,
                        line_start: 1,
                        origin,
                        source: source.lines().next().unwrap_or_default(),
                        annotations: vec![],
                    }],
                    footer,
                    ..Default::default()
                })
                .map_err(LintError::from)
                .with_context(|_| LintSnafu {
                    origin: origin.map(PathBuf::from),
                })?;
            return Ok(None);
        }
        Err(preamble::SplitError::MissingEnd { .. }) => {
            reporter
                .report(Snippet {
                    title: Some(Annotation {
                        id: None,
                        label: Some("preamble must be followed by a line containing `---` exactly"),
                        annotation_type: AnnotationType::Error,
                    }),
                    ..Default::default()
                })
                .map_err(LintError::from)
                .with_context(|_| LintSnafu {
                    origin: origin.map(PathBuf::from),
                })?;
            return Ok(None);
        }
    };

    let preamble = match Preamble::parse(origin, preamble_source) {
        Ok(p) => p,
        Err(e) => {
            for snippet in e.into_errors() {
                reporter
                    .report(snippet)
                    .map_err(LintError::from)
                    .with_context(|_| LintSnafu {
                        origin: origin.map(PathBuf::from),
                    })?;
            }
            Preamble::default()
        }
    };

    let options = ComrakOptions {
        extension: ComrakExtensionOptions {
            table: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut preamble_lines: u32 = preamble_source.matches('\n').count().try_into().unwrap();
    preamble_lines += 3;

    let body = comrak::parse_document(arena, body_source, &options);

    for node in body.descendants() {
        let mut data = node.data.borrow_mut();
        if data.start_line == 0 {
            if let Some(parent) = node.parent() {
                // XXX: This doesn't actually work.
                data.start_line = parent.data.borrow().start_line;
            }
        } else {
            data.start_line += preamble_lines;
        }
    }

    Ok(Some(InnerContext {
        body,
        source,
        body_source,
        preamble,
        origin,
    }))
}
