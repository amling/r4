use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::DefaultedStringOption;
use record::RecordTrait;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::OperationBe;
use super::OperationRegistrant;
use super::SubOperationOption;
use super::TwoRecordUnionOption;

option_defaulters! {
    FileDefaulter: String => "FILE".to_string(),
}

#[derive(Default)]
#[derive(Validates)]
pub struct Options {
    tru: TwoRecordUnionOption,
    fk: DefaultedStringOption<FileDefaulter>,
    op: SubOperationOption,
}

pub(crate) type Impl = OperationRegistrant<ImplBe>;

pub(crate) struct ImplBe();

impl Optionsable for ImplBe {
    type Options = Options;

    fn options(opt: &mut OptionsPile<Options>) {
        opt.add_sub(|p| &mut p.tru, TwoRecordUnionOption::new_options());
        opt.match_single(&["fk", "file-key"], |p, a| p.fk.set_str(a), "key to read file names from (default: 'FILE')");
        opt.match_extra_hard(|p, a| p.op.push(a), "operation to run on each file");
    }
}

impl OperationBe for ImplBe {
    fn names() -> Vec<&'static str> {
        return vec!["expand-files"];
    }

    fn help_msg() -> &'static str {
        return "run an operation on multiple files, themselves listed in input record values";
    }

    fn get_extra(o: Arc<OptionsValidated>) -> Vec<String> {
        return o.op.extra.clone();
    }

    fn stream(o: Arc<OptionsValidated>) -> Stream {
        return stream::closures(
            (),
            move |_s, e, w| {
                let r1 = e.parse();

                let o1 = o.clone();
                let file = r1.get_path(&o.fk).coerce_string();
                let mut substream = stream::compound(
                    o.op.wr.stream(),
                    stream::transform_records(move |r2| {
                        return o1.tru.union(r1.clone(), r2);
                    }),
                );
                for line in BufReader::new(File::open(&file as &str).unwrap()).lines() {
                    let line = line.unwrap();
                    if !substream.write(Entry::Line(Arc::from(line)), w) {
                        // flow hint ends substream, but nothing more
                        break;
                    }
                }
                substream.close(w);

                return true;
            },
            |_s, _w| {
            },
        );
    }
}
