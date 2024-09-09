use charset::Charset;
use exocore::protos::prost::Timestamp;
use exomind_protos::base::{Contact, Email, EmailAttachment, EmailPart, EmailThread};

#[derive(Default)]
pub struct ParsedThread {
    pub thread: EmailThread,
    pub emails: Vec<FlaggedEmail>,
    pub labels: Vec<String>,
}

#[derive(Debug)]
pub struct FlaggedEmail {
    pub proto: Email,
    pub unread: bool,
}

pub fn parse_thread(thread: google_gmail1::api::Thread) -> Result<ParsedThread, anyhow::Error> {
    let mut parsed_thread = ParsedThread::default();
    parsed_thread.thread.source_id = thread.id.unwrap_or_default();

    if let Some(last_message) = thread.messages.as_ref().and_then(|m| m.last()) {
        if let Some(ids) = &last_message.label_ids {
            parsed_thread.labels = ids.clone();
        }
    }

    let mut messages = thread.messages.unwrap_or_default();
    messages.sort_by_key(|m| m.internal_date.unwrap_or_default());

    for message in messages {
        if let Some(part) = message.payload {
            let is_draft = message
                .label_ids
                .as_ref()
                .map(|lbls| lbls.iter().any(|l| l == "DRAFT"))
                .unwrap_or(false);
            if is_draft {
                continue;
            }

            let unread = message
                .label_ids
                .as_ref()
                .map(|labels| labels.iter().any(|l| l == "UNREAD"))
                .unwrap_or(false);
            let mut parsed_email = FlaggedEmail {
                proto: exomind_protos::base::Email {
                    snippet: message.snippet.clone().unwrap_or_default(),
                    source_id: message.id.clone().unwrap_or_default(),
                    ..Default::default()
                },
                unread,
            };

            match parse_part(&part, &mut parsed_email.proto) {
                Ok(()) => {
                    parsed_thread.emails.push(parsed_email);
                }
                Err(err) => {
                    error!(
                        "Error parsing email in thread {:?}: {}",
                        parsed_thread.thread.source_id, err
                    );
                }
            }
        }
    }

    if let Some(last) = parsed_thread.emails.last() {
        parsed_thread.thread.snippet = last.proto.snippet.clone();
        parsed_thread.thread.subject = last.proto.subject.clone();
    }

    Ok(parsed_thread)
}

fn parse_part(part: &google_gmail1::api::MessagePart, email: &mut Email) -> anyhow::Result<()> {
    parse_part_headers(part, email)?;

    let mime_type = part
        .mime_type
        .as_deref()
        .ok_or_else(|| anyhow!("Part has no mime-type"))?;

    let content_type = get_part_header(part, "Content-Type").unwrap_or("UTF-8");

    if let Some(filename) = part.filename.as_deref().filter(|f| !f.is_empty()) {
        parse_attachment(part, email, mime_type, filename);

        return Ok(());
    }

    match mime_type {
        "text/html" | "text/plain" => {
            let body_bytes = part
                .body
                .as_ref()
                .and_then(|b| b.data.as_ref())
                .ok_or_else(|| {
                    anyhow!(
                        "Expected the part to have a body, but got none. Mime:{}",
                        mime_type
                    )
                })?;

            let encoding = mailparse::parse_content_type(content_type);
            let utf8_charset = Charset::for_label(b"UTF-8").unwrap();
            let charset = Charset::for_label(encoding.charset.as_bytes()).unwrap_or(utf8_charset);

            // Sometimes, the header indicates a charset, but then the HTML indicates
            // another charset. This happens mostly on non-UTF-8 charsets, so in
            // this case, we try to detect the right encoding. See https://stackoverflow.com/questions/27037816/can-an-email-header-have-different-character-encoding-than-the-body-of-the-email
            let charset = if charset != utf8_charset {
                let mut detector = chardetng::EncodingDetector::new();
                detector.feed(body_bytes, true);
                let encoding = detector.guess(None, true);
                Charset::for_encoding(encoding)
            } else {
                charset
            };

            let (decoded, _detected_charset, had_errors) = charset.decode(body_bytes);

            if had_errors {
                warn!(
                    "Error decoding body: charset={:?} body={}",
                    charset,
                    String::from_utf8_lossy(body_bytes)
                );
            }

            email.parts.push(EmailPart {
                mime_type: mime_type.to_string(),
                body: decoded.to_string(),
            });
        }
        "multipart/mixed" | "multipart/alternative" | "multipart/related" | "multipart/signed" => {
            let empty_parts = Vec::new();
            let sub_parts = part.parts.as_ref().unwrap_or(&empty_parts);
            for sub_part in sub_parts {
                parse_part(sub_part, email)?;
            }
        }
        other => {
            warn!("Unhandled mime-type: {}", other);
        }
    }

    Ok(())
}

fn parse_attachment(
    part: &google_gmail1::api::MessagePart,
    email: &mut Email,
    mime_type: &str,
    filename: &str,
) {
    let attachment_id = get_part_header(part, "Content-Id")
        .or_else(|| get_part_header(part, "X-Attachment-Id"))
        .map(|s| s.to_string());

    let key = attachment_id
        .clone()
        .unwrap_or_else(|| filename.to_string());

    let size = part
        .body
        .as_ref()
        .and_then(|b| b.size)
        .map(|size| size as u64)
        .unwrap_or_default();

    email.attachments.push(EmailAttachment {
        key,
        name: filename.to_string(),
        mime_type: mime_type.to_string(),
        inline_placeholder: attachment_id.unwrap_or_default(),
        size,
        ..Default::default()
    });
}

