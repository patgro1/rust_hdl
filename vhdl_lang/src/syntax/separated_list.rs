// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2023, Olof Kraigher olof.kraigher@gmail.com

use crate::ast::{IdentList, NameList, SeparatedList, WithRef};
use crate::data::{DiagnosticHandler, DiagnosticResult};
use crate::syntax::common::ParseResult;
use crate::syntax::names::parse_name;
use crate::syntax::Kind::{Comma};
use crate::syntax::{Kind, TokenAccess, TokenStream};
use std::fmt::Debug;

/// Parses a list of the form
///   `element { separator element }`
/// where `element` is an AST element and `separator` is a token of some `ast::Kind`.
/// The returned list retains information of the whereabouts of the separator tokens.
pub fn parse_list_with_separator<F, T: Debug>(
    stream: &TokenStream,
    separator: Kind,
    diagnostics: &mut dyn DiagnosticHandler,
    final_token: Kind,
    parse_fn: F,
) -> DiagnosticResult<SeparatedList<T>>
where
    F: Fn(&TokenStream) -> ParseResult<T>,
{
    let mut items = vec![parse_fn(stream)?];
    let mut tokens = Vec::new();
    while let Some(separator) = stream.pop_if_kind(separator) {
        tokens.push(separator);
        if stream.next_kind_is(final_token) {
            diagnostics.error(stream.get_pos(separator), "Trailing comma not allowed");
            break
        }
        items.push(parse_fn(stream)?);
    }
    Ok(SeparatedList { items, tokens })
}

pub fn parse_name_list(
    stream: &TokenStream,
    diagnostics: &mut dyn DiagnosticHandler,
    final_token: Kind
) -> DiagnosticResult<NameList> {
    parse_list_with_separator(
        stream,
        Comma,
        diagnostics,
        final_token,
        parse_name,
    )
}

pub fn parse_ident_list(
    stream: &TokenStream,
    diagnostics: &mut dyn DiagnosticHandler,
    final_token: Kind
) -> DiagnosticResult<IdentList> {
    parse_list_with_separator(
        stream,
        Comma,
        diagnostics,
        final_token,
        |stream| stream.expect_ident().map(WithRef::new),
    )
}

#[cfg(test)]
mod test {
    use crate::ast::{IdentList, NameList};
    use crate::syntax::separated_list::{parse_ident_list, parse_name_list};
    use crate::syntax::test::Code;
    use assert_matches::assert_matches;
    use crate::syntax::Kind::SemiColon;

    #[test]
    pub fn test_error_on_empty_list() {
        let code = Code::new("");
        let (res, diag) = code.with_partial_stream_diagnostics(|stream, diag| parse_ident_list(stream, diag, SemiColon));
        assert_matches!(res, Err(_))
    }

    #[test]
    pub fn parse_single_element_list() {
        let code = Code::new("abc");
        assert_eq!(
            code.parse_ok_no_diagnostics(|stream, diag| parse_ident_list(stream, diag, SemiColon)),
            IdentList::single(code.s1("abc").ident().into_ref())
        )
    }

    #[test]
    pub fn parse_list_with_multiple_elements() {
        let code = Code::new("abc, def, ghi");
        assert_eq!(
            code.parse_ok_no_diagnostics(|stream, diag| parse_ident_list(stream, diag, SemiColon)),
            IdentList {
                items: vec![
                    code.s1("abc").ident().into_ref(),
                    code.s1("def").ident().into_ref(),
                    code.s1("ghi").ident().into_ref()
                ],
                tokens: vec![code.s(",", 1).token(), code.s(",", 2).token()]
            }
        )
    }

    #[test]
    fn parse_list_with_many_names() {
        let code = Code::new("work.foo, lib.bar.all");
        assert_eq!(
            code.parse_ok_no_diagnostics(|stream, diag| parse_name_list(stream, diag, SemiColon)),
            NameList {
                items: vec![code.s1("work.foo").name(), code.s1("lib.bar.all").name()],
                tokens: vec![code.s1(",").token()],
            }
        )
    }
}
