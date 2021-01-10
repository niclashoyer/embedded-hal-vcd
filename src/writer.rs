use crate::pins::*;
use embedded_time::duration::*;
use std::convert::TryInto;
use std::io::Result as IOResult;
use std::sync::atomic::Ordering;
use std::sync::Arc;

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
