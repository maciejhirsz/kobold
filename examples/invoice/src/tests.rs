use crate::csv::update_csv_row_for_modified_table_cells;
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
}
