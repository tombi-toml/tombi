use tombi_schema_store::{AllOfSchema, AnyOfSchema, OneOfSchema, ReferableValueSchemas};

pub(crate) trait CompositeSchema {
    fn composite_title(&self) -> Option<String>;
    fn composite_description(&self) -> Option<String>;
    fn schemas(&self) -> &ReferableValueSchemas;
}

impl CompositeSchema for OneOfSchema {
    fn composite_title(&self) -> Option<String> {
        self.title.clone()
    }

    fn composite_description(&self) -> Option<String> {
        self.description.clone()
    }

    fn schemas(&self) -> &ReferableValueSchemas {
        &self.schemas
    }
}

impl CompositeSchema for AnyOfSchema {
    fn composite_title(&self) -> Option<String> {
        self.title.clone()
    }

    fn composite_description(&self) -> Option<String> {
        self.description.clone()
    }

    fn schemas(&self) -> &ReferableValueSchemas {
        &self.schemas
    }
}

impl CompositeSchema for AllOfSchema {
    fn composite_title(&self) -> Option<String> {
        self.title.clone()
    }

    fn composite_description(&self) -> Option<String> {
        self.description.clone()
    }

    fn schemas(&self) -> &ReferableValueSchemas {
        &self.schemas
    }
}
