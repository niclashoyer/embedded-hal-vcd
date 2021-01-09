use core::borrow::Borrow;
use core::convert::Infallible;
use embedded_hal::digital as hal;
use fnv::FnvHashMap;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::convert::TryInto;
use std::io::Result as IOResult;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use embedded_time::duration::*;

#[derive(Clone, Debug, PartialEq, FromPrimitive, ToPrimitive)]
pub enum PinState {
	High = 1,
	Low,
	Floating,
}

impl From<vcd::Value> for PinState {
	fn from(val: vcd::Value) -> PinState {
		use vcd::Value::*;
		match val {
			V0 => PinState::Low,
			V1 => PinState::High,
			Z => PinState::Floating,
			X => PinState::Floating,
		}
	}
}

impl From<PinState> for vcd::Value {
	fn from(state: PinState) -> vcd::Value {
		use vcd::Value;
		use PinState::*;
		match state {
			High => Value::V1,
			Low => Value::V0,
			Floating => Value::Z,
		}
	}
}

#[derive(Debug)]
pub struct AtomicPinState {
	state: AtomicUsize,
}

impl AtomicPinState {
	pub fn new(state: PinState) -> Self {
		AtomicPinState {
			state: AtomicUsize::new(state.to_usize().unwrap()),
		}
	}

	pub fn load(&self, order: Ordering) -> PinState {
		PinState::from_usize(self.state.load(order)).unwrap()
	}

	pub fn store(&self, state: PinState, order: Ordering) {
		self.state.store(state.to_usize().unwrap(), order);
	}
}

#[derive(Clone, Debug)]
pub struct InputPin {
	state: Arc<AtomicPinState>,
}

impl InputPin {
	pub fn new(state: Arc<AtomicPinState>) -> Self {
		InputPin { state }
	}
}

impl hal::InputPin for InputPin {
	type Error = Infallible;

	fn try_is_high(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::High)
	}

	fn try_is_low(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::Low)
	}
}

#[derive(Clone, Debug)]
pub struct PushPullPin {
	state: Arc<AtomicPinState>,
}

impl PushPullPin {
	pub fn new(state: Arc<AtomicPinState>) -> Self {
		PushPullPin { state }
	}
}

impl hal::OutputPin for PushPullPin {
	type Error = Infallible;

	fn try_set_high(&mut self) -> Result<(), Self::Error> {
		Ok(self.state.store(PinState::High, Ordering::SeqCst))
	}

	fn try_set_low(&mut self) -> Result<(), Self::Error> {
		Ok(self.state.store(PinState::Low, Ordering::SeqCst))
	}
}

impl hal::InputPin for PushPullPin {
	type Error = Infallible;

	fn try_is_high(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::High)
	}

	fn try_is_low(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::Low)
	}
}

#[derive(Clone, Debug)]
pub struct OpenGainPin {
	state: Arc<AtomicPinState>,
}

impl OpenGainPin {
	pub fn new(state: Arc<AtomicPinState>) -> Self {
		OpenGainPin { state }
	}
}

impl hal::OutputPin for OpenGainPin {
	type Error = Infallible;

	fn try_set_high(&mut self) -> Result<(), Self::Error> {
		Ok(self.state.store(PinState::Low, Ordering::SeqCst))
	}

	fn try_set_low(&mut self) -> Result<(), Self::Error> {
		Ok(self.state.store(PinState::Floating, Ordering::SeqCst))
	}
}

impl hal::InputPin for OpenGainPin {
	type Error = Infallible;

	fn try_is_high(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::High)
	}

	fn try_is_low(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::Low)
	}
}

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

pub struct VcdWriterBuilder<W>
where
	W: std::io::Write,
{
	writer: vcd::Writer<W>,
	pins: Vec<(vcd::IdCode, Arc<AtomicPinState>)>,
}

impl<W> VcdWriterBuilder<W>
where
	W: std::io::Write,
{
	pub fn new(writer: W) -> IOResult<Self> {
		let mut writer = vcd::Writer::new(writer);
		writer.timescale(1, vcd::TimescaleUnit::NS)?;
		writer.add_module("top")?;
		Ok(VcdWriterBuilder {
			writer,
			pins: vec![],
		})
	}

	pub fn add_push_pull_pin(&mut self, reference: &str) -> IOResult<PushPullPin> {
		let code = self.writer.add_wire(1, reference)?;
		let pin = Arc::new(AtomicPinState::new(PinState::Floating));
		self.pins.push((code, pin.clone()));
		Ok(PushPullPin::new(pin))
	}

	pub fn add_open_gain_pin(&mut self, reference: &str) -> IOResult<OpenGainPin> {
		let code = self.writer.add_wire(1, reference)?;
		let pin = Arc::new(AtomicPinState::new(PinState::Floating));
		self.pins.push((code, pin.clone()));
		Ok(OpenGainPin::new(pin))
	}

	pub fn build(mut self) -> IOResult<VcdWriter<W>> {
		self.writer.upscope()?;
		self.writer.enddefinitions()?;
		Ok(VcdWriter {
			writer: self.writer,
			pins: self.pins,
		})
	}
}

pub struct VcdWriter<W>
where
	W: std::io::Write,
{
	writer: vcd::Writer<W>,
	pins: Vec<(vcd::IdCode, Arc<AtomicPinState>)>,
}

impl<W> VcdWriter<W>
where
	W: std::io::Write,
{
	pub fn timestamp<D: TryInto<Nanoseconds<u64>>>(&mut self, timestamp: D) -> IOResult<()> {
		let ts: Nanoseconds<u64> = timestamp.try_into().map_err(|_e| {
			std::io::Error::new(
				std::io::ErrorKind::InvalidInput,
				"can't convert timestamp to nanoseconds",
			)
		})?;
		self.writer.timestamp(ts.0)
	}

	pub fn sample(&mut self) -> IOResult<()> {
		for (id, pin) in self.pins.iter() {
			let state: PinState = pin.load(Ordering::SeqCst);
			self.writer.change_scalar(*id, vcd::Value::from(state))?;
		}
		Ok(())
	}
}
