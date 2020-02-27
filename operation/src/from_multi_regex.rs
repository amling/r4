use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::BooleanOption;
use opts::vals::StringVecOption;
use opts::vals::UnvalidatedOption;
use record::Record;
use record::RecordTrait;
use regex::Regex;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;
use validates::ValidationError;
use validates::ValidationResult;

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    res: UnvalidatedOption<Vec<(bool, bool, Vec<String>, Regex)>>,

    keep: StringVecOption,
    keep_all: BooleanOption,

    clobber: BooleanOption,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) type ImplBe = OperationBeForBe2<ImplBe2>;

pub(crate) struct ImplBe2();

impl Optionsable for ImplBe2 {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        fn _add_re(p: &mut Options, pre_flush: bool, post_flush: bool, s: &str) -> ValidationResult<()> {
            match s.find('=') {
                Some(idx) => {
                    let keys = (&s[0..idx]).split(',').map(|s| s.to_string()).collect();
                    let re = Regex::new(&s[(idx + 1)..])?;
                    p.res.0.push((pre_flush, post_flush, keys, re));
                }
                None => {
                    return ValidationError::message(format!("No equals in regex spec: {}", s));
                }
            }
            return Result::Ok(());
        }

        opt.match_single(&["re"], |p, a| _add_re(p, false, false, a), "regex to match (no flush)");
        opt.match_single(&["pre"], |p, a| _add_re(p, true, false, a), "regex to match (flush before)");
        opt.match_single(&["post"], |p, a| _add_re(p, false, true, a), "regex to match (flush after)");

        opt.match_single(&["keep"], |p, a| p.keep.push(a), "keys to keep between flushes");
        opt.match_zero(&["keep-all"], |p| p.keep_all.set(), "keep all keys between flushes");

        opt.match_zero(&["clobber"], |p| p.clobber.set(), "overwrite colliding keys rather than flush");
    }
}

impl OperationBe2 for ImplBe2 {
    fn names() -> Vec<&'static str> {
        return vec!["from-multire"];
    }

    fn help_msg() -> &'static str {
        return "parse records by matching multiple regexes against input lines";
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        struct State(Record);

        impl State {
            fn flush(&mut self, o: &OptionsValidated, w: &mut dyn FnMut(Entry) -> bool) {
                if !self.0.expect_hash().is_empty() {
                    // We ignore the flow hint, but that's okay as the
                    // surrounding Stream will remember it and at worst we do
                    // the rest of our process for the line.
                    w(Entry::Record(self.0.clone()));
                }

                if o.keep_all {
                    return;
                }

                let mut r2 = Record::empty_hash();
                for path in o.keep.iter() {
                    if self.0.has_path(path) {
                        r2.set_path(path, self.0.get_path(path));
                    }
                }

                self.0 = r2;
            }
        }

        return stream::closures(
            (State(Record::empty_hash()), o),
            |s, e, w| {
                let line = e.deparse();

                for (pre_flush, post_flush, keys, re) in s.1.res.iter() {
                    let mut pre_flush = *pre_flush;
                    if let Some(m) = re.captures(&line) {
                        if !s.1.clobber {
                            let ki = keys.iter();
                            let gi = m.iter().skip(1);
                            for (k, g) in ki.zip(gi) {
                                if g.is_some() {
                                    if (s.0).0.has_path(&k) {
                                        pre_flush = true;
                                        break;
                                    }
                                }
                            }
                        }
                        if pre_flush {
                            s.0.flush(&s.1, w);
                        }
                        let ki = keys.iter();
                        let gi = m.iter().skip(1);
                        for (k, g) in ki.zip(gi) {
                            if let Some(m) = g {
                                (s.0).0.set_path(&k, Record::from(m.as_str()));
                            }
                        }
                        if *post_flush {
                            s.0.flush(&s.1, w);
                        }
                    }
                }

                return true;
            },
            |mut s, w| {
                if !s.1.clobber {
                    s.0.flush(&s.1, w);
                }
            },
        );
    }
}
