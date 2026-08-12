// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (alihaj [at] dtu.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The non-interactive protocol commands.
//!
//! The editor is the interface, but three things are better done from a shell
//! than from a TUI, because they belong in a script or a CI job rather than in
//! someone's hands:
//!
//! - `carp protocol sync` keeps the catalogue current, and can run unattended
//! - `carp protocol check` validates a protocol and exits non-zero when it is
//!   broken, which is what makes it usable as a pre-commit or CI check
//! - `carp protocol show` prints a protocol's shape without opening anything
//!
//! None of them need a CARP session, because none of them talk to CAWS.

use std::path::Path;

use color_eyre::Result;
use color_eyre::eyre::bail;
use carp_protocol::validate::Severity;

use crate::config::Config;

/// Download the upstream studies and rebuild the catalogue.
pub async fn sync(config: &Config) -> Result<()> {
    println!(
        "syncing from {}/{}…",
        carp_catalog::UPSTREAM_OWNER,
        carp_catalog::UPSTREAM_REPO
    );

    let report = carp_catalog::sync::sync(&config.data_dir).await?;
    println!("{}", report.summary());

    if let Some(version) = report.catalog.version.as_ref() {
        println!("  commit    {} - {}", version.commit.sha, version.commit.subject);
        println!("  dated     {}", version.commit.date);
    }
    println!(
        "  learned   {} measure types, {} health metrics, {} device classes",
        report.catalog.measure_types.len(),
        report.catalog.health_data_types.len(),
        report.catalog.device_types.len()
    );
    println!("  templates {}", report.catalog.templates.len());

    for skipped in &report.catalog.skipped {
        eprintln!("warning: could not read {skipped}");
    }
    Ok(())
}

/// Report what the stored catalogue holds, without touching the network.
pub async fn catalog_status(config: &Config) -> Result<()> {
    let catalog = match carp_catalog::sync::load(&config.data_dir).await {
        Ok(catalog) => catalog,
        Err(carp_catalog::Error::Missing) => {
            println!("no catalogue yet - run `carp protocol sync`");
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };

    match catalog.version.as_ref() {
        Some(version) => {
            println!("catalogue {}", version.label());
            println!("  from      {}", version.repository);
            println!("  commit    {}", version.commit.sha);
            println!("  subject   {}", version.commit.subject);
        }
        None => println!("catalogue present but unversioned"),
    }

    for (label, count) in [
        ("measure types", catalog.measure_types.len()),
        ("device classes", catalog.device_types.len()),
        ("health metrics", catalog.health_data_types.len()),
        ("input types", catalog.input_data_types.len()),
        ("templates", catalog.templates.len()),
    ] {
        println!("  {label:<15} {count}");
    }
    Ok(())
}

/// Validate a protocol, exiting non-zero when it has errors.
///
/// Warnings do not fail the check: they are legal protocols that are probably
/// a mistake, and a CI job that failed on them would be turned off.
pub fn check(path: &Path) -> Result<()> {
    let (protocol, resolved) = crate::studio::storage::read_checked(path)?;
    println!("{} - {}", protocol.name, protocol.summary());
    println!("{}", resolved.display());

    let diagnostics = carp_protocol::validate(&protocol);
    let (errors, warnings, notices) = carp_protocol::validate::counts(&diagnostics);

    for diagnostic in &diagnostics {
        println!(
            "  {:<7} {:<28} {}",
            diagnostic.severity.label(),
            diagnostic.location,
            diagnostic.message
        );
        if let Some(hint) = &diagnostic.hint {
            println!("          {hint}");
        }
    }

    println!("\n{errors} error(s), {warnings} warning(s), {notices} notice(s)");

    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        bail!("{} is not ready to deploy", resolved.display());
    }
    Ok(())
}

/// Print a protocol's shape: its devices, tasks and schedules.
///
/// A summary rather than the JSON, because `cat` already prints the JSON and
/// what is hard to see there is how the pieces connect.
pub fn show(path: &Path) -> Result<()> {
    let (protocol, resolved) = crate::studio::storage::read_checked(path)?;

    println!("{}", protocol.name);
    println!("  file      {}", resolved.display());
    println!("  id        {}", protocol.id);
    println!("  revision  {}", protocol.version);
    println!("  owner     {}", protocol.owner_id);
    if let Some(description) = &protocol.description {
        println!("  about     {description}");
    }

    println!("\ndevices");
    for device in protocol.devices() {
        let placement = if device.is_primary() { "primary" } else { "connected" };
        println!(
            "  {:<28} {:<24} {placement}",
            device.role_name(),
            device.type_label()
        );
    }

    println!("\ntasks");
    for task in &protocol.tasks {
        println!(
            "  {:<28} {:<24} {} measure(s)",
            task.name(),
            task.type_label(),
            task.measures().len()
        );
        for control in protocol.controls_for_task(task.name()) {
            let schedule = protocol
                .triggers
                .get(&control.trigger_id)
                .map(carp_protocol::Trigger::schedule_label)
                .unwrap_or_else(|| "a missing trigger".to_owned());
            println!("      {schedule}, on {}", control.destination_device_role_name);
        }
    }

    if !protocol.participant_roles.is_empty() {
        println!("\nparticipants");
        for role in &protocol.participant_roles {
            let asked: Vec<&str> = protocol
                .expected_participant_data
                .iter()
                .filter(|expected| {
                    expected
                        .assigned_to
                        .role_names()
                        .is_none_or(|names| names.iter().any(|name| *name == role.role))
                })
                .map(|expected| carp_protocol::node::short_type(expected.input_data_type()))
                .collect();
            println!("  {:<28} {}", role.role, asked.join(", "));
        }
    }

    let (errors, warnings, _) = carp_protocol::validate::counts(&carp_protocol::validate(&protocol));
    println!("\n{errors} error(s), {warnings} warning(s) - `carp protocol check` for detail");
    Ok(())
}
