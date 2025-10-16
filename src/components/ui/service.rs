use std::env;

pub async fn admin_page(site_url: &str, site_name: &str, server_url: &str) -> String {
  let recaptcha_v3_key = env::var("recaptchaV3Key")
    .ok()
    .unwrap_or("undefined".to_string());
  let turnstile_key = env::var("turnstileKey")
    .ok()
    .unwrap_or("undefined".to_string());
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
