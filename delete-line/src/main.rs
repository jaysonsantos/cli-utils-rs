use std::fs::{rename, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::ops::RangeInclusive;

use clap::Parser;
use color_eyre::eyre::Result;
use env_logger::try_init;
use log::{debug, trace};

use crate::utils::{Range, LINE_ENDING};

pub mod utils;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    filename: String,
    #[arg(short, long)]
    line_number: Range,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    try_init()?;
    let args = Args::parse();

    let destination_filename = format!("{}.new", args.filename);
    // Run in a block to have both files closed when rename is called
    {
        let origin_file_buffer = File::open(&args.filename)?;
        let mut destination_file = File::create(&destination_filename)?;
        skip_lines(
            args.line_number.into_inner(),
            origin_file_buffer,
            &mut destination_file,
        )?;

        destination_file.sync_all()?;
    }

    rename(destination_filename, args.filename)?;
    Ok(())
}

/// Read from input and write to output skipping lines within a range.
/// This is generic function and does not know about the aspects of a file and if it has to
/// sync the file.
pub fn skip_lines<R, W>(
    lines_to_skip: RangeInclusive<usize>,
    input: R,
    output: &mut W,
) -> Result<()>
where
    R: Read,
    W: Write,
{
    for (line_number, line) in BufReader::new(input).lines().enumerate() {
        let line_to_skip = line_number + 1;
        if lines_to_skip.contains(&line_to_skip) {
            debug!("Skipping {}", line_to_skip);
            continue;
        }
        trace!("Including {}", line_to_skip);
        output.write_all(line?.as_bytes())?;
        output.write_all(&LINE_ENDING[..])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::skip_lines;
    use crate::utils::LINE_ENDING;

    #[test]
    fn test_skip_lines() {
        let line_ending = String::from_utf8(LINE_ENDING.to_vec()).unwrap();
        let input = vec!["1", "2", "3"].join(&line_ending);
        let output = vec![];
        let mut output_cursor = Cursor::new(output);

        skip_lines(2..=2, Cursor::new(input), &mut output_cursor).unwrap();

        let output = String::from_utf8(output_cursor.into_inner()).unwrap();

        assert_eq!(
            format!("{}{}", vec!["1", "3"].join(&line_ending), line_ending),
            output
        );
    }

    #[test]
    fn test_skip_initial_lines() {
        let line_ending = String::from_utf8(LINE_ENDING.to_vec()).unwrap();
        let input = vec!["1", "2", "3"].join(&line_ending);
        let output = vec![];
        let mut output_cursor = Cursor::new(output);

        skip_lines(1..=2, Cursor::new(input), &mut output_cursor).unwrap();

        let output = String::from_utf8(output_cursor.into_inner()).unwrap();

        assert_eq!(
            format!("{}{}", vec!["3"].join(&line_ending), line_ending),
            output
        );
    }

    #[test]
    fn test_skip_final_lines() {
        let line_ending = String::from_utf8(LINE_ENDING.to_vec()).unwrap();
        let input = vec!["1", "2", "3"].join(&line_ending);
        let output = vec![];
        let mut output_cursor = Cursor::new(output);

        skip_lines(2..=3, Cursor::new(input), &mut output_cursor).unwrap();

        let output = String::from_utf8(output_cursor.into_inner()).unwrap();

        assert_eq!(
            format!("{}{}", vec!["1"].join(&line_ending), line_ending),
            output
        );
    }

    #[test]
    fn test_skip_last_line() {
        let line_ending = String::from_utf8(LINE_ENDING.to_vec()).unwrap();
        let input = vec!["1", "2", "3"].join(&line_ending);
        let output = vec![];
        let mut output_cursor = Cursor::new(output);

        skip_lines(3..=3, Cursor::new(input), &mut output_cursor).unwrap();

        let output = String::from_utf8(output_cursor.into_inner()).unwrap();

        assert_eq!(
            format!("{}{}", vec!["1", "2"].join(&line_ending), line_ending),
            output
        );
    }
}
