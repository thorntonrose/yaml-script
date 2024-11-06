pub struct Writer {
    pub log: Vec<String>,
    write_fn: fn(&mut Writer, val: String),
}

impl Writer {
    pub fn new(log_opt: Option<Vec<String>>) -> Writer {
        let write_fn = match log_opt {
            Some(_) => Self::write_log,
            None => Self::write_stdout,
        };

        Self {
            log: log_opt.unwrap_or(Vec::new()),
            write_fn,
        }
    }

    pub fn write(&mut self, val: String) {
        (self.write_fn)(self, val);
    }

    fn write_log(&mut self, val: String) {
        self.log.push(val);
    }

    fn write_stdout(&mut self, val: String) {
        println!("{val}");
    }
}
