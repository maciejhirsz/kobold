use log::debug;

// incase there are more columns in some rows than others we will pad them for processing
pub fn pad_csv_data<'a>(
    original_csv: &Vec<&'a str>,
    new_csv: &mut Vec<Vec<&'a str>>,
    new_csv_lens: &mut Vec<usize>,
    padding: &'a str,
) {
    let mut old_csv: Vec<Vec<&str>> = vec![];
    let mut old_csv_lens: Vec<usize> = vec![];
    original_csv
        .into_iter()
        .enumerate()
        .for_each(|(i, row_data)| {
            let data: Vec<&str> = row_data.split(",").collect();
            old_csv.push(data.clone());
            old_csv_lens.push(data.len());
        });

    let old_csv_lens_most_columns = old_csv_lens.iter().max().unwrap();
    debug!("old_csv {:?}", old_csv);
    debug!("old_csv_lens_most_columns {:?}", old_csv_lens_most_columns);

    old_csv.into_iter().enumerate().for_each(|(i, row_data)| {
        debug!("row_data {:?}", row_data);
        let mut data = row_data.clone();
        let mut data_len = &data.len();

        // incase the uploaded data has an extra column on the right with only
        // a label with cell data but no data for the other rows in that column,
        // e.g. "description,total,qr,aaa\neat,1,0x0,\nsleep,2,0x1,"
        // then we need to manually add the extra row values here so we don't
        // get index out of bounds error when swapping values in
        // function `update_csv_row_for_modified_table_cells`
        if &data_len < &old_csv_lens_most_columns {
            // resize to add padding to this row_data with empty string "" so
            // has the same as the longest length
            data.resize(*old_csv_lens_most_columns, &padding);
        }
        // create longer lived data length value
        let mut data_len = &data.len(); // update after resize
        debug!("data {:?}", &data);

        new_csv.push(data);
        new_csv_lens.push(*data_len);
    });
    debug!("new_csv {:?}", new_csv);
    debug!("new_csv_lens {:?}", new_csv_lens);
}
