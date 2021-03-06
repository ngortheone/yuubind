use bytes::Bytes;
use std::io;

use crate::{
    byteslice::ByteSlice,
    parseresult::{nom_to_result, ParseError},
    sendable::Sendable,
};

// TODO: (C) factor code over all these with a macro or similar
use crate::{
    data::{command_data_args, DataCommand},
    ehlo::{command_ehlo_args, EhloCommand},
    expn::{command_expn_args, ExpnCommand},
    helo::{command_helo_args, HeloCommand},
    help::{command_help_args, HelpCommand},
    mail::{command_mail_args, MailCommand},
    noop::{command_noop_args, NoopCommand},
    quit::{command_quit_args, QuitCommand},
    rcpt::{command_rcpt_args, RcptCommand},
    rset::{command_rset_args, RsetCommand},
    vrfy::{command_vrfy_args, VrfyCommand},
};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum Command {
    Data(DataCommand), // DATA <CRLF>
    Ehlo(EhloCommand), // EHLO <domain> <CRLF>
    Expn(ExpnCommand), // EXPN <name> <CRLF>
    Helo(HeloCommand), // HELO <domain> <CRLF>
    Help(HelpCommand), // HELP [<subject>] <CRLF>
    Mail(MailCommand), // MAIL FROM:<@ONE,@TWO:JOE@THREE> [SP <mail-parameters>] <CRLF>
    Noop(NoopCommand), // NOOP [<string>] <CRLF>
    Quit(QuitCommand), // QUIT <CRLF>
    Rcpt(RcptCommand), // RCPT TO:<@ONE,@TWO:JOE@THREE> [SP <rcpt-parameters] <CRLF>
    Rset(RsetCommand), // RSET <CRLF>
    Vrfy(VrfyCommand), // VRFY <name> <CRLF>
}

impl Command {
    pub fn parse(arg: Bytes) -> Result<Command, ParseError> {
        nom_to_result(command(ByteSlice::from(&arg)))
    }

    pub fn send_to(&self, w: &mut dyn io::Write) -> io::Result<()> {
        match self {
            &Command::Data(ref c) => c.send_to(w),
            &Command::Ehlo(ref c) => c.send_to(w),
            &Command::Expn(ref c) => c.send_to(w),
            &Command::Helo(ref c) => c.send_to(w),
            &Command::Help(ref c) => c.send_to(w),
            &Command::Mail(ref c) => c.send_to(w),
            &Command::Noop(ref c) => c.send_to(w),
            &Command::Quit(ref c) => c.send_to(w),
            &Command::Rcpt(ref c) => c.send_to(w),
            &Command::Rset(ref c) => c.send_to(w),
            &Command::Vrfy(ref c) => c.send_to(w),
        }
    }
}

