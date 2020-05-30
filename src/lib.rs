#![cfg_attr(feature = "drain-filter", feature(drain_filter))]

pub use self::parser::TomlParser;

use lalrpop_util::lalrpop_mod;

pub mod comment;
pub mod format;
pub mod key;
pub mod value;

mod ast;
mod escape;
mod merge;
lalrpop_mod!(parser, "/parser.rs");
mod quotes;

#[cfg(test)]
mod test {
    use super::*;
    use crate::format::Independent;
    use anyhow::Result;

    fn parse<'a>(input: &'a str) -> Result<String> {
        let parser = TomlParser::new();
        let is_inline = |_: &[&str]| false;
        let i = Independent::new(parser.parse(input).unwrap(), is_inline);
        println!("i: {}", i);
        Ok(i.to_string())
        // Ok(Independent::new(parser.parse(input).unwrap(), is_inline).to_string())
    }

    #[test]
    fn temp() -> Result<()> {
        assert_eq!(
            parse(r#""""a """" = 'b'""" = false"#)?.trim(),
            r#""a \"\"\"\" = 'b'" = false"#
        );
        // assert_eq!(parse(r#""a = \"b\"" = false"#)?.trim(), r#"'a = "b"' = false"#);
        // assert_eq!(parse(r#""'a' = \"b\"" = false"#)?.trim(), r#""'a' = \"b\"" = false"#);
        Ok(())
    }

    #[test]
    fn test() -> Result<()> {
        // TODO: trim
        assert_eq!(
            parse(r#"'a = "b"' = false"#)?.trim(),
            r#"'a = "b"' = false"#
        );
        assert_eq!(
            parse(r#""a = 'b'" = false"#)?.trim(),
            r#""a = 'b'" = false"#
        );
        assert_eq!(
            parse(r#""a = \"b\"" = false"#)?.trim(),
            r#"'a = "b"' = false"#
        );
        assert_eq!(
            parse(r#""'a' = \"b\"" = false"#)?.trim(),
            r#""'a' = \"b\"" = false"#
        );

        // assert_eq!(parse(r#"'a = "b"' = false"#)?.trim(), r#"'a = "b"' = false"#);
        assert_eq!(
            parse("\"\"\"a \n = 'b'\"\"\" = false")?.trim(),
            r#""a \n = 'b'" = false"#
        );
        // assert_eq!(parse(r#""a = \"b\"" = false"#)?.trim(), r#"'a = "b"' = false"#);
        // assert_eq!(parse(r#""'a' = \"b\"" = false"#)?.trim(), r#""'a' = \"b\"" = false"#);
        Ok(())
    }
}
