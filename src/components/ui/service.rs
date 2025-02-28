use std::env;

pub async fn admin_page() -> String {
  let site_url = env::var("SITE_URL").ok().unwrap_or("''".to_string());
  let site_name = env::var("SITE_NAME").ok().unwrap_or("''".to_string());
  let recaptcha_v3_key = env::var("recaptchaV3Key")
    .ok()
    .unwrap_or("undefined".to_string());
  let turnstile_key = env::var("turnstileKey")
    .ok()
    .unwrap_or("undefined".to_string());
  let server_url = env::var("SERVER_URL").ok().unwrap_or("".to_string());
  format!(
    r#"<!doctype html>
       <html>
         <head>
           <meta charset="utf-8">
           <title>Waline Management System</title>
           <meta name="viewport" content="width=device-width,initial-scale=1">
         </head>
         <body>
           <script>
           window.SITE_URL = `{site_url}`;
           window.SITE_NAME = `{site_name}`;
           window.recaptchaV3Key = {recaptcha_v3_key};
           window.turnstileKey = {turnstile_key};
           window.serverURL = '{server_url}/api/'
           </script>
           <script src="//unpkg.com/@waline/admin"></script>
         </body>
       </html>"#
  )
}

pub async fn index_page() -> String {
  let server_url = env::var("SERVER_URL")
    .ok()
    .unwrap_or("https://waline-mini-hydv.shuttle.app".to_string());
  format!(
    r#"<!DOCTYPE html>
<html lang="en">

<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>waline-mini by shuttle deploy</title>
  <link rel="stylesheet" href="https://unpkg.com/@waline/client@v3/dist/waline.css" />
</head>

<body>
  <div id="article-info" style="text-align: center;">
    浏览量: <span class="waline-pageview-count" data-path="/" />
  </div>
  <div id="waline"></div>
  <script type="module">
    import {{ init }} from 'https://unpkg.com/@waline/client@v3/dist/waline.js';
    init({{
      el: '#waline',
      serverURL: '{server_url}',
      reaction: true,
      pageview: true,
  }});

  </script>
</body>"#
  )
}