named!(command(ByteSlice) -> Command, alt!(
    map!(command_data_args, Command::Data) |
    map!(command_ehlo_args, Command::Ehlo) |
    map!(command_expn_args, Command::Expn) |
    map!(command_helo_args, Command::Helo) |
    map!(command_help_args, Command::Help) |
    map!(command_mail_args, Command::Mail) |
    map!(command_noop_args, Command::Noop) |
    map!(command_quit_args, Command::Quit) |
    map!(command_rcpt_args, Command::Rcpt) |
    map!(command_rset_args, Command::Rset) |
    map!(command_vrfy_args, Command::Vrfy)
));

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{domain::Domain, email::Email, smtpstring::SmtpString};

    #[test]
    fn valid_command() {
        let tests: Vec<(&[u8], Box<fn(Command) -> bool>)> = vec![
            (
                &b"DATA\r\n"[..],
                Box::new(|x| {
                    if let Command::Data(_) = x {
                        true
                    } else {
                        false
                    }
                }),
            ),
            (
                &b"EHLO foo.bar.baz\r\n"[..],
                Box::new(|x| {
                    if let Command::Ehlo(r) = x {
                        SmtpString::from_sendable(r.domain()).unwrap()
                            == SmtpString::from(&b"foo.bar.baz"[..])
                    } else {
                        false
                    }
                }),
            ),
            (
                &b"EXPN mailing.list \r\n"[..],
                Box::new(|x| {
                    if let Command::Expn(r) = x {
                        r.name() == &SmtpString::from(&b"mailing.list "[..])
                    } else {
                        false
                    }
                }),
            ),
            (
                &b"HELO foo.bar.baz\r\n"[..],
                Box::new(|x| {
                    if let Command::Helo(r) = x {
                        SmtpString::from_sendable(r.domain()).unwrap()
                            == SmtpString::from(&b"foo.bar.baz"[..])
                    } else {
                        false
                    }
                }),
            ),
            (
                &b"HELP foo.bar.baz\r\n"[..],
                Box::new(|x| {
                    if let Command::Help(r) = x {
                        r.subject() == &SmtpString::from(&b"foo.bar.baz"[..])
                    } else {
                        false
                    }
                }),
            ),
            (
                &b"HELP \r\n"[..],
                Box::new(|x| {
                    if let Command::Help(r) = x {
                        r.subject() == &SmtpString::from(&b""[..])
                    } else {
                        false
                    }
                }),
            ),
            (
                &b"HELP\r\n"[..],
                Box::new(|x| {
                    if let Command::Help(r) = x {
                        r.subject() == &SmtpString::from(&b""[..])
                    } else {
                        false
                    }
                }),
            ),
            (
                &b"MAIL FROM:<hello@world.example>\r\n"[..],
                Box::new(|x| {
                    if let Command::Mail(r) = x {
                        r.from
                            == Some(Email::new(
                                (&b"hello"[..]).into(),
                                Some(Domain::parse_slice(&b"world.example"[..]).unwrap()),
                            ))
                    } else {
                        false
                    }
                }),
            ),
            (
                &b"NOOP\r\n"[..],
                Box::new(|x| {
                    if let Command::Noop(_) = x {
                        true
                    } else {
                        false
                    }
                }),
            ),
            (
                &b"QUIT\r\n"[..],
                Box::new(|x| {
                    if let Command::Quit(_) = x {
                        true
                    } else {
                        false
                    }
                }),
            ),
            (
                &b"rCpT To: foo@bar.baz\r\n"[..],
                Box::new(|x| {
                    if let Command::Rcpt(r) = x {
                        r.to.raw_localpart().bytes() == &b"foo"[..]
                            && r.to.hostname() == &Some(Domain::parse_slice(b"bar.baz").unwrap())
                    } else {
                        false
                    }
                }),
            ),
            (
                &b"RCPT to:<@foo.bar,@bar.baz:baz@quux.foo>\r\n"[..],
                Box::new(|x| {
                    if let Command::Rcpt(r) = x {
                        r.to.raw_localpart().bytes() == &b"baz"[..]
                            && r.to.hostname() == &Some(Domain::parse_slice(b"quux.foo").unwrap())
                    } else {
                        false
                    }
                }),
            ),
            (
                &b"RSET\r\n"[..],
                Box::new(|x| {
                    if let Command::Rset(_) = x {
                        true
                    } else {
                        false
                    }
                }),
            ),
            (
                &b"RsEt \t \r\n"[..],
                Box::new(|x| {
                    if let Command::Rset(_) = x {
                        true
                    } else {
                        false
                    }
                }),
            ),
            (
                &b"VRFY  root\r\n"[..],
                Box::new(|x| {
                    if let Command::Vrfy(r) = x {
                        r.name() == &SmtpString::from(&b" root"[..])
                    } else {
                        false
                    }
                }),
            ),
        ];
        for (s, r) in tests.into_iter() {
            let b = Bytes::from(s);
            assert!(r(command(ByteSlice::from(&b)).unwrap().1));
        }
    }

    #[test]
    pub fn do_send_ok() {
        let mut v = Vec::new();
        Command::Vrfy(VrfyCommand::new((&b"fubar"[..]).into()))
            .send_to(&mut v)
            .unwrap();
        assert_eq!(v, b"VRFY fubar\r\n");
    }
}
