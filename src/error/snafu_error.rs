use snafu::Snafu;
use std::env::VarError;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum ErrorHandling {
    #[snafu(display("Unable to read {line_index} from file {file_name}"))]
    BadFile {
        line_index: usize,
        file_name: String,
    },
    #[snafu(display("{source} NO VARIABLE {varname}"))]
    StdEnv { varname: String, source: VarError },
}
