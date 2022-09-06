/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use arcw_lint::lints::preamble::Uint;
use arcw_lint::reporters::Text;
use arcw_lint::Linter;

#[tokio::test]
async fn valid() {
    let src = r#"---
header: value0
other-header: value
header: value1
foo: bar
arc: 1234
---
hello world"#;

    let reports = Linter::<Text<String>>::default()
        .clear_lints()
        .add_lint("preamble-arc", Uint("arc"))
        .check_slice(None, src)
        .run()
        .await
        .unwrap()
        .into_inner();

    assert_eq!(reports, "");
}

#[tokio::test]
async fn invalid() {
    let src = r#"---
header: value0
other-header: value
header: value1
foo: bar
arc: -1234
---
hello world"#;

    let reports = Linter::<Text<String>>::default()
        .clear_lints()
        .add_lint("preamble-arc", Uint("arc"))
        .check_slice(None, src)
        .run()
        .await
        .unwrap()
        .into_inner();

    assert_eq!(
        reports,
        r#"error[preamble-arc]: preamble header `arc` must be an unsigned integer
  |
6 | arc: -1234
  |     ^^^^^^ not a non-negative integer
  |
"#
    );
}
