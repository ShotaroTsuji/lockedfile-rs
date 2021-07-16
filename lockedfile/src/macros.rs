#[macro_export]
macro_rules! impl_open_options {
    () => {
        pub fn read(&mut self, read: bool) -> &mut Self {
            self.sys.read(read);

            self
        }

        pub fn write(&mut self, write: bool) -> &mut Self {
            self.sys.write(write);

            self
        }

        pub fn append(&mut self, append: bool) -> &mut Self {
            self.sys.append(append);

            self
        }

        pub fn truncate(&mut self, truncate: bool) -> &mut Self {
            self.sys.truncate(truncate);

            self
        }

        pub fn create(&mut self, create: bool) -> &mut Self {
            self.sys.create(create);

            self
        }

        pub fn create_new(&mut self, create_new: bool) -> &mut Self {
            self.sys.create_new(create_new);

            self
        }
    };
}
