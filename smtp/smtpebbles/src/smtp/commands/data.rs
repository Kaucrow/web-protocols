use crate::prelude::*;
use crate::settings::get_settings;
use crate::{ smtp::startup::Attachment, SmtpSession, EmailData };
use anyhow::Result;
use mailparse::{ parse_mail, MailHeaderMap };

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

        let mut reader = BufReader::new(&mut self.stream);

        let mut set_headers = false;

        let mut raw_headers = String::new();
        let mut content_type: Option<String> = None;

        // Set the email headers
        while !set_headers {
            let mut line = String::new();
            reader.read_line(&mut line).await?;

            line = line.trim_end_matches("\r\n").to_string();

            match line {
                _ if line.starts_with("From: ") => {
                    let from = line.splitn(2, ": ").nth(1).unwrap().trim();
                    self.email.as_mut().unwrap().data.as_mut().unwrap().from = Some(from.to_string());
                }
                _ if line.starts_with("To: ") => {
                    let to = line.splitn(2, ": ").nth(1).unwrap().trim();
                    self.email.as_mut().unwrap().data.as_mut().unwrap().to = Some(to.to_string());
                }
                _ if line.starts_with("Subject: ") => {
                    let subject = line.splitn(2, ": ").nth(1).unwrap().trim();
                    self.email.as_mut().unwrap().data.as_mut().unwrap().subject = Some(subject.to_string());
                }
                _ if line.starts_with("Date: ") => {
                    let date_str = line.splitn(2, ": ").nth(1).unwrap().trim();
                    let date = DateTime::parse_from_rfc2822(date_str)?.with_timezone(&Utc);
                    self.email.as_mut().unwrap().data.as_mut().unwrap().date = Some(date);
                }
                _ if line.starts_with("Content-Type: ") => {
                    let content_type_str = line.splitn(2, ": ").nth(1).unwrap().trim();
                    content_type = Some(content_type_str.to_string());
                }
                _ if line.trim().is_empty() => {
                    set_headers = true;
                }
                _ => {}
            }

            raw_headers.push_str(&format!("{}\n", &line));
        }

        let (sender_domain, recipient_domain) = {
            let email = self.email.as_ref().unwrap();

            let sender = email.sender.as_ref().unwrap();
            let recipient = email.recipient.as_ref().unwrap();

            (
                sender.splitn(2, '@').nth(1).ok_or(anyhow!("Malformed sender address"))?.to_string(),
                recipient.splitn(2, '@').nth(1).ok_or(anyhow!("Malformed recipient address"))?.to_string(),
            )
        };

        let settings = get_settings()?;

        let mut processed_multipart = false;

        if let Some(content_type) = content_type {
            // If the mail is a multipart email
            if content_type.starts_with("multipart") {
                let server_user_email = match settings.domain {
                    domain if domain == sender_domain => Some(self.email.as_ref().unwrap().sender.as_ref().unwrap().clone()),
                    domain if domain == recipient_domain => Some(self.email.as_ref().unwrap().recipient.as_ref().unwrap().clone()),
                    _ => None
                };

                let (content, attachments) = process_mail_multipart(server_user_email, raw_headers, &mut reader).await?;
                let data = self.email.as_mut().unwrap().data.as_mut().unwrap();
                data.content = Some(content);
                data.attachments = attachments;

                processed_multipart = true;
            }
        }

        if !processed_multipart {
            tracing::debug!(target: "smtp", "NO CONTAINS CONTENT_TYPE");
            let mut content_buf = String::new();

            loop {
                let mut line = String::new();
                let bytes_read = reader.read_line(&mut line).await?;

                line = line.trim_end_matches("\r\n").to_string();

                if bytes_read == 0 || line.trim() == "." {
                    break;  // End of message when line contains only a period '.'
                }

                content_buf.push_str(&line);
            }

            self.email.as_mut().unwrap().data.as_mut().unwrap().content = Some(content_buf);
        }

        let data = self.email.as_mut().unwrap().data.as_mut().unwrap();

        data.to.get_or_insert("<>".to_string());
        data.from.get_or_insert("undisclosed-recipients".to_string());
        data.subject.get_or_insert("".to_string());
        data.date.get_or_insert(Utc::now());

        self.stream.write_all(b"250 OK: Not that it solves anyone's problem but yours.\r\n").await?;

        
        let settings = get_settings()?;

        match settings.domain {
            domain if domain == sender_domain => self.send_email().await?,
            domain if domain == recipient_domain => self.receive_email().await?,
            _ => {}
        }

        Ok(())
    }
}
    
async fn process_mail_multipart(server_user_email: Option<String>, raw_headers: String, reader: &mut BufReader<&mut TcpStream>) -> Result<(String, Vec<Attachment>)> {
    let server_user = if let Some(email) = server_user_email {
        Some(email.splitn(2, '@').nth(0).unwrap().to_string())
    } else {
        None
    };

    let raw_mail = {
        let mut content_buf = String::new();

        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line).await?;

            line = line.trim_end_matches("\r\n").to_string();

            if bytes_read == 0 || line.trim() == "." {
                break;  // End of content when line contains only a period '.'
            }

            content_buf.push_str(&format!("{}\n", line));
        }

        format!("{}{}", raw_headers, content_buf)
    };

    let mut body_buf = String::new();
    let mut attachments: Vec<Attachment> = Vec::new();
    let parsed_mail = parse_mail(raw_mail.as_bytes())?;

    for part in &parsed_mail.subparts {
        let headers = part.get_headers();

        let content_disposition = headers.get_first_value("Content-Disposition").unwrap_or("inline".to_string());

        // If the part is an attachment, save its content
        if content_disposition.starts_with("attachment") {
            let body = part.get_body_raw()?;
            let filename = content_disposition.splitn(2, "filename=").nth(1).unwrap_or("unknown_file").trim_matches('"');

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

                file.sync_all().await?;

                let raw_body = std::fs::read(&path)?;

                attachments.push(Attachment::new(filename.to_string(), raw_body));
            }
        } else {
            let body = part.get_body()?;
            body_buf.push_str(&body);
        }
    }

    Ok((body_buf, attachments))
}