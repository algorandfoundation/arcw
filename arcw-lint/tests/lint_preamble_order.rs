/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use arcw_lint::lints::preamble::Order;
use arcw_lint::reporters::Text;
use arcw_lint::Linter;

#[tokio::test]
async fn one_extra() {
    let src = r#"---
header: value1
---
hello world"#;

    let reports = Linter::<Text<String>>::default()
        .clear_lints()
        .add_lint("preamble-order", Order(&["a1", "b2"]))
        .check_slice(None, src)
        .run()
        .await
        .unwrap()
        .into_inner();

    assert_eq!(
        reports,
        r#"error[preamble-order]: preamble has extra header(s)
  |
2 | header: value1
  | ^^^^^^ unrecognized header
  |
"#
    );
}

#[tokio::test]
async fn two_extra() {
    let src = r#"---
header: value1
b2: hiya
header2: value1
---
hello world"#;

    let reports = Linter::<Text<String>>::default()
        .clear_lints()
        .add_lint("preamble-order", Order(&["a1", "b2"]))
        .check_slice(None, src)
        .run()
        .await
        .unwrap()
        .into_inner();

    assert_eq!(
        reports,
        r#"error[preamble-order]: preamble has extra header(s)
  |
2 | header: value1
  | ^^^^^^ unrecognized header
  |
4 | header2: value1
  | ^^^^^^^ unrecognized header
  |
"#
    );
}

#[tokio::test]
async fn out_of_order() {
    let src = r#"---
b2: hiya
a1: foo
---
hello world"#;

    let reports = Linter::<Text<String>>::default()
        .clear_lints()
        .add_lint("preamble-order", Order(&["a1", "b2"]))
        .check_slice(None, src)
        .run()
        .await
        .unwrap()
        .into_inner();

    assert_eq!(
        reports,
        r#"error[preamble-order]: preamble header `b2` is out of order
  |
2 | b2: hiya
  |
  = help: `b2` should come after `a1`
"#
    );
}

#[tokio::test]
async fn out_of_order_with_optional() {
    let src = r#"---
b2: hiya
a1: foo
---
hello world"#;

    let reports = Linter::<Text<String>>::default()
        .clear_lints()
        .add_lint("preamble-order", Order(&["a1", "a2", "b2"]))
        .check_slice(None, src)
        .run()
        .await
        .unwrap()
        .into_inner();

    assert_eq!(
        reports,
        r#"error[preamble-order]: preamble header `b2` is out of order
  |
2 | b2: hiya
  |
  = help: `b2` should come after `a1`
"#
    );
}
