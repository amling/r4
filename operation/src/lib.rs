extern crate aggregator;
extern crate bgop;
extern crate clumper;
extern crate deaggregator;
extern crate executor;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate opts;
extern crate record;
extern crate regex;
#[macro_use]
extern crate registry;
extern crate registry_args;
extern crate sorts;
extern crate stream;
extern crate validates;
#[macro_use]
extern crate validates_derive;

mod tru;
pub(crate) use self::tru::TwoRecordUnionOption;

mod clumper_options;
pub(crate) use self::clumper_options::ClumperOptions;

mod subop_options;
pub(crate) use self::subop_options::SubOperationOption;

mod sort_options;
pub(crate) use self::sort_options::GenericSortBucket;
pub(crate) use self::sort_options::SortOptions;
pub(crate) use self::sort_options::SortOptionsValidated;

use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::IntoArcOption;
use opts::vals::StringVecOption;
use registry::Registrant;
use registry::args::ZeroRegistryArgs;
use std::sync::Arc;
use stream::Stream;
use validates::Validates;
use validates::ValidationError;
use validates::ValidationResult;

pub type BoxedOperation = Box<OperationInbox>;

registry! {
    BoxedOperation,
    aggregate,
    bg,
    chain,
    collate,
    decollate,
    deparse,
    eval,
    expand_files,
    expand_lines,
    from_multi_regex,
    from_regex,
    from_split,
    grep,
    head,
    help,
    join,
    multiplex,
    parse,
    provenance,
    shell,
    sort,
    tail,
    to_ptable,
    to_table,
    wrap_lines,
    xform,
}

pub struct StreamWrapper(Box<Fn() -> Stream + Send + Sync>);

impl StreamWrapper {
    pub fn new<F: Fn() -> Stream + Send + Sync + 'static>(f: F) -> Self {
        return StreamWrapper(Box::new(f));
    }

    pub fn stream(&self) -> Stream {
        return self.0();
    }
}

pub trait OperationBe: Optionsable {
    fn names() -> Vec<&'static str>;
    fn help_msg() -> &'static str;
    fn get_extra(o: Arc<<Self::Options as Validates>::Target>) -> Vec<String>;
    fn stream(o: Arc<<Self::Options as Validates>::Target>) -> Stream;
}

pub trait OperationInbox {
    fn help(&self) -> Vec<String>;
    fn parse(&self, args: &mut Vec<String>) -> ValidationResult<StreamWrapper>;
}

struct OperationInboxImpl<B: OperationBe> {
    _b: std::marker::PhantomData<B>,
}

impl<B: OperationBe> Default for OperationInboxImpl<B> {
    fn default() -> Self {
        return OperationInboxImpl {
            _b: std::marker::PhantomData::default(),
        };
    }
}

impl<B: OperationBe + 'static> OperationInboxImpl<B> where <B::Options as Validates>::Target: Send + Sync {
    fn new_options() -> OptionsPile<B::Options> {
        let mut opt = OptionsPile::<B::Options>::new();
        B::options(&mut opt);
        opt.match_zero(&["help"], |_p| {
            return ValidationError::help(Self::static_help());
        }, "show help");
        return opt;
    }

    fn static_help() -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(format!("{} - {}", B::names()[0], B::help_msg()));
        lines.append(&mut Self::new_options().dump_help());
        return lines;
    }
}

impl<B: OperationBe + 'static> OperationInbox for OperationInboxImpl<B> where <B::Options as Validates>::Target: Send + Sync {
    fn help(&self) -> Vec<String> {
        return Self::static_help();
    }

    fn parse(&self, args: &mut Vec<String>) -> ValidationResult<StreamWrapper> {
        let opt = Self::new_options();
        let o = opt.to_parser().parse(args);
        let o = o.map_err(|e| e.label("While parsing arguments"))?;
        let o = o.validate();
        let o = o.map_err(|e| e.label("While validating arguments"))?;
        let o = Arc::new(o);
        *args = B::get_extra(o.clone());

        return Result::Ok(StreamWrapper::new(move || B::stream(o.clone())));
    }
}

pub struct OperationRegistrant<B: OperationBe> {
    _b: std::marker::PhantomData<B>,
}

impl<B: OperationBe + 'static> Registrant<BoxedOperation> for OperationRegistrant<B> where <B::Options as Validates>::Target: Send + Sync {
    type Args = ZeroRegistryArgs;

    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn help_msg() -> &'static str {
        return B::help_msg();
    }

    fn init(_a: ZeroRegistryArgs) -> BoxedOperation {
        return Box::new(OperationInboxImpl::<B>::default());
    }
}




pub trait OperationBe2: Optionsable {
    fn names() -> Vec<&'static str>;
    fn help_msg() -> &'static str;
    fn stream(o: Arc<<Self::Options as Validates>::Target>) -> Stream;
}

#[derive(Default)]
#[derive(Validates)]
pub struct AndArgsOptions<P: Validates> {
    #[ValidatesName = ""]
    p: IntoArcOption<P>,
    args: StringVecOption,
}

pub struct OperationBeForBe2<B: OperationBe2> {
    _b: std::marker::PhantomData<B>,
}

impl<B: OperationBe2> Optionsable for OperationBeForBe2<B> {
    type Options = AndArgsOptions<B::Options>;

    fn options(opt: &mut OptionsPile<AndArgsOptions<B::Options>>) {
        let mut opt1 = OptionsPile::new();
        B::options(&mut opt1);
        opt.add_sub(|p| &mut p.p.0, opt1);
        opt.match_extra_soft(|p, a| p.args.maybe_push(a), "'extra' files to read as input (default: read standard input instead)");
    }
}

impl<B: OperationBe2> OperationBe for OperationBeForBe2<B> {
    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn help_msg() -> &'static str {
        return B::help_msg();
    }

    fn get_extra(p: Arc<AndArgsOptionsValidated<B::Options>>) -> Vec<String> {
        return p.args.clone();
    }

    fn stream(p: Arc<AndArgsOptionsValidated<B::Options>>) -> Stream {
        return B::stream(p.p.clone());
    }
}
