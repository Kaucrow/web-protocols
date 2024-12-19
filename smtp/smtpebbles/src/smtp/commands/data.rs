use crate::prelude::*;
use crate::settings::get_settings;
use crate::{ smtp::startup::Attachment, SmtpSession, EmailData };
use anyhow::Result;
use mailparse::{ parse_mail, MailHeaderMap, ParsedMail };

impl SmtpSession {
    #[tracing::instrument(
        name = "Handling DATA command",
        skip(self)
    )]
    pub async fn data(&mut self) -> Result<()> {
        if let Some(email) = &self.email {
            if email.data.is_some() {
                self.stream.write_all(b"503 Is this some kind of joke? (Bad sequence of commands)\r\n").await?;
                return Ok(());
            }
        } else {
            self.stream.write_all(b"503 Is this some kind of joke? (Bad sequence of commands)\r\n").await?;
            return Ok(());
        }

        self.email.as_mut().unwrap().data = Some(EmailData {
            from: None,
            to: None,
            subject: None,
            date: None,
            content: None,
            attachments: Vec::new(),
        });

        self.stream.write_all(b"354 Let us see if there is anything important written on this. (Start mail input; end with <CRLF>.<CRLF>)\r\n").await?;
        tracing::debug!(target: "smtp", "DATA command started");

        let (sender_domain, recipient_domain) = {
            let email = self.email.as_ref().unwrap();

            let sender = email.sender.as_ref().unwrap();
            let recipient = email.recipient.as_ref().unwrap();

            (
                sender.splitn(2, '@').nth(1).ok_or(anyhow!("Malformed sender address"))?.to_string(),
                recipient.splitn(2, '@').nth(1).ok_or(anyhow!("Malformed recipient address"))?.to_string(),
            )
        };

        let mut reader = BufReader::new(&mut self.stream);
        let mut mail_buf = String::new();
        
        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line).await?;

            if bytes_read == 0 || line.trim() == "." {
                break;  // End of message when line contains only a period '.'
            }

            mail_buf.push_str(&line);
        }

        let parsed_mail = parse_mail(mail_buf.as_bytes())?;

        let settings = get_settings()?;

        // Executes if the mail is single-part
        if parsed_mail.subparts.is_empty() {
            let data = self.email.as_mut().unwrap().data.as_mut().unwrap();

            extract_headers(&parsed_mail, data)?;

            let body = parsed_mail.get_body()?;
            data.content = Some(body);

        // Executes if the mail is multipart
        } else {
            let email = self.email.as_mut().unwrap();
            let data = email.data.as_mut().unwrap();

            extract_headers(&parsed_mail, data)?;

            let server_user = match &settings.domain {
                domain if *domain == sender_domain => Some(email.sender.as_ref().unwrap()),
                domain if *domain == recipient_domain => Some(email.recipient.as_ref().unwrap()),
                _ => None
            };

            // Process each part in the multipart mail
            for part in parsed_mail.subparts {
                let headers = part.get_headers();

                let content_disposition = headers.get_first_value("Content-Disposition").unwrap_or("inline".to_string());

                // If the part is an attachment, register it, and optionally save its conents
                if content_disposition.starts_with("attachment") {
                    let body = part.get_body_raw()?;
                    let filename = content_disposition.splitn(2, "filename=").nth(1).unwrap_or("unknown_file").trim_matches('"');

                    // If either the sender user or the recipient user belongs to the server's domain, save the file contents
                    if let Some(user) = server_user.as_ref() {
                        let path = {
                            let base_path = PathBuf::from(format!("./attachments/{}/{}", user, filename));
                            let mut path = base_path.clone();

                            let path_parent = base_path.parent().unwrap_or(Path::new(""));
                            let file_stem = base_path.file_stem().unwrap_or(std::ffi::OsStr::new(""));
                            let extension = base_path.extension().unwrap_or(std::ffi::OsStr::new(""));

                            let mut copy_count = 1;
                            while path.exists() {
                                let new_filename = PathBuf::from(format!("{}-{}.{}", file_stem.to_string_lossy(), copy_count, extension.to_string_lossy()));
                                path = path_parent.join(new_filename);
                                copy_count += 1;
                            }

                            path
                        };

                        if let Some(parent_dir) = path.parent() {
                            if !parent_dir.exists() {
                                fs::create_dir_all(parent_dir).await?;
                            }
                        }

                        let mut file = fs::File::create(&path).await.unwrap();
                        file.write_all(&body).await.unwrap();

                        data.attachments.push(Attachment::new(filename.to_string(), body));
                    }

                // If the part is not an attachment, register it as the main body
                } else {
                    let body = part.get_body()?;
                    data.content = Some(body);
                }
            }
        }
      
        self.stream.write_all(b"250 OK: Not that it solves anyone's problem but yours.\r\n").await?;

        match settings.domain {
            domain if domain == sender_domain => self.send_email().await?,
            domain if domain == recipient_domain => self.receive_email().await?,
            _ => {}
        }

        Ok(())
    }
}

// Extracts the main mail headers. This includes the subject, sender, recipient, and date
fn extract_headers(parsed_mail: &ParsedMail, data: &mut EmailData) -> Result<()> {
    let headers = parsed_mail.get_headers();

    data.subject = Some(headers.get_first_value("Subject").unwrap_or_default());
    data.to = Some(headers.get_first_value("To").unwrap_or("<>".to_string()));
    data.from = Some(headers.get_first_value("From").unwrap_or("undisclosed-recipients".to_string()));
    data.date = headers
        .get_first_value("Date")
        .map(|date| DateTime::parse_from_rfc2822(&date).unwrap().with_timezone(&Utc))
        .or(Some(Utc::now()));

    Ok(())
}