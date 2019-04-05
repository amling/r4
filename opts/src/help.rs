use super::parser::ExtraHandler;
use super::parser::OptionsMatch;

pub struct OptionsHelp {
    meta: Option<String>,
    msg: Option<String>,
}

impl OptionsHelp {
    pub(crate) fn to_pair<P>(&self, m: &OptionsMatch<P>) -> (String, String) {
        let mut lhs;
        match *m {
            OptionsMatch::Args(ref aliases, argct, _) => {
                lhs = String::new();
                for (i, alias) in aliases.iter().enumerate() {
                    if i > 0 {
                        lhs.push_str("|")
                    }
                    lhs.push_str("-");
                    if alias.len() > 1 {
                        lhs.push_str("-");
                    }
                    lhs.push_str(alias);
                }
                if argct > 0 {
                    match self.meta {
                        Some(ref s) => {
                            lhs.push_str(" ");
                            lhs.push_str(s);
                        }
                        None => {
                            for _ in 0..argct {
                                lhs.push_str(" <arg>");
                            }
                        }
                    }
                }
            }
            OptionsMatch::Extra(ExtraHandler::Soft(_)) => {
                lhs = match self.meta {
                    Some(ref s) => s.clone(),
                    None => "<arg>".to_string(),
                };
            }
            OptionsMatch::Extra(ExtraHandler::Hard(_)) => {
                lhs = match self.meta {
                    Some(ref s) => s.clone(),
                    None => "<args>".to_string(),
                };
            }
        }

        let rhs = self.msg.clone().unwrap_or_else(String::new);

        return (lhs, rhs);
    }
}

pub trait ToOptionsHelpString {
    fn to_help_string(self) -> String;
}

impl ToOptionsHelpString for String {
    fn to_help_string(self) -> String {
        return self;
    }
}

impl ToOptionsHelpString for &str {
    fn to_help_string(self) -> String {
        return self.to_string();
    }
}

pub trait ToOptionsHelp {
    fn to_help(self) -> Option<OptionsHelp>;
}

impl ToOptionsHelp for () {
    fn to_help(self) -> Option<OptionsHelp> {
        return Some(OptionsHelp {
            meta: None,
            msg: None,
        });
    }
}

impl<S: ToOptionsHelpString> ToOptionsHelp for S {
    fn to_help(self) -> Option<OptionsHelp> {
        return Some(OptionsHelp {
            meta: None,
            msg: Some(self.to_help_string()),
        });
    }
}

impl<S1: ToOptionsHelpString, S2: ToOptionsHelpString> ToOptionsHelp for (S1, S2) {
    fn to_help(self) -> Option<OptionsHelp> {
        return Some(OptionsHelp {
            meta: Some(self.0.to_help_string()),
            msg: Some(self.1.to_help_string()),
        });
    }
}

pub enum NoHelp {
}

impl ToOptionsHelp for Option<NoHelp> {
    fn to_help(self) -> Option<OptionsHelp> {
        return None;
    }
}
