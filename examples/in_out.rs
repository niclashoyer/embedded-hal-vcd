use embedded_hal::digital as hal;
use embedded_hal_vcd::*;
use std::fs;

fn main() -> Result<(), std::io::Error> {
	let f = fs::File::open("examples/test.vcd")?;
	let f2 = fs::File::create("examples/test2.vcd")?;
	let mut reader = VcdReader::new(f).unwrap();
	let in_pin = reader.get_pin(&["libsigrok", "data"]).unwrap();
	let mut writer = VcdWriterBuilder::new(f2).unwrap();
	let mut out_pin = writer.add_push_pull_pin("data")?;
	let mut writer = writer.build().unwrap();

	let mut last_t = None;
	for t in reader {
		println!("{:?}: {:?}", t, in_pin);
		if last_t.is_none() {
			last_t = Some(t);
		} else {
			writer.timestamp(last_t.unwrap())?;
			let high = hal::InputPin::try_is_high(&in_pin).unwrap();
			if high {
				hal::OutputPin::try_set_high(&mut out_pin).unwrap();
			} else {
				hal::OutputPin::try_set_low(&mut out_pin).unwrap();
			}
			writer.sample()?;
			last_t = Some(t);
		}
	}

	Ok(())
}
