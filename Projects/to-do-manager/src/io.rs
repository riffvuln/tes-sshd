pub struct IOEngine<'a> {
    file_path: &'a str,
}

impl IOEngine<'_> {
    pub fn new(file_path: &str) -> Self {
        Self { file_path}
    }

    pub fn read(&self) -> Option<String> {
        Some(std::fs::read_to_string(self.file_path).unwrap_or_default())
    }

    pub fn write(&self) -> 
}