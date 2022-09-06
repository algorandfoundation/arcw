/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use arcw_lint::lints::preamble::RequireReferenced;
use arcw_lint::reporters::Text;
use arcw_lint::Linter;

#[tokio::test]
async fn valid() {
    let src = r#"---
header: Extension of ARC-44
other: 1234, 44, 55
---
hello world"#;

    let reports = Linter::<Text<String>>::default()
        .clear_lints()
        .add_lint(
            "preamble-req-ref",
            RequireReferenced {
                name: "header",
                requires: "other",
            },
        )
        .check_slice(None, src)
        .run()
        .await
        .unwrap()
        .into_inner();

    assert_eq!(reports, "");
}

#[tokio::test]
async fn one_missing() {
    let src = r#"---
header: Extension of ARC-9999 and ARC-44
other: 1234, 44, 55
---
hello world"#;

    let reports = Linter::<Text<String>>::default()
        .clear_lints()
        .add_lint(
            "preamble-req-ref",
            RequireReferenced {
                name: "header",
                requires: "other",
            },
        )
        .check_slice(None, src)
        .run()
        .await
        .unwrap()
        .into_inner();

    assert_eq!(
        reports,
        r#"error[preamble-req-ref]: proposals mentioned in preamble header `header` must appear in `other`
  |
2 | header: Extension of ARC-9999 and ARC-44
  |                      ^^^^^^^^ mentioned here
  |
"#
    );
}

#[tokio::test]
async fn two_missing() {
    let src = r#"---
header: Extension of ARC-9999 and ARC-45
other: 1234, 44, 55
---
hello world"#;

    let reports = Linter::<Text<String>>::default()
        .clear_lints()
        .add_lint(
            "preamble-req-ref",
            RequireReferenced {
                name: "header",
                requires: "other",
            },
        )
        .check_slice(None, src)
        .run()
        .await
        .unwrap()
        .into_inner();

    assert_eq!(
        reports,
        r#"error[preamble-req-ref]: proposals mentioned in preamble header `header` must appear in `other`
  |
2 | header: Extension of ARC-9999 and ARC-45
  |                      ^^^^^^^^ mentioned here
  |                                   ^^^^^^ mentioned here
  |
"#
    );
}
