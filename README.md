arcw
====

An engine which ensures [ARC](https://github.com/algorandfoundation/ARCs) meet certain requirements.

## Getting Started

To install `arcw` and validate the ARCs repository:

```console
git clone git@github.com:algorandfoundation/arcw.git
cargo install --path=arcw arcw
arcw /path/to/ARCs
```

```
USAGE:
    arcw [OPTIONS] [SOURCES]...

ARGS:
    <SOURCES>...    Files and/or directories to check

OPTIONS:
        --format <FORMAT>     Output format [default: text] [possible values: text, json]
    -h, --help                Print help information
        --lints <LINTS>       Additional lints to enable
        --list-lints          List all available lints
        --no-default-lints    Do not enable the default lints
```



## Demo

### Example arc

```markdown
---
arc: 2
description: A really short example of an arc.
title: Sample of an arc
author: Stéphane Barroso(@sudoweezy)
discussions-to: https://example.com/
status: Living
type: Meta
created: 2022-06-30
---

## Specification

Implementers of this arc must...

## Abstract

This is an abstract!
```

### Output

```
error[markdown-order-section]: section `Specification` must come after `Motivation`
  --> /tmp/demo.md
   |
12 | ## Specification
   |
error[preamble-order]: preamble header `description` must come after `title`
 --> /tmp/demo.md
  |
3 | description: A really short example of an arc.
  |
```

## Lints

| id                                  | Description                                                                                   |
|-------------------------------------|-----------------------------------------------------------------------------------------------|
| `markdown-link-first`               | First mention of an arc must be a link.                                                       |
| `markdown-link-status`              | arcs linked in the body have statuses further along than the current proposal.                |
| `markdown-order-section`            | There are no extra sections and the sections are in the correct order.                        |
| `markdown-re-arc-dash`              | Other arcs are referenced using ARC-X, not ARCX or ARC X.                                     |
| `markdown-re-arc-not-arc`           | Other arcs are referenced using ARC-X, not arc-X.                                             |
| `markdown-rel-links`                | All URLs in the page are relative.                                                            |
| `markdown-req-section`              | Required sections are present in the body of the proposal.                                    |
| `preamble-author`                   | The author header is correctly formatted, and there is at least one GitHub user listed.       |
| `preamble-date-created`             | The `created` header is a date.                                                               |
| `preamble-date-last-call-deadline`  | The `last-call-deadline` header is a date.                                                    |
| `preamble-discussions-to`           | The `discussions-to` header is a valid URL.                                                   |
| `preamble-arc`                      | The `arc` header is a non-negative integer.                                                   |
| `preamble-enum-category`            | The `category` header is a recognized value.                                                  |
| `preamble-enum-status`              | The `status` header is a recognized value.                                                    |
| `preamble-enum-type`                | The `type` header is a recognized value.                                                      |
| `preamble-file-name`                | The file name reflects the arc number.                                                        |
| `preamble-len-description`          | The `description` header isn't too long.                                                      |
| `preamble-len-title`                | The `title` header isn't too long.                                                            |
| `preamble-list-author`              | The `author` header is a correctly formatted comma-separated list.                            |
| `preamble-list-requires`            | The `requires` header is a correctly formatted comma-separated list.                          |
| `preamble-no-dup`                   | There are no duplicate headers.                                                               |
| `preamble-order`                    | The preamble headers are in the correct order.                                                |
| `preamble-re-description`           | The description doesn't contain "standard" or similar words.                                  |
| `preamble-re-description-arc-dash`  | arcs referenced in the `description` header use a dash.                                       |
| `preamble-re-description-arc`       | ARCs referenced in the `description` header use the `ARC-X` format.                           |
| `preamble-re-discussions-to`        | The `discussions-to` header points to algorandfoundation/ARCs/issues                                     |
| `preamble-re-title`                 | The title doesn't contain "standard" or similar words.                                        |
| `preamble-re-title-arc-dash`        | arcs referenced in the `title` header use a dash.                                             |
| `preamble-re-title-arc`             | ARCs referenced in the `title` header use the `ARC-X` format.                                 |
| `preamble-req`                      | All required preamble headers are present.                                                    |
| `preamble-req-category`             | The `category` header is present only when required.                                          |
| `preamble-req-last-call-deadline`   | The `last-call-deadline` header is present only when required.                                |
| `preamble-req-withdrawal-reason`    | The `withdrawal-reason` header is present only when required.                                 |
| `preamble-requires-ref-description` | Proposals mentioned in the `description` header appear in the `requires` header.              |
| `preamble-requires-ref-title`       | Proposals mentioned in the `title` header appear in the `requires` header.                    |
| `preamble-requires-status`          | arcs listed in `requires` have statuses further along than the current proposal.              |
| `preamble-trim`                     | There is no extra whitespace around preamble fields.                                          |
| `preamble-uint-requires`            | The `requires` header is a sorted list of non-negative integers.                              |

## JavaScript / WebAssembly

`arcw-lint-js` packages `arcw` as an npm package, for use in JavaScript / TypeScript.

You can find the [package on npm](https://www.npmjs.com/package/arcw-lint-js).

### Building & Publishing

```bash
cd arcw-lint-js
wasm-pack test --node
wasm-pack build -t nodejs
wasm-pack publish -t nodejs
```

This tool is an adaptation of (EIPW)[https://github.com/ethereum/eipw]
