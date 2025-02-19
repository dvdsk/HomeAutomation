use color_eyre::eyre::Context;
use color_eyre::eyre::Result;
use color_eyre::Section;

use std::fs;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

pub(crate) struct Csv {
    pub(crate) file: BufWriter<std::fs::File>,
}

impl Csv {
    pub(crate) fn open(
        readings: &[protocol::Reading],
        path: PathBuf,
    ) -> Result<Self> {
        let file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .wrap_err("Could not open output csv path")
            .with_note(|| format!("csv path: {}", path.display()))?;
        let mut file = BufWriter::new(file);
        write!(file, "ts").wrap_err("Could not write header")?;
        for reading in readings {
            write!(file, "{reading:?},").wrap_err("Could not write header")?;
        }

        Ok(Self { file })
    }

    pub(crate) fn write_line(&mut self, ts: u64, line: &[f32]) -> Result<()> {
        write!(self.file, "{ts}").wrap_err("Could not write ts to file")?;
        for val in line {
            write!(self.file, "{val}")
                .wrap_err("Could not write float value to file")?;
        }
        write!(self.file, "\n").wrap_err("Could not write lineend to file")?;

        Ok(())
    }
}
