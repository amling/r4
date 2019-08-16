use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::UnvalidatedOption;
use std::sync::Arc;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;
use validates::ValidationError;

#[derive(Clone)]
pub(crate) enum HeadCountSign {
    POSITIVE,
    NEGATIVE,
    UNSPECIFIED,
}

impl HeadCountSign {
    pub(crate) fn is_positive(&self, default: bool) -> bool {
        return match self {
            HeadCountSign::POSITIVE => true,
            HeadCountSign::NEGATIVE => false,
            HeadCountSign::UNSPECIFIED => default,
        };
    }
}

#[derive(Clone)]
pub(crate) struct HeadCountOption {
    pub(crate) sign: HeadCountSign,
    pub(crate) n: usize,
}

impl Default for HeadCountOption {
    fn default() -> Self {
        return HeadCountOption {
            sign: HeadCountSign::UNSPECIFIED,
            n: 10,
        };
    }
}

impl HeadCountOption {
    pub(crate) fn set(&mut self, mut arg: &str) -> Result<(), ValidationError> {
        let mut sign = HeadCountSign::UNSPECIFIED;
        let mut split = arg.chars();
        match split.next() {
            Some('+') => {
                sign = HeadCountSign::POSITIVE;
                arg = split.as_str();
            }
            Some('-') => {
                sign = HeadCountSign::NEGATIVE;
                arg = split.as_str();
            }
            _ => {
                // OK
            }
        }
        *self = HeadCountOption {
            sign: sign,
            n: arg.parse()?,
        };
        return Result::Ok(());
    }
}

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    n: UnvalidatedOption<HeadCountOption>,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) type ImplBe = OperationBeForBe2<ImplBe2>;

pub(crate) struct ImplBe2();

impl Optionsable for ImplBe2 {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.match_single(&["n"], |p, a| p.n.0.set(a), "count of inputs, may be +N or -N as UNIX head");
    }
}

impl OperationBe2 for ImplBe2 {
    fn names() -> Vec<&'static str> {
        return vec!["head"];
    }

    fn help_msg() -> &'static str {
        return "keeps a prefix of inputs";
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        if o.n.sign.is_positive(true) {
            return stream::closures(
                o.n.n,
                |s, e, w| {
                    if *s == 0 {
                        return false;
                    }

                    *s -= 1;
                    return w(e);
                },
                |_s, _w| {
                },
            );
        }
        else {
            return stream::closures(
                Vec::new(),
                |s, e, _w| {
                    s.push(e);
                    return true;
                },
                move |s, w| {
                    if o.n.n >= s.len() {
                        return;
                    }
                    for e in &s[0..(s.len() - o.n.n)] {
                        if !w(e.clone()) {
                            return;
                        }
                    }
                },
            );
        }
    }
}
