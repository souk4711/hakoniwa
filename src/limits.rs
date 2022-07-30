#[derive(Default)]
pub struct Limits {
    pub r#as: Option<u64>,
    pub core: Option<u64>,
    pub cpu: Option<u64>,
    pub fsize: Option<u64>,
    pub nofile: Option<u64>,
}
