use helpers::hash;

pub fn get_avatar(plain: &str) -> String {
  let re = regex::Regex::new(r"^\d+@qq\.com$").unwrap();
  if re.is_match(plain) {
    let number = plain.split("@").next().unwrap();
    format!("https://q1.qlogo.cn/g?b=qq&nk={number}&s=100")
  } else {
    // format!("https://api.multiavatar.com/{}.png", utc_now())
    format!("https://cravatar.cn/avatar/{}", hash::md5(""))
  }
}
