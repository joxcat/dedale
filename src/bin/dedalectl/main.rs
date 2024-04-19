use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
/// This is dedalectl, the Fly.. **cough** poor man command line interface to reach the sun.
struct App {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Parser)]
enum Command {
    #[clap(subcommand, about = "Manage apps")]
    Apps(apps::Command),
    #[clap(subcommand, about = "Manage machines")]
    Machine(machine::Command),
}

mod apps {
    use clap::Parser;

    #[derive(Debug, Clone, Parser)]
    #[clap(
        visible_alias = "app", 
        long_about,
        verbatim_doc_comment,
    )]
    /// The APPS commands focus on managing your Dedale applications. Start with the CREATE command to register your application.
    /// The LIST command will list all currently registered applications.
    pub enum Command {
        Create,
        List,
        Destroy
    }
}

mod machine {
    use clap::Parser;

    #[derive(Debug, Clone, Parser)]
    #[clap(visible_aliases = ["machines", "m"], verbatim_doc_comment)]
    /// Manage Dedale Machines. 
    /// Dedale Machines are super-slow, or at least not lighting fast VMs that can be created, and then "quickly" started and stopped as needed with dedalectl commands or with the Machines REST dedale.
    pub enum Command {
        Create,
        List,
        Destroy,
        Stop,
        Start,
    }
}

fn main() {
    let _app = App::parse();
}
