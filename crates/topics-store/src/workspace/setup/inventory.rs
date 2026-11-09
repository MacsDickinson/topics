use super::*;
fn role(path: &str, roles: &[FolderRoleOverride]) -> (FolderRole, String) {
    if let Some(r) = roles
        .iter()
        .filter(|r| path == r.path || path.starts_with(&(r.path.clone() + "/")))
        .max_by_key(|r| r.path.len())
    {
        return (r.role, "Your editable folder role".into());
    }
    let lower = path.to_lowercase();
    let r = if lower.contains("people") || lower.contains("profile") || lower.contains("reference")
    {
        FolderRole::Reference
    } else if lower.contains("project") {
        FolderRole::Project
    } else if lower.contains("meeting") || lower.contains("transcript") {
        FolderRole::Meeting
    } else if lower.contains("log") || lower.contains("journal") {
        FolderRole::Log
    } else {
        FolderRole::Other
    };
    (
        r,
        if r == FolderRole::Other {
            "Unfamiliar layout; choose a role if useful"
        } else {
            "Suggested from folder name; editable, not a truth ranking"
        }
        .into(),
    )
}
fn overrides(roles: &[FolderRoleOverride]) -> WResult<()> {
    if roles.len() > 1000 {
        return Err(error("invalid_input", "Too many role overrides"));
    }
    for r in roles {
        relative(&r.path)?;
    }
    Ok(())
}
pub(super) fn inventory(ctx: &Context, roles: &[FolderRoleOverride]) -> WResult<SetupInventory> {
    overrides(roles)?;
    let state = load(ctx)?;
    let mut documents = vec![];
    let mut excluded = vec![];
    let mut records = vec![];
    let op = new_id();
    let mut total = 0;
    let mut walker = walkdir::WalkDir::new(&ctx.root)
        .follow_links(false)
        .into_iter();
    while let Some(entry) = walker.next() {
        let entry = entry.map_err(|_| {
            error(
                "invalid_input",
                "Cannot inventory folder; check local access",
            )
        })?;
        if entry.depth() == 0 {
            continue;
        }
        let path = entry
            .path()
            .strip_prefix(&ctx.root)
            .map_err(internal)?
            .to_str()
            .ok_or_else(|| error("invalid_input", "Source paths must be UTF-8"))?
            .replace(std::path::MAIN_SEPARATOR, "/");
        if entry.file_type().is_dir() {
            if entry.file_name().to_string_lossy().starts_with('.')
                || (entry.depth() == 1
                    && entry
                        .file_name()
                        .to_string_lossy()
                        .eq_ignore_ascii_case("topics"))
            {
                excluded.push(Exclusion {
                    path,
                    reason: "Hidden/config folder or app-owned records".into(),
                });
                walker.skip_current_dir();
            }
            continue;
        }
        if path.split('/').any(|p| p.starts_with('.')) {
            excluded.push(Exclusion {
                path,
                reason: "Hidden/config source".into(),
            });
            continue;
        }
        if entry.file_type().is_symlink() || !path.to_lowercase().ends_with(".md") {
            excluded.push(Exclusion {
                path,
                reason: "Source symlink or non-Markdown attachment".into(),
            });
            continue;
        }
        let bytes = match scopes::source_bytes(ctx, &path) {
            Ok(b) => b,
            Err(_) => {
                excluded.push(Exclusion {
                    path,
                    reason: "Unsafe, oversized or non-UTF-8 source".into(),
                });
                continue;
            }
        };
        let h = hash(&bytes);
        let id = scopes::source_identity(ctx, &state, &path, &h);
        let sensitivity = scopes::source_sensitivity(&state, &path, &h);
        let text = std::str::from_utf8(&bytes).map_err(internal)?;
        let parsed = crate::parse::document(&path, text, h.clone(), "0".into());
        let summary = SourceSummary {
            source_id: id.clone(),
            path: path.clone(),
            hash: h,
            date: parsed.date,
            sensitivity,
        };
        if !state.sources.contains_key(&id) {
            records.push((
                format!("topics/sources/{id}.md"),
                Record::new("source", &id, &op, &summary)?,
            ));
        }
        let (role, reason) = role(&path, roles);
        let size = bytes.len() as u64;
        if sensitivity == Sensitivity::LocalOnly {
            excluded.push(Exclusion {
                path: path.clone(),
                reason: "Local-only source; human inventory only".into(),
            });
        } else {
            total += size;
        }
        documents.push(InventoryDocument {
            source: summary,
            bytes: size,
            title: parsed.title,
            linked: text.contains("[[") || text.contains("]("),
            role,
            role_reason: reason,
        });
    }
    if !records.is_empty() {
        commit(
            ctx,
            "Register setup source inventory",
            records,
            CommitIntent {
                dependencies: vec![],
                tombstones: vec![],
                undo_of: None,
                expected: Some(&state.hashes),
                proposal: None,
            },
        )?;
    }
    documents.sort_by_key(|d| d.source.path.clone());
    let mut folders: BTreeMap<String, InventoryFolder> = BTreeMap::new();
    for d in &documents {
        let parent = d.source.path.rsplit_once('/').map(|(p, _)| p).unwrap_or("");
        let f = folders
            .entry(parent.into())
            .or_insert_with(|| InventoryFolder {
                path: parent.into(),
                role: d.role,
                reason: d.role_reason.clone(),
                documents: 0,
                bytes: 0,
                dated: 0,
                undated: 0,
                linked: 0,
                date_from: None,
                date_to: None,
            });
        f.documents += 1;
        f.bytes += d.bytes;
        f.linked += u64::from(d.linked);
        if let Some(date) = &d.source.date {
            f.dated += 1;
            if f.date_from.as_ref().is_none_or(|x| date < x) {
                f.date_from = Some(date.clone())
            }
            if f.date_to.as_ref().is_none_or(|x| date > x) {
                f.date_to = Some(date.clone())
            }
        } else {
            f.undated += 1
        }
    }
    let saved_topic_count = state.topics.len() as u64;
    let entry = if saved_topic_count > 0 {
        SetupEntry::Returning
    } else if documents.is_empty() {
        SetupEntry::Empty
    } else {
        SetupEntry::New
    };
    Ok(SetupInventory {
        schema: SCHEMA,
        root_id: ctx.root_id.clone(),
        entry,
        folders: folders.into_values().collect(),
        documents,
        excluded,
        eligible_bytes: total,
        saved_topic_count,
        resumable_job_ids: jobs(ctx)?
            .into_iter()
            .filter(|j| !matches!(j.status, SetupJobStatus::Finished))
            .map(|j| j.job_id)
            .collect(),
    })
}
fn limits(l: &SetupLimits) -> WResult<()> {
    if l.max_files_per_batch == 0
        || l.max_files_per_batch > 100
        || l.max_bytes_per_batch == 0
        || l.max_bytes_per_batch > 2 * 1024 * 1024
        || l.max_batches == 0
        || l.max_batches > 1000
        || l.max_candidates_per_turn == 0
        || l.max_candidates_per_turn > 200
        || l.max_turns_per_batch == 0
        || l.max_turns_per_batch > 32
    {
        return Err(error(
            "invalid_input",
            "Setup limits exceed bounded scope/turn budgets",
        ));
    }
    Ok(())
}
impl WorkspaceService {
    pub fn setup_inventory(&self, request: SetupInventoryRequest) -> WResult<SetupInventory> {
        self.with(&request.root_id, |ctx| inventory(ctx, &request.roles))
    }
    pub fn setup_manifest_preview(
        &self,
        request: SetupManifestPreviewRequest,
    ) -> WResult<SetupManifest> {
        self.with(&request.root_id,|ctx|{
 limits(&request.limits)?; validate_date_range(&request.spec.date_from,&request.spec.date_to)?;
 if request.runtime.trim().is_empty()||request.runtime.len()>256{return Err(error("invalid_input","Choose a configured runtime"))}
 for f in request.spec.files.iter().chain(&request.spec.folders){relative(f)?;}
 let inv=inventory(ctx,&request.roles)?;let mut excluded=inv.excluded;let mut selected=vec![];
 for d in inv.documents {
  if d.source.sensitivity==Sensitivity::LocalOnly{continue}
  let p=&d.source.path;
  let matches=(request.spec.files.is_empty()&&request.spec.folders.is_empty())||request.spec.files.contains(p)||request.spec.folders.iter().any(|f|p.starts_with(&(f.clone()+"/")));
  let date_matches=if request.spec.date_from.is_some()||request.spec.date_to.is_some(){d.source.date.as_ref().is_some_and(|date|request.spec.date_from.as_ref().is_none_or(|from|date>=from)&&request.spec.date_to.as_ref().is_none_or(|to|date<=to))}else{true};
  if !matches||!date_matches{excluded.push(Exclusion{path:p.clone(),reason:"Outside selection or inclusive document date filter".into()});continue}
  if d.bytes>request.limits.max_bytes_per_batch{return Err(error("invalid_input","A selected document exceeds the batch byte limit; exclude it or revise limits"))}
  selected.push(SetupManifestSource{source:d.source,bytes:d.bytes,role:d.role});
 }
 selected.sort_by_key(|s|(match s.role{FolderRole::Reference=>0,FolderRole::Project=>1,FolderRole::Meeting=>2,FolderRole::Log=>3,FolderRole::Other=>4},s.source.path.clone()));
 let mut batches=vec![];let mut sources=vec![];let mut bytes=0;
 for s in selected {if sources.len()>=request.limits.max_files_per_batch as usize||bytes+s.bytes>request.limits.max_bytes_per_batch{batches.push(SetupManifestBatch{batch_id:new_id(),sources:std::mem::take(&mut sources)});bytes=0;}bytes+=s.bytes;sources.push(s);}
 if !sources.is_empty(){batches.push(SetupManifestBatch{batch_id:new_id(),sources});}
 if batches.len()>request.limits.max_batches as usize{return Err(error("invalid_input","Selection exceeds the reviewed batch budget; narrow the selection"))}
 let mut manifest=SetupManifest{schema:SCHEMA,manifest_id:new_id(),root_id:ctx.root_id.clone(),fingerprint:String::new(),spec:request.spec,roles:request.roles,batches,excluded,limits:request.limits,runtime:request.runtime,created_at:now()};manifest.fingerprint=digest(&manifest)?;
 save_json(&setup_dir(ctx).join("manifests").join(format!("{}.json",manifest.manifest_id)),&manifest)?;Ok(manifest)
 })
    }
}
