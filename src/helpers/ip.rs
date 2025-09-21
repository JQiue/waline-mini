use ip2region::Searcher;

#[derive(Clone)]
pub struct Ip2Region {
  pub searcher: Searcher,
}

impl Ip2Region {
  pub fn new(file: &str) -> Result<Self, ()> {
    match ip2region::Searcher::new(file) {
      Ok(searcher) => Ok(Self { searcher }),
      Err(err) => {
        tracing::warn!("Can't find xdb file, ip2region cannot be used: {err}");
        Err(())
      }
    }
  }
  pub fn search(&self, ip: &str) -> Option<String> {
    let location = self.searcher.std_search(ip).ok();
    tracing::debug!("ip location: {:#?}", location);
    match location {
      Some(data) => Some(data.province.unwrap_or("".to_string())),
      None => None,
    }
  }
}
