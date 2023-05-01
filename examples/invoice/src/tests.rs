use crate::csv::{
    update_csv_row_for_modified_table_cells
};
use crate::helpers::{csv_helpers};
use crate::state::Text;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_csv_row_for_modified_table_cells() -> Result<(), String> {
        let cells: Vec<Text> = vec![
            Text::Owned("11task1".into()),
            Text::Insitu(27..29),
            Text::Insitu(30..40),
        ];
        let bindings = vec![
            "task1".to_string(),
            "10".to_string(),
            "0x000|h160".to_string(),
        ];
        let mut csv_row: Vec<&str> = vec![&bindings[0], &bindings[1], &bindings[2]];
        let actual: String = update_csv_row_for_modified_table_cells(&cells, &mut csv_row);
        let expected = "11task1,10,0x000|h160".to_string();

        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_pad_csv_data() -> Result<(), String> {
        // label row has 5 columns, but the data rows only have 4 columns of data
        let label_row = "description,total,qr,aaa".to_string();
        let row_1 = "task1,10,0x0|h160".to_string();
        let row_2 = "task2,20,0x1|h160".to_string();
        let original_csv = vec![label_row.as_str(), row_1.as_str(), row_2.as_str()];
        let mut new_csv: Vec<Vec<&str>> = vec![];
        let mut new_csv_lens: Vec<usize> = vec![];
        // add padding "" to a 5th column for the data rows so all rows have same qty of columns
        let padding = "".to_string();
        csv_helpers::pad_csv_data(&original_csv, &mut new_csv, &mut new_csv_lens, &padding);

        let label_row_vec: Vec<&str> = label_row.split(&[','][..]).collect();
        let mut row_1_vec: Vec<&str> = row_1.split(&[','][..]).collect();
        let mut row_2_vec: Vec<&str> = row_2.split(&[','][..]).collect();
        row_1_vec.push(&padding);
        row_2_vec.push(&padding);

        let actual: Vec<Vec<&str>> = new_csv.clone();
        let expected = vec![label_row_vec, row_1_vec, row_2_vec];

        assert_eq!(actual, expected);
        Ok(())
    }
}
