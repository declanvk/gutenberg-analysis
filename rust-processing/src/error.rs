error_chain! {
    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
        Clap(::clap::Error);
        Send(::futures::channel::mpsc::SendError);
        Serde(::serde_json::Error);
        ParseInt(::std::num::ParseIntError);
    }

    errors {
        InvalidInput(n: String) {
            description("Invalid input"),
            display("Invalid input: '{}'", n),
        }
    }
}
