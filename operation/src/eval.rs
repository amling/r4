use executor::BoxedExecutor2;
use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::BooleanOption;
use opts::vals::DefaultedOption;
use opts::vals::OptionDefaulter;
use opts::vals::OptionalStringOption;
use opts::vals::RequiredStringOption;
use record::Record;
use record::RecordTrait;
use registry::Registrant;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe2;
use super::OperationBeForBe2;
use super::OperationRegistrant;
use validates::Validates;
use validates::ValidationResult;

option_defaulters! {
    InputRecordsDefaulter: InputType => InputType::Records(),
    InputLinesDefaulter: InputType => InputType::Lines(),

    OutputRecordsDefaulter: OutputType => OutputType::Records(),
    OutputLinesDefaulter: OutputType => OutputType::Lines(),
    OutputGrepDefaulter: OutputType => OutputType::Grep(),

    FalseDefaulter: bool => false,
    TrueDefaulter: bool => true,
}

#[derive(Clone)]
pub enum InputType {
    Records(),
    Lines(),
}

#[derive(Clone)]
pub enum OutputType {
    Records(),
    Lines(),
    Grep(),
}

#[derive(Default)]
struct CodeOptions {
    engine: OptionalStringOption,
    code: RequiredStringOption,
}

impl Validates for CodeOptions {
    type Target = BoxedExecutor2;

    fn validate(self) -> ValidationResult<BoxedExecutor2> {
        let engine = self.engine.validate()?.unwrap_or_else(|| executor::r4l::Impl::names()[0].to_string());
        let executor = executor::REGISTRY.find(&engine, &[])?;
        let executor = executor.parse(&self.code.validate()?)?;
        return Result::Ok(executor);
    }
}

#[derive(Default)]
#[derive(Validates)]
pub struct EvalOptions<I: OptionDefaulter<InputType>, O: OptionDefaulter<OutputType>, R: OptionDefaulter<bool>> {
    invert: BooleanOption,
    code: CodeOptions,
    input: DefaultedOption<InputType, I>,
    output: DefaultedOption<OutputType, O>,
    ret: DefaultedOption<bool, R>,
}

pub trait EvalBe {
    type I: OptionDefaulter<InputType> + Default;
    type O: OptionDefaulter<OutputType> + Default;
    type R: OptionDefaulter<bool> + Default;

    fn names() -> Vec<&'static str>;
    fn help_msg() -> &'static str;
}

pub struct EvalBe2<B: EvalBe> {
    _b: std::marker::PhantomData<B>,
}

impl<B: EvalBe + 'static> Optionsable for EvalBe2<B> {
    type Options = EvalOptions<B::I, B::O, B::R>;

    fn options(opt: &mut OptionsPile<Self::Options>) {
        opt.match_zero(&["v", "invert"], |p| p.invert.set(), "invert truthiness of output values");
        opt.match_zero(&["no-invert"], |p| p.invert.clear(), "(default)");
        opt.match_extra_soft(|p, a| p.code.code.maybe_set_str(a), "code to execute");
        opt.match_single(&["engine"], |p, a| p.code.engine.set_str(a), "'engine' to execute code with");
        opt.add(executor::REGISTRY.help_options("executor"));
        opt.match_zero(&["lua"], |p| p.code.engine.set("lua".to_string()), "evaluate as lua");
        opt.match_zero(&["input-lines"], |p| p.input.set(InputType::Lines()), "provide input as string lines");
        opt.match_zero(&["input-records"], |p| p.input.set(InputType::Records()), "provide input as structured records");
        opt.match_zero(&["output-lines"], |p| p.output.set(OutputType::Lines()), "interpret output as string lines");
        opt.match_zero(&["output-records"], |p| p.output.set(OutputType::Records()), "interpret output as structured records");
        opt.match_zero(&["output-grep"], |p| p.output.set(OutputType::Grep()), "interpret output as flag to indicate if input should be passed");
        opt.match_zero(&["return"], |p| p.ret.set(true), "interpret return value as output");
        opt.match_zero(&["no-return"], |p| p.ret.set(false), "interpret r variable value as output");
    }
}

impl<B: EvalBe + 'static> OperationBe2 for EvalBe2<B> {
    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn help_msg() -> &'static str {
        return B::help_msg();
    }

    fn stream(o: Arc<EvalOptionsValidated<B::I, B::O, B::R>>) -> Stream {
        let f: Box<FnMut(Record) -> Record>;
        f = o.code.stream(o.ret);

        return stream::closures(
            f,
            move |s, e, w| {
                let ri = match o.input {
                    InputType::Records() => e.clone().parse(),
                    InputType::Lines() => Record::from(e.clone().deparse()),
                };
                let ro = s(ri);
                let ro = if o.invert { Record::from(!ro.coerce_bool()) } else { ro };
                return match o.output {
                    OutputType::Records() => w(Entry::Record(ro)),
                    OutputType::Lines() => w(Entry::Line(ro.coerce_string())),
                    OutputType::Grep() => !ro.coerce_bool() || w(e),
                };
            },
            |_s, _w| {
            },
        );
    }
}

pub type EvalImpl<B> = OperationRegistrant<OperationBeForBe2<EvalBe2<B>>>;

pub enum EvalBeImpl {
}

impl EvalBe for EvalBeImpl {
    type I = InputRecordsDefaulter;
    type O = OutputLinesDefaulter;
    type R = TrueDefaulter;

    fn names() -> Vec<&'static str> {
        return vec!["eval"];
    }

    fn help_msg() -> &'static str {
        return "evaluate code on each record, outputting the string results as lines";
    }
}

pub type Impl = EvalImpl<EvalBeImpl>;
