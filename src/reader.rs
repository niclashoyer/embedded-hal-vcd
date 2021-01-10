use crate::pins::*;
use core::borrow::Borrow;
use embedded_time::duration::*;
use fnv::FnvHashMap;
use std::io::Result as IOResult;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub struct VcdReader<R>
where
	R: std::io::Read,
{
	parser: vcd::Parser<R>,
	scale: Generic<u64>,
	header: vcd::Header,
	pins: FnvHashMap<vcd::IdCode, Arc<AtomicPinState>>,
}

impl<R> VcdReader<R>
where
	R: std::io::Read,
{
	pub fn new(read: R) -> IOResult<Self> {
		let mut parser = vcd::Parser::new(read);
		let header = parser.parse_header()?;
		let scale = Self::timescale_to_duration(&header).unwrap();
		Ok(Self {
			parser,
			header,
			scale,
			pins: FnvHashMap::default(),
		})
	}

	pub fn scale(&self) -> Generic<u64> {
		self.scale
	}

	fn timescale_to_duration(header: &vcd::Header) -> Option<Generic<u64>> {
		if let Some((scale, unit)) = header.timescale {
			let fraction = Fraction::new(1, unit.divisor() as u32);
			Some(Generic::new(scale as u64, fraction))
		} else {
			None
		}
	}

	pub fn get_pin<S>(&mut self, path: &[S]) -> Option<InputPin>
	where
		S: Borrow<str>,
	{
		if let Some(v) = self.header.find_var(path) {
			let state = Arc::new(AtomicPinState::new(PinState::Floating));
			let pin = InputPin::new(state.clone());
			self.pins.insert(v.code, state);
			Some(pin)
		} else {
			None
		}
	}
}

impl<R> Iterator for VcdReader<R>
where
	R: std::io::Read,
{
	type Item = Generic<u64>;

	fn next(&mut self) -> Option<Self::Item> {
		use vcd::Command::*;
		let mut timestamp = None;
		while let Some(cmd) = self.parser.next() {
			match cmd {
				Ok(Timestamp(t)) => {
					timestamp = Some(Generic::new(
						self.scale.integer() * t,
						*self.scale.scaling_factor(),
					));
					break;
				}
				Ok(ChangeScalar(id, val)) => {
					if let Some(pin) = self.pins.get_mut(&id) {
						(*pin).store(val.into(), Ordering::SeqCst);
					}
				}
				_ => {}
			}
		}
		timestamp
	}
}
