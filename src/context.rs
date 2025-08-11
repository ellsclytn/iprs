use std::{
    fmt::Display,
    io::{Result, Write},
};

pub struct Ctx<W: Write, E: Write> {
    output: W,
    err_output: E,
    pub errored: bool,
}

impl<W: Write, E: Write> Ctx<W, E> {
    pub fn new(output: W, err_output: E) -> Self {
        Ctx {
            output,
            err_output,
            errored: false,
        }
    }

    pub fn writeln<D: Display>(&mut self, msg: D) -> Result<()> {
        writeln!(self.output, "{msg}")
    }

    pub fn ewriteln<D: Display>(&mut self, msg: D) -> Result<()> {
        writeln!(self.err_output, "{msg}")
    }

    pub fn error_and_exit<D: Display>(&mut self, msg: D) -> ! {
        self.ewriteln(msg).unwrap();
        std::process::exit(1);
    }

    pub fn error_without_exit<D: Display>(&mut self, msg: D) -> Result<()> {
        self.ewriteln(msg)?;
        self.errored = true;

        Ok(())
    }

    #[cfg(test)]
    pub fn output(&self) -> &W {
        &self.output
    }
}

#[cfg(test)]
pub mod test_util {
    use super::Ctx;

    pub fn create_test_ctx() -> Ctx<Vec<u8>, Vec<u8>> {
        Ctx::new(Vec::new(), Vec::new())
    }

    pub fn get_output_as_string(ctx: &Ctx<Vec<u8>, Vec<u8>>) -> String {
        String::from_utf8(ctx.output().clone()).unwrap()
    }
}
