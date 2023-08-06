extern crate clap;

use anyhow::anyhow;
use clap::{value_parser, Arg, ArgAction, ArgGroup};
use std::path::Path;

use crate::commands::engine::*;
use crate::commands::utils::*;
use crate::commands::Command;
use crate::thin::delta::*;
use crate::thin::delta_visitor::Snap;

//------------------------------------------

pub struct ThinDeltaCommand;

impl ThinDeltaCommand {
    fn cli(&self) -> clap::Command {
        let cmd = clap::Command::new(self.name())
            .next_display_order(None)
            .version(crate::tools_version!())
            .about("Print the differences in the mappings between two thin devices")
            .arg(
                Arg::new("METADATA_SNAPSHOT")
                    .help("Use metadata snapshot")
                    .short('m')
                    .long("metadata-snap")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("VERBOSE")
                    .help("Provide extra information on the mappings")
                    .long("verbose")
                    .action(ArgAction::SetTrue),
            )
            // options
            .arg(
                Arg::new("ROOT1")
                    .help("The root block for the first thin volume to diff")
                    .long("root1")
                    .value_name("BLOCKNR")
                    .value_parser(value_parser!(u64)),
            )
            .arg(
                Arg::new("ROOT2")
                    .help("The root block for the second thin volume to diff")
                    .long("root2")
                    .value_name("BLOCKNR")
                    .value_parser(value_parser!(u64)),
            )
            .arg(
                Arg::new("THIN1")
                    .help("The numeric identifier for the first thin volume to diff")
                    .long("thin1")
                    .value_name("DEV_ID")
                    .value_parser(value_parser!(u64))
                    .visible_alias("snap1"),
            )
            .arg(
                Arg::new("THIN2")
                    .help("The numeric identifier for the second thin volume to diff")
                    .long("thin2")
                    .value_name("DEV_ID")
                    .value_parser(value_parser!(u64))
                    .visible_alias("snap2"),
            )
            // arguments
            .arg(
                Arg::new("INPUT")
                    .help("Specify the input device")
                    .required(true)
                    .index(1),
            )
            // groups
            .group(
                ArgGroup::new("SNAP1")
                    .args(["ROOT1", "THIN1"])
                    .required(true),
            )
            .group(
                ArgGroup::new("SNAP2")
                    .args(["ROOT2", "THIN2"])
                    .required(true),
            );
        engine_args(cmd)
    }
}

impl<'a> Command<'a> for ThinDeltaCommand {
    fn name(&self) -> &'a str {
        "thin_delta"
    }

    fn run(&self, args: &mut dyn Iterator<Item = std::ffi::OsString>) -> exitcode::ExitCode {
        let matches = self.cli().get_matches_from(args);

        let input_file = Path::new(matches.get_one::<String>("INPUT").unwrap());

        let report = mk_report(false);

        if let Err(e) = check_input_file(input_file).and_then(check_file_not_tiny) {
            return to_exit_code::<()>(&report, Err(e));
        }

        let snap1 = match matches.get_one::<clap::Id>("SNAP1").unwrap().as_str() {
            "THIN1" => Snap::DeviceId(*matches.get_one::<u64>("THIN1").unwrap()),
            "ROOT1" => Snap::RootBlock(*matches.get_one::<u64>("ROOT1").unwrap()),
            _ => return to_exit_code::<()>(&report, Err(anyhow!("unknown option"))),
        };

        let snap2 = match matches.get_one::<clap::Id>("SNAP2").unwrap().as_str() {
            "THIN2" => Snap::DeviceId(*matches.get_one::<u64>("THIN2").unwrap()),
            "ROOT2" => Snap::RootBlock(*matches.get_one::<u64>("ROOT2").unwrap()),
            _ => return to_exit_code::<()>(&report, Err(anyhow!("unknown option"))),
        };

        let engine_opts = parse_engine_opts(ToolType::Thin, &matches);
        if engine_opts.is_err() {
            return to_exit_code(&report, engine_opts);
        }

        let opts = ThinDeltaOptions {
            input: input_file,
            engine_opts: engine_opts.unwrap(),
            report: report.clone(),
            snap1,
            snap2,
            verbose: matches.get_flag("VERBOSE"),
        };

        to_exit_code(&report, delta(opts))
    }
}

//------------------------------------------
