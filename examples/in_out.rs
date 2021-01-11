use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_vcd::{reader::VcdReader, writer::VcdWriterBuilder};
use std::fs::File;
use std::io::{BufReader, BufWriter};

// read in a vcd file and write it out again using bit banging

fn main() -> Result<(), std::io::Error> {
	// construct a reader
	let f = BufReader::new(File::open("examples/data/test.vcd")?);
	let mut reader = VcdReader::new(f).unwrap();
	// get the input pin from the reader
	let in_pin = reader.get_pin(&["libsigrok", "data"]).unwrap();

	// construct a writer builder
	let f2 = BufWriter::new(File::create("examples/data/test2.vcd")?);
	let mut writer = VcdWriterBuilder::new(f2).unwrap();
	// add output pin to writer
	let mut out_pin = writer.add_push_pull_pin("data")?;
	// build the writer
	let mut writer = writer.build().unwrap();

	// closure used to copy the input pin from the reader
	// to the output pin of the writer
	let mut copy_pins = || {
		if in_pin.try_is_high().unwrap() {
			out_pin.try_set_high().unwrap();
		} else {
			out_pin.try_set_low().unwrap();
		}
	};

	// get first timestamp from vcd and pass it to the writer
	writer.timestamp(reader.next().unwrap())?;

	// Iterate over all change events in input file.
	// The iterator will yield each timestamp *before* the value
	// changes are applied, so we first need to get a timestamp,
	// save it to `last_t` and start from there.
	for t in reader {
		copy_pins();
		writer.sample()?;
		writer.timestamp(t)?;
	}
	// no timestamp left in file, we need to copy
	// the final pin values to the output and
	// push a last sample
	copy_pins();
	writer.sample()?;

	Ok(())
}
