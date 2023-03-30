use serenity::{
    all::CommandInteraction,
    async_trait,
    builder::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage, CreateAttachment, CreateInteractionResponseFollowup},
    prelude::Context,
};
use zip::write::FileOptions;

use crate::state::AppState;
use super::{command::Command, util::{CommandResponse, FailureMessageKind}};
use tokio::{process, task::spawn_blocking};
use std::{fs::File, io::{Read, Write}};

const TARGET_REPO_ON_SYSTEM: &str = "/home/josiah/external/capstone-project-team-4";
const OUTPUT_DIR: &str = "/tmp/artifacts/1/project-proposal";

pub struct GenerateCommand;
impl<'a> TryFrom<&'a CommandInteraction> for GenerateCommand {
    type Error = String;
    fn try_from(_: &'a CommandInteraction) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

#[async_trait]
impl<'a> Command<'a> for GenerateCommand {
    fn name() -> &'static str {
        "generate"
    }

    fn description() -> &'static str {
        "Generate the current report and send it as a response to this message"
    }

    fn get_application_command_options(i: CreateCommand) -> CreateCommand {
        i
    }

    #[allow(clippy::invisible_characters)]
    async fn handle_application_command<'b>(
        self,
        interaction: &'b CommandInteraction,
        _: &'b AppState,
        context: &'b Context,
    ) -> Result<CommandResponse, CommandResponse> {
        println!("Sending loading response...");
        if let Err(e) = interaction
            .create_response(&context, CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::default()
                    .content("Generating report...")
            )).await {
            return Err(CommandResponse::ComplexFailure{
                response: String::from("Failed to send loading response"),
                kind: FailureMessageKind::Error,
                log_message: format!("Failed to send loading response: {}", e),
            });
            }

        println!("Generating report command started.");
        // run "git pull" in the target repo on the system
        let mut git_pull = process::Command::new("git");
        let output = git_pull
            .arg("pull")
            .current_dir(TARGET_REPO_ON_SYSTEM)
            .output()
            .await
            .expect("failed to execute git pull");

        println!("git pull output: {}", String::from_utf8_lossy(&output.stdout));

        // if output does NOT contain "Already up to date.", then we need to generate the report
        // "act -j compile --artifact-server-path /tmp/artifacts -W .github/workflows/build_proposal.yml"
        if !String::from_utf8_lossy(&output.stdout).contains("Already up to date.") || true {
            println!("Generating report...");
            // run the generation command
            let mut act = process::Command::new("act");
            let output = act
                .arg("-j")
                .arg("compile")
                .arg("--artifact-server-path")
                .arg("/tmp/artifacts")
                .arg("-W")
                .arg(".github/workflows/build_proposal.yml")
                .current_dir(TARGET_REPO_ON_SYSTEM)
                .output()
                .await
                .expect("failed to execute act");

            println!("act output: {}", String::from_utf8_lossy(&output.stdout));

            // if failed to run - return error
            if !output.status.success() {
                // return Err(CommandResponse::ComplexFailure{
                //     response: String::from("Failed to gene rate report"),
                //     kind: FailureMessageKind::Error,
                //     log_message: format!("Failed to generate report: {}", String::from_utf8_lossy(&output.stderr)),
                // });

                // put stderr output into a .txt file
                let mut file = File::create("/tmp/report_error.txt").unwrap();
                file.write_all(&output.stderr).unwrap();

                // create followup
                interaction
                    .create_followup(&context, CreateInteractionResponseFollowup::default()
                        .content("Failed to generate report")
                        .add_file(CreateAttachment::path("/tmp/report_error.txt").await.unwrap())
                        .add_file(CreateAttachment::path("/tmp/artifacts/1/project-proposal/full.log.gz__").await.unwrap())
                    ).await
                    .unwrap();

                // delete the file
                std::fs::remove_file("/tmp/report_error.txt").unwrap();

                // return
                return Ok(CommandResponse::NoResponse);
            }

            println!("Report generated successfully.");
        }

        // the OUTPUT_DIR contains the reports which are all individually compressed with .gz
        // we need to unzip them and then zip them all up into a single file

        spawn_blocking(move || {
            let dir = std::fs::read_dir(OUTPUT_DIR).unwrap();
            for entry in dir {
                let entry = entry.unwrap();
                let path = entry.path();
                let name = path.file_name().unwrap().to_str().unwrap();
                if name.ends_with(".gz__") {
                    let mut file = File::open(&path).unwrap();
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer).unwrap();
                    let mut decoder = flate2::read::GzDecoder::new(&buffer[..]);
                    let mut output = Vec::new();
                    decoder.read_to_end(&mut output).unwrap();

                    // the actual extension is right before the .gz__, and will be either .log or .pdf
                    let extension;
                    if name.ends_with(".log.gz__") {
                        extension = "log";
                    } else if name.ends_with(".pdf.gz__") {
                        extension = "pdf";
                    } else {
                        panic!("Unknown extension: {}", name);
                    }

                    let mut file = File::create(OUTPUT_DIR.to_owned() + "/" + &name[..name.len() - 8] + extension).unwrap();
                    file.write_all(&output).unwrap();

                    // delete the original file
                    std::fs::remove_file(&path).unwrap();
                }
            }
        }).await.unwrap();

        println!("Zipping report...");

        // zip up all files in the output directory to a temporary location
        let temp_dir = tempfile::Builder::new()
            .prefix("report")
            .tempdir_in("/tmp").unwrap();

        let temp_path = temp_dir.path();
        let zip_path = temp_path.join("report.zip");
        // blocking
        let zip_path = spawn_blocking(move || {


            let file = File::create(&zip_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let options = FileOptions::default()
                .compression_method(zip::CompressionMethod::Bzip2)
                .unix_permissions(0o755);
            let dir = std::fs::read_dir(OUTPUT_DIR).unwrap();
            for entry in dir {
                let entry = entry.unwrap();
                let path = entry.path();
                let name = path.file_name().unwrap().to_str().unwrap();

                // if name != "full.pdf" {
                //     continue;
                // }

                zip.start_file(name, options).unwrap();
                let mut file = File::open(path).unwrap();
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).unwrap();
                zip.write_all(&buffer).unwrap();
            }
            zip.finish().unwrap();
            zip_path
        }).await.unwrap();

        println!("Report zipped successfully.");

        let zipped_file = tokio::fs::File::open(&zip_path).await.unwrap();

        // "Project Proposal - {TIME}.pdf"
        let filename = format!("Project Proposal - {}.bz2", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));

        // print path to file
        println!("Zipped file path: {}", zip_path.display());

        // forget the temp dir
        std::mem::forget(temp_dir);

        if let Err(e) = interaction.create_followup(&context, CreateInteractionResponseFollowup::default()
            .content("Here are your generated reports")
            .add_file(CreateAttachment::file(&zipped_file, filename).await.expect("failed to create attachment"))
        ).await {
            return Err(CommandResponse::ComplexFailure{
                response: String::from("Failed to send report"),
                kind: FailureMessageKind::Error,
                log_message: format!("Failed to send report: {}", e),
            });
        }

        Ok(CommandResponse::NoResponse)
    }
}
