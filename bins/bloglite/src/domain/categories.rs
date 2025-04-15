#[derive(Debug, Clone)]
pub struct Category {
    id: String,
    name: String,
}

impl Category {
    pub fn new<T: Into<String>>(id: T, name: T) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub trait CategoryRepository {
    type Error;
    fn find(
        &self,
        id: &impl AsRef<str>,
    ) -> impl std::future::Future<Output = Result<Option<Category>, Self::Error>>;
}
