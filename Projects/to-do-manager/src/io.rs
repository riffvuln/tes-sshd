pub struct IOEngine<'a> {
    file_path: &'a str,
}

impl IOEngine<'a> {
    pub fn new(file_path: &'a str) -> Self {
        Self { file_path}
    }

    pub fn read(&self) -> Option<String> {
        Some(std::fs::read_to_string(self.file_path).unwrap_or_else(|| None))
    }

    pub fn write(&self, content: &str) -> bool {
        std::fs::write(self.file_path, content).is_ok()
    } 
}