use plist::Dictionary;

pub trait Parser {
    fn parse(&self) -> Dictionary;

    fn from_config(file_name: &str) -> anyhow::Result<Self>
    where
        Self: Sized;
}
