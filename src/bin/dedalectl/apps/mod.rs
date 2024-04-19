use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[clap(
    visible_alias = "app", 
    long_about,
    verbatim_doc_comment,
)]
/// The APPS commands focus on managing your Dedale applications. Start with the CREATE command to register your application.
/// The LIST command will list all currently registered applications.
pub(super) enum Command {
    Create,
    List,
    Destroy
}