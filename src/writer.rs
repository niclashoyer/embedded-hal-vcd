//! Write VCD files based on pin states.
//!
//!

use crate::pins::*;
use embedded_time::duration::*;
use std::convert::TryInto;
use std::io::Result as IOResult;
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// A builder for a [VcdWriter].
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
    /// Create a new builder from a writer that implements [std::io::Write].
    pub fn new(writer: W) -> IOResult<Self> {
        let mut writer = vcd::Writer::new(writer);
        writer.timescale(1, vcd::TimescaleUnit::NS)?;
        writer.add_module("top")?;
        Ok(VcdWriterBuilder {
            writer,
            pins: vec![],
        })
    }

    /// Add a push pull pin with a corresponding named VCD variable.
    ///
    /// The pin state will be written to the VCD file according to the
    /// state of the pin:
    ///
    /// | Pin state | VCD value |
    /// |-----------|-----------|
    /// | high      | 1         |
    /// | low       | 0         |
    ///
    /// The initial pin state is low.
    pub fn add_push_pull_pin(&mut self, reference: &str) -> IOResult<PushPullPin> {
        let code = self.writer.add_wire(1, reference)?;
        let pin = Arc::new(AtomicPinState::new_with_state(PinState::Low));
        self.pins.push((code, pin.clone()));
        Ok(PushPullPin::new(pin))
    }

    /// Add an open drain pin with a corresponding named VCD variable.
    ///
    /// The pin state will be written to the VCD file according to the
    /// state of the pin:
    ///
    /// | Pin state | VCD value |
    /// |-----------|-----------|
    /// | high      | 0         |
    /// | low       | Z         |
    ///
    /// The initial pin state is floating.
    pub fn add_open_drain_pin(&mut self, reference: &str) -> IOResult<OpenDrainPin> {
        let code = self.writer.add_wire(1, reference)?;
        let pin = Arc::new(AtomicPinState::new_with_state(PinState::Floating));
        self.pins.push((code, pin.clone()));
        Ok(OpenDrainPin::new(pin))
    }

    /// Build a VCD writer.
    ///
    /// This consumes the builder.
    pub fn build(mut self) -> IOResult<VcdWriter<W>> {
        self.writer.upscope()?;
        self.writer.enddefinitions()?;
        Ok(VcdWriter {
            writer: self.writer,
            pins: self.pins,
        })
    }
}

/// A writer for VCD files.
///
/// Write VCD files based on pin states.
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
    /// Write a timestamp to the VCD file.
    ///
    /// A timestamp represents a point in time that is used for the following
    /// pin states.
    pub fn timestamp<D: TryInto<Nanoseconds<u64>>>(&mut self, timestamp: D) -> IOResult<()> {
        let ts: Nanoseconds<u64> = timestamp.try_into().map_err(|_e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "can't convert timestamp to nanoseconds",
            )
        })?;
        self.writer.timestamp(ts.0)
    }

    /// Sample all pins and write their state to the VCD file.
    ///
    /// All assigned pins will be sampled and their state is written
    /// according to the variable configuration.
    pub fn sample(&mut self) -> IOResult<()> {
        for (id, pin) in self.pins.iter() {
            let state: PinState = pin.load(Ordering::SeqCst);
            self.writer.change_scalar(*id, vcd::Value::from(state))?;
        }
        Ok(())
    }
}
