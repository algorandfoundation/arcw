/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use arcw_lint::lints::preamble::Url;
use arcw_lint::reporters::Text;
use arcw_lint::Linter;

#[tokio::test]
async fn invalid() {
    let src = r#"---
header: example.com/foo?bar
---
hello world"#;

    let reports = Linter::<Text<String>>::default()
        .clear_lints()
        .add_lint("preamble-url", Url("header"))
        .check_slice(None, src)
        .run()
        .await
        .unwrap()
        .into_inner();

    assert_eq!(
        reports,
        r#"error[preamble-url]: preamble header `header` is not a valid URL
  |
2 | header: example.com/foo?bar
  |        ^^^^^^^^^^^^^^^^^^^^ relative URL without a base
  |
"#,
    );
}

#[tokio::test]
async fn valid() {
    let src = r#"---
header: https://example.com/foo?bar
---
hello world"#;

    let reports = Linter::<Text<String>>::default()
        .clear_lints()
        .add_lint("preamble-url", Url("header"))
        .check_slice(None, src)
        .run()
        .await
        .unwrap()
        .into_inner();

    assert_eq!(reports, "");
}
