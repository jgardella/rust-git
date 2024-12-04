use crate::{command::GitCommand, object::id::GitObjectId, repo::RepoState, RustGitError};

use super::cli::TagArgs;

pub(crate) struct CreateTag {
    tag_name: String,
    object_id: Option<GitObjectId>,
    force: bool,
    message: Option<String>,
}

pub(crate) struct DeleteTag {
    tag_name: String,
}

pub(crate) enum TagCommand {
    ListTags(),
    CreateTag(CreateTag),
    DeleteTag(DeleteTag),
}

impl TagCommand {
    pub fn new(args: TagArgs) -> Result<TagCommand, RustGitError> {
        match (args.tag_name, args.object) {
            (Some(tag_name), Some(object_id)) => {
                let object_id = GitObjectId::new(object_id);

                Ok(TagCommand::CreateTag(CreateTag {
                    tag_name,
                    object_id: Some(object_id),
                    force: args.force,
                    message: args.message,
                }))
            }
            (Some(tag_name), None) => {
                if args.delete {
                    Ok(TagCommand::DeleteTag(DeleteTag { tag_name }))
                } else {
                    Ok(TagCommand::CreateTag(CreateTag {
                        tag_name,
                        object_id: None,
                        force: args.force,
                        message: args.message,
                    }))
                }
            }
            (None, None) => Ok(TagCommand::ListTags()),
            (None, Some(_)) => Err(RustGitError::new("tag_name required when object provided")),
        }
    }
}

impl GitCommand for TagCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError> {
        let repo = repo_state.try_get()?;

        match self {
            TagCommand::CreateTag(create_cmd) => {
                if !create_cmd.force && repo.refs.try_read_tag(&create_cmd.tag_name)?.is_some() {
                    return Err(RustGitError::new(format!(
                        "cannot overwrite existing tag {}",
                        create_cmd.tag_name
                    )));
                }

                let object_id = if let Some(object_id) = &create_cmd.object_id {
                    object_id
                } else {
                    if let (_, Some(head_ref)) = repo.refs.get_head_ref()? {
                        &head_ref.clone()
                    } else {
                        return Err(RustGitError::new("no HEAD ref"));
                    }
                };

                if let Some(message) = &create_cmd.message {
                    return repo.create_annotated_tag(&create_cmd.tag_name, &object_id, &message);
                }

                return repo.create_lightweight_tag(&create_cmd.tag_name, &object_id);
            }
            TagCommand::ListTags() => {
                let tags = repo.refs.list_tags()?;
                for tag in tags {
                    println!("{}", tag)
                }
            }
            TagCommand::DeleteTag(delete_cmd) => repo.refs.delete_tag(&delete_cmd.tag_name)?,
        }

        Ok(())
    }
}
