use std::ffi::CString;
use pyo3::prelude::*;
use pyo3::Python;

// TODO: This is temporary
const LOC: &str = r"D:\Documents\Programming\graphport\mwparserfromhell-rs\mwparserfromhell\src\mwparserfromhell";



fn parse(input: &str) -> Result<(), ()> {
    // import library from LOC
    Python::with_gil(|py| {
        py.run(
            CString::new(format!(
                "import sys; sys.path.append('{}'); import mwparserfromhell",
                LOC
            )).unwrap().as_c_str(),
            None,
            None,
        ).unwrap();
        let mwparserfromhell = py.import("mwparserfromhell").map_err(|e| e.to_string()).unwrap();
        dbg!(mwparserfromhell);
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = "== Header ==\nThis is a test.";
        let result = parse(input);
        assert!(result.is_ok());
    }
}
