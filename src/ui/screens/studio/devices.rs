// Copyright (c) 2026 Copenhagen Research Platform
// All rights reserved.
//
// Authors: Alireza Hajebrahimi (support@carp.dk)
//
// This file is part of CARP CLI.
// Unauthorized copying, modification, or distribution is prohibited.

//! The Devices tab: what the protocol collects data from.

use carp_protocol::Device;
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::Row;

use crate::studio::Studio;
use crate::ui::theme;
use crate::ui::widgets::{detail, master_detail, table};

/// Draw the devices list and the panel describing the selected one.
pub fn render(frame: &mut Frame, area: Rect, studio: &mut Studio) {
    let (list_area, detail_area) = master_detail(area, 3, 2);

    let devices: Vec<&Device> = studio.protocol.devices().collect();
    let title = format!(
        "devices {}",
        table::position_label(&studio.lists.devices, devices.len())
    );

    if devices.is_empty() {
        frame.render_widget(
            table::placeholder("no devices yet - press a to add one", theme::block(&title)),
            list_area,
        );
    } else {
        let rows: Vec<Row> = devices
            .iter()
            .map(|device| {
                let role = if device.is_primary() {
                    "primary"
                } else {
                    "connected"
                };
                Row::new(vec![
                    Line::styled(device.role_name().to_owned(), theme::value()),
                    Line::styled(device.type_label().to_owned(), theme::label()),
                    Line::styled(
                        role.to_owned(),
                        if device.is_primary() {
                            theme::ok()
                        } else {
                            theme::dim()
                        },
                    ),
                ])
            })
            .collect();

        let widths = vec![
            Constraint::Fill(2),
            Constraint::Fill(2),
            Constraint::Length(10),
        ];
        let list = table::table(
            table::header(["role name", "type", "role"]),
            rows,
            widths,
            theme::focused_block(&title),
        );
        table::render(
            frame,
            list_area,
            list,
            &mut studio.lists.devices,
            devices.len(),
        );
    }

    let Some(detail_area) = detail_area else {
        return;
    };
    let block = theme::block("device");
    match studio.lists.selected_device(&studio.protocol) {
        Some(device) => {
            frame.render_widget(
                detail::panel(block, device_lines(device, studio)),
                detail_area,
            );
        }
        None => frame.render_widget(detail::empty(block, "no device selected"), detail_area),
    }
}

/// The selected device's settings, its connection, and what uses it.
fn device_lines(device: &Device, studio: &Studio) -> Vec<Line<'static>> {
    let protocol = &studio.protocol;
    let mut lines = vec![
        detail::field("role name", device.role_name().to_owned()),
        detail::field("type", device.type_label().to_owned()),
        detail::field(
            "placement",
            if device.is_primary() {
                "primary - runs the study"
            } else {
                "connected - reached through a phone"
            },
        ),
        detail::field("optional", if device.is_optional() { "yes" } else { "no" }),
    ];

    if !device.is_primary() {
        let connection = protocol
            .connections
            .iter()
            .find(|connection| connection.role_name == device.role_name());
        lines.push(detail::field_styled(
            "connected to",
            connection
                .map(|connection| connection.connected_to_role_name.clone())
                .unwrap_or_else(|| "nothing - unreachable".to_owned()),
            if connection.is_some() {
                theme::value()
            } else {
                theme::error()
            },
        ));
    }

    // Which triggers fire on this device, and which tasks run on it: the two
    // questions someone asks before deleting one.
    let triggers = protocol
        .triggers
        .values()
        .filter(|trigger| trigger.source_device() == device.role_name())
        .count();
    let tasks: Vec<&str> = protocol
        .task_controls
        .iter()
        .filter(|control| control.destination_device_role_name == device.role_name())
        .map(|control| control.task_name.as_str())
        .collect();

    lines.push(detail::blank());
    lines.push(detail::section("used by"));
    lines.push(detail::field(
        "triggers",
        format!(
            "{triggers} fire{} here",
            if triggers == 1 { "s" } else { "" }
        ),
    ));
    if tasks.is_empty() {
        lines.push(detail::note("  no tasks run on this device"));
    } else {
        for task in tasks {
            lines.push(detail::bullet(task.to_owned(), theme::value()));
        }
    }

    // Sampling defaults apply to every task on the device, so they belong
    // here rather than on each measure.
    if let Some(sampling) = device.sampling()
        && !sampling.is_empty()
    {
        lines.push(detail::blank());
        lines.push(detail::section("default sampling"));
        for (measure, configuration) in sampling {
            lines.push(detail::field(
                carp_protocol::node::short_type(measure),
                configuration.label(),
            ));
        }
    }

    if matches!(device, Device::Unknown(_)) {
        lines.push(detail::blank());
        lines.push(detail::note(
            "This device class is newer than this build. It is kept exactly as \
             it was, but its settings cannot be shown here.",
        ));
    }

    lines
}
