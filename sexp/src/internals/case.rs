use std::fmt::Display;

use self::RenameRule::*;

#[derive(Default, Copy, Clone, PartialEq)]
pub enum RenameRule {
    #[default]
    None,
    LowerCase,
    UpperCase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

static RENAME_RULES: &[(&str, RenameRule)] = &[
    ("lowercase", LowerCase),
    ("UPPERCASE", UpperCase),
    ("PascalCase", PascalCase),
    ("camelCase", CamelCase),
    ("SCREAMING_SNAKE_CASE", ScreamingSnakeCase),
    ("kebab-case", KebabCase),
    ("SCREAMING-KEBAB-CASE", ScreamingKebabCase),
];

impl RenameRule {
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        for (name, rule) in RENAME_RULES {
            if s == *name {
                return Ok(*rule);
            }
        }
        Err(ParseError(s))
    }

    pub fn apply_to_variant(self, variant: &str) -> String {
        match self {
            None | PascalCase => variant.to_owned(),
            LowerCase => variant.to_ascii_lowercase(),
            UpperCase => variant.to_ascii_uppercase(),
            CamelCase => variant[..1].to_ascii_lowercase() + &variant[1..],
            SnakeCase => {
                let mut s = String::new();
                for (i, ch) in variant.char_indices() {
                    if i > 0 && ch.is_uppercase() {
                        s.push('_');
                    }
                    s.push(ch.to_ascii_lowercase());
                }
                s
            }
            ScreamingSnakeCase => SnakeCase.apply_to_variant(variant).to_ascii_uppercase(),
            KebabCase => SnakeCase.apply_to_variant(variant).replace('_', "-"),
            ScreamingKebabCase => ScreamingSnakeCase
                .apply_to_variant(variant)
                .replace('_', "-"),
        }
    }

    pub fn apply_to_field(self, field: &str) -> String {
        match self {
            None | LowerCase | SnakeCase => field.to_owned(),
            UpperCase => field.to_ascii_uppercase(),
            PascalCase => {
                let mut s = String::new();
                let mut capitalize = true;

                for ch in field.chars() {
                    if ch == '_' {
                        capitalize = true;
                    } else if capitalize {
                        s.push(ch.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        s.push(ch);
                    }
                }
                s
            }
            CamelCase => {
                let pascal = PascalCase.apply_to_field(field);
                pascal[..1].to_ascii_lowercase() + &pascal[1..]
            }
            ScreamingSnakeCase => field.to_ascii_uppercase(),
            KebabCase => field.replace('_', "-"),
            ScreamingKebabCase => ScreamingSnakeCase.apply_to_field(field).replace('_', "-"),
        }
    }
}

pub struct ParseError<'a>(&'a str);

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unknown rename rule for #[sexp(rename_all = {:?})]",
            self.0
        )
    }
}
