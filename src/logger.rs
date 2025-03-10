pub type LoggerFilter = dyn Fn(&str, &str) -> bool;

pub struct Logger<'a> {
    pub filter: Option<&'a LoggerFilter>,
}

impl Logger<'_> {
    pub fn log(&self, label: &str, message: &str) {
        if let Some(filter) = self.filter {
            if filter(label, message) {
                return;
            }
        }
        println!("{}: {}", label, message);
    }
}
