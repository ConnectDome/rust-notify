# ConnectDome - Rust Notify

<img src="https://user-images.githubusercontent.com/41021374/167411018-51939385-d919-4406-bee2-0969aa1c8b9e.png" width="256">

A simple service to announce new blogs on your Slack/Discord + send emails to your list via your TES.

* [x] Feature 1: Posts blogs updates to Discord channel.
* [ ] Feature 2: Sends blog updates to subscribers on our mailing list via Sendgrid.

## Feature 1: Supported Platforms

* [x] Discord
  + [x] Notion
  + [ ] Medium
  + [ ] Revue

* [ ] Slack

## Feature 2: Supported Providers

* [ ] SendGrid
* [x] MailGun
* [ ] SES

### Who is this for?

This is a free and open-source solution for those self-hosting [a blog like us](https://blog.connectdome.com).

### Libraries used

* Tokio
* Reqwest
* Serde
* Lettre
* Tera
* Tracing

### Example config

```toml
interval = 10 # Interval (in seconds) to check for new posts

[discord]
webhook = "..." # Discord webhook URL

[notion]
secret = "..." # Notion integration secret
database = "..." # Notion database ID. The database has to be shared with the integration.

[mail_simple]
to = "..." # Address of the mailing list
from = "..." # From address
domain = "..." # Domain of the SMTP server
user = "..." # Username for the SMTP server
pass = "..." # Password for the SMTP server
```
