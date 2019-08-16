use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::UnvalidatedOption;
use std::collections::VecDeque;
use std::sync::Arc;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;
use super::head::HeadCountOption;

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
        opt.match_single(&["n"], |p, a| p.n.0.set(a), "count of inputs, may be +N or -N as UNIX tail");
    }
}

impl OperationBe2 for ImplBe2 {
    fn names() -> Vec<&'static str> {
        return vec!["tail"];
    }

    fn help_msg() -> &'static str {
        return "keeps a suffix of inputs";
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        if o.n.sign.is_positive(false) {
            return stream::closures(
                o.n.n,
                |s, e, w| {
                    if *s == 0 {
                        return w(e);
                    }

                    *s -= 1;
                    return true;
                },
                |_s, _w| {
                },
            );
        }
        else {
            return stream::closures(
                VecDeque::new(),
                move |s, e, _w| {
                    s.push_back(e);
                    if s.len() > o.n.n {
                        s.pop_front();
                    }
                    return true;
                },
                |s, w| {
                    for e in s {
                        if !w(e) {
                            return;
                        }
                    }
                },
            );
        }
    }
}
