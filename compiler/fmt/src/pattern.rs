use crate::annotation::{Formattable, Parens};
use crate::expr::is_multiline_pattern;
use crate::spaces::{fmt_comments_only, fmt_spaces, is_comment};
use bumpalo::collections::String;
use roc_parse::ast::{Base, Pattern};

pub fn fmt_pattern<'a>(
    buf: &mut String<'a>,
    pattern: &'a Pattern<'a>,
    indent: u16,
    parens: Parens,
    _only_comments: bool,
) {
    pattern.format_with_parens(buf, parens, indent);
}

impl<'a> Formattable<'a> for Pattern<'a> {
    fn is_multiline(&self) -> bool {
        // Theory: a pattern should only be multiline when it contains a comment
        match self {
            Pattern::SpaceBefore(_, spaces) | Pattern::SpaceAfter(_, spaces) => {
                debug_assert!(!spaces.is_empty());

                spaces.iter().any(|s| is_comment(s))
            }

            Pattern::Nested(nested_pat) => is_multiline_pattern(nested_pat),

            Pattern::RecordDestructure(fields) => fields.iter().any(|f| f.value.is_multiline()),
            Pattern::RecordField(_, subpattern) => subpattern.value.is_multiline(),

            Pattern::Identifier(_)
            | Pattern::GlobalTag(_)
            | Pattern::PrivateTag(_)
            | Pattern::Apply(_, _)
            | Pattern::NumLiteral(_)
            | Pattern::NonBase10Literal { .. }
            | Pattern::FloatLiteral(_)
            | Pattern::StrLiteral(_)
            | Pattern::BlockStrLiteral(_)
            | Pattern::Underscore
            | Pattern::Malformed(_)
            | Pattern::QualifiedIdentifier { .. } => false,
        }
    }

    fn format_with_parens(&self, buf: &mut String<'a>, parens: Parens, indent: u16) {
        use self::Pattern::*;

        match self {
            Identifier(string) => buf.push_str(string),
            GlobalTag(name) | PrivateTag(name) => {
                buf.push_str(name);
            }
            Apply(loc_pattern, loc_arg_patterns) => {
                // Sometimes, an Apply pattern needs parens around it.
                // In particular when an Apply's argument is itself an Apply (> 0) arguments
                let parens = !loc_arg_patterns.is_empty() && parens == Parens::InApply;

                if parens {
                    buf.push('(');
                }

                loc_pattern
                    .value
                    .format_with_parens(buf, Parens::InApply, indent);

                for loc_arg in loc_arg_patterns.iter() {
                    buf.push(' ');
                    loc_arg
                        .value
                        .format_with_parens(buf, Parens::InApply, indent);
                }

                if parens {
                    buf.push(')');
                }
            }
            RecordDestructure(loc_patterns) => {
                buf.push_str("{ ");

                let mut it = loc_patterns.iter().peekable();

                while let Some(loc_pattern) = it.next() {
                    loc_pattern
                        .value
                        .format_with_parens(buf, Parens::NotNeeded, indent);

                    if it.peek().is_some() {
                        buf.push_str(", ");
                    }
                }

                buf.push_str(" }");
            }

            RecordField(name, loc_pattern) => {
                buf.push_str(name);
                buf.push_str(": ");
                loc_pattern
                    .value
                    .format_with_parens(buf, Parens::NotNeeded, indent);
            }

            NumLiteral(string) => buf.push_str(string),
            NonBase10Literal {
                base,
                string,
                is_negative,
            } => {
                if *is_negative {
                    buf.push('-');
                }

                match base {
                    Base::Hex => buf.push_str("0x"),
                    Base::Octal => buf.push_str("0o"),
                    Base::Binary => buf.push_str("0b"),
                    Base::Decimal => { /* nothing */ }
                }

                buf.push_str(string);
            }
            FloatLiteral(string) => buf.push_str(string),
            StrLiteral(string) => buf.push_str(string),
            BlockStrLiteral(lines) => {
                for line in *lines {
                    buf.push_str(line)
                }
            }
            Underscore => buf.push('_'),

            // Space
            SpaceBefore(sub_pattern, spaces) => {
                if !sub_pattern.is_multiline() {
                    fmt_comments_only(buf, spaces.iter(), indent)
                } else {
                    fmt_spaces(buf, spaces.iter(), indent);
                }
                sub_pattern.format_with_parens(buf, parens, indent);
            }
            SpaceAfter(sub_pattern, spaces) => {
                sub_pattern.format_with_parens(buf, parens, indent);
                // if only_comments {
                if !sub_pattern.is_multiline() {
                    fmt_comments_only(buf, spaces.iter(), indent)
                } else {
                    fmt_spaces(buf, spaces.iter(), indent);
                }
            }

            Nested(sub_pattern) => {
                sub_pattern.format_with_parens(buf, parens, indent);
            }

            // Malformed
            Malformed(string) => buf.push_str(string),
            QualifiedIdentifier { module_name, ident } => {
                if !module_name.is_empty() {
                    buf.push_str(module_name);
                    buf.push('.');
                }

                buf.push_str(ident);
            }
        }
    }
}
