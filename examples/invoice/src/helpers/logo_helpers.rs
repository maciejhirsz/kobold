// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use log::{debug, error};
use heck::ToTitleCase;

use kobold::prelude::*;

use crate::state::State;

// fetch data from the first row of data for a table
pub fn get_row_value_for_label_for_table(label_to_search_for: &str, state: &State) -> String {
    let binding_source = &state.details.table.source.source;
    let binding = binding_source.find("\n");
    let first_newline_index = match &binding {
        Some(i) => i,
        None => panic!("must be a newline after the labels row in source"),
    };
    let labels_row_str = match binding_source.get(0..*first_newline_index) {
        Some(s) => s.to_string(),
        None => panic!("unable to get labels row from source"),
    };
    let labels_row_vec: Vec<&str> = labels_row_str.split(&[','][..]).collect();

    let col_org_name_idx = match labels_row_vec
        .iter()
        .position(|&label| label == label_to_search_for)
    {
        Some(i) => i,
        None => {
            error!("unable to find index of org_name label in labels");
            return "unknown".to_string();
        },
    };
    debug!("get_row_value_for_label_for_table get_text source {:?}", col_org_name_idx);
    let mut org_name = "".to_string();
    if col_org_name_idx <= (state.details.table.columns.len() - 1) {
        org_name = state
            .details
            .table
            .source
            .get_text(&state.details.table.rows[0][col_org_name_idx]).to_string();
    }
    let org_name_caption: String = format!("{}", org_name.to_title_case());
    org_name_caption
}
