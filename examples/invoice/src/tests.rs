use crate::{get_details_data, Details};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_details_data() -> Result<(), String> {
        let details = Details {
            inv_date: String::from("01.01.1970"),
            inv_no: String::from("0001"),
            from_attn_name: String::from("unknown"),
            from_org_name: String::from("unknown"),
            from_org_addr: String::from("unknown"),
            from_email: String::from("unknown"),
            to_attn_name: String::from("unknown"),
            to_title: String::from("unknown"),
            to_org_name: String::from("unknown"),
            to_email: String::from("unknown")
        };
        let arr: [(&str, &str); 10] = [
            ("inv_date","01.01.1970"),
            ("inv_no","0001"),
            ("from_attn_name","unknown"),
            ("from_org_name","unknown"),
            ("from_org_addr","unknown"),
            ("from_email","unknown"),
            ("to_attn_name","unknown"),
            ("to_title","unknown"),
            ("to_org_name","unknown"),
            ("to_email","unknown")
        ];
        let vec: Vec<(String, String)> = arr.iter().map(|x|
            (x.0.to_string(), x.1.to_string())
        ).collect();
        assert_eq!(
            get_details_data(&details),
            vec,
        );
        Ok(())
    }
}
