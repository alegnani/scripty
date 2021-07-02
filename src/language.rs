#[derive(Debug)]
pub struct Language {
    name: String,
    extension: String,
    launch: String,
}

impl Language {
    pub fn new(name: String, extension: String, launch: String) -> Self {
        Self {
            name,
            extension,
            launch,
        }
    }
    
    pub async fn get_name(&self) -> String {
        self.name.clone()
    }

    pub async fn get_extension(&self) -> String {
        self.extension.clone()
    }

    pub async fn get_launch(&self) -> String {
        self.launch.clone()
    }
}