fn parse_part_headers(
    part: &google_gmail1::api::MessagePart,
    email: &mut Email,
) -> anyhow::Result<()> {
    let empty_headers = Vec::new();
    let headers = part.headers.as_ref().unwrap_or(&empty_headers);

    for header in headers {
        let name = if let Some(name) = &header.name {
            name.to_lowercase()
        } else {
            continue;
        };

        let value = if let Some(value) = &header.value {
            value.as_str()
        } else {
            continue;
        };

        match name.as_str() {
            "subject" => email.subject = value.to_string(),
            "from" => {
                email.from = parse_contacts(value)?.into_iter().next();
            }
            "to" => {
                email.to = parse_contacts(value).unwrap_or_default();
            }
            "cc" => {
                email.cc = parse_contacts(value).unwrap_or_default();
            }
            "bcc" => {
                email.bcc = parse_contacts(value).unwrap_or_default();
            }
            "date" => {
                let ts = mailparse::dateparse(value)?;
                email.received_date = Some(Timestamp {
                    seconds: ts,
                    nanos: 0,
                });
            }
            _ => {}
        }
    }

    Ok(())
}

fn get_part_header<'p>(part: &'p google_gmail1::api::MessagePart, key: &str) -> Option<&'p str> {
    let headers = part.headers.as_ref()?;

    headers
        .iter()
        .find(|h| h.name.as_deref() == Some(key))
        .and_then(|h| h.value.as_deref())
}

fn parse_contacts(value: &str) -> anyhow::Result<Vec<Contact>> {
    let addrs = mailparse::addrparse(value)?;
    let mut contacts = Vec::new();

    for addr in addrs.iter() {
        match addr {
            mailparse::MailAddr::Single(single) => {
                contacts.push(Contact {
                    name: single.display_name.clone().unwrap_or_default(),
                    email: single.addr.clone(),
                });
            }
            mailparse::MailAddr::Group(group) => {
                for single in group.addrs.iter() {
                    contacts.push(Contact {
                        name: single
                            .display_name
                            .clone()
                            .unwrap_or_else(|| group.group_name.clone()),
                        email: single.addr.clone(),
                    });
                }
            }
        }
    }

    Ok(contacts)
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use exocore::core::tests_utils::find_test_fixture;

    use super::*;

    #[test]
    fn parse_html_simple() -> anyhow::Result<()> {
        let thread = read_thread_fixture("html_simple")?;
        let parsed = parse_thread(thread)?;
        assert_eq!("17199eb1b9ffdc7b", parsed.thread.source_id);
        assert_eq!("Some snippet", parsed.thread.snippet);
        assert_eq!("Some subject", parsed.thread.subject);
        assert_eq!(vec!["UNREAD", "CATEGORY_UPDATES", "INBOX"], parsed.labels);
        assert!(parsed.emails[0].unread);

        assert_eq!(
            Some(Contact {
                name: "From Someone".to_string(),
                email: "some@email.com".to_string(),
            }),
            parsed.emails[0].proto.from
        );

        Ok(())
    }

    #[test]
    fn parse_multipart_plain_and_html() {
        let thread = read_thread_fixture("multipart_plain_and_html").unwrap();
        let parsed = parse_thread(thread).unwrap();

        assert_eq!(1, parsed.emails.len());
        assert_eq!(2, parsed.emails[0].proto.parts.len());
    }

    #[test]
    fn parse_multiple_messages() {
        let thread = read_thread_fixture("multiple_messages").unwrap();
        let parsed = parse_thread(thread).unwrap();

        assert_eq!(2, parsed.emails.len());
    }

    #[test]
    fn parse_pgp_signature() {
        let thread = read_thread_fixture("pgp_signature").unwrap();
        let parsed = parse_thread(thread).unwrap();

        assert_eq!(1, parsed.emails.len());
        assert_eq!(2, parsed.emails[0].proto.parts.len());

        assert_eq!(1, parsed.emails[0].proto.attachments.len());
        assert_eq!(
            vec![EmailAttachment {
                key: "signature.asc".to_string(),
                name: "signature.asc".to_string(),
                mime_type: "application/pgp-signature".to_string(),
                size: 849,
                ..Default::default()
            }],
            parsed.emails[0].proto.attachments
        );
    }

    #[test]
    fn parse_windows_1252_header_utf8_body() {
        let thread = read_thread_fixture("windows_1252_header_utf8_body").unwrap();
        let parsed = parse_thread(thread).unwrap();

        assert_eq!(1, parsed.emails.len());
        assert_eq!(1, parsed.emails[0].proto.parts.len());

        let body = &parsed.emails[0].proto.parts[0].body;
        assert!(body.contains("Découvrir"));
    }

    #[test]
    fn parse_no_to_address() {
        let thread = read_thread_fixture("no_to_address").unwrap();
        let parsed = parse_thread(thread).unwrap();

        assert_eq!(1, parsed.emails.len());

        // The email is marked as ISO-8859, but is actually UTF8.
        // The detector should have detected correct encoding.
        let body = &parsed.emails[0].proto.parts[0].body;
        assert!(body.contains("reportées"));
    }

    fn read_thread_fixture(file: &str) -> Result<google_gmail1::api::Thread, anyhow::Error> {
        let path = find_test_fixture(&format!(
            "integrations/gmail/fixtures/threads/{}.json",
            file
        ));

        let mut file = std::fs::File::open(path)?;

        let mut body = String::new();
        file.read_to_string(&mut body)?;

        Ok(serde_json::from_str(&body)?)
    }
}
