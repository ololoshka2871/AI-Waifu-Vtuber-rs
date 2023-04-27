use num2words::Num2Words;
use regex::Regex;

pub fn convert_numbers2words(input: String) -> String {
    lazy_static::lazy_static! {
        static ref RE: Regex = Regex::new(r"\d+\.*\d+").unwrap();
    }

    RE.replace_all(&input, |caps: &regex::Captures| {
        let number_src = caps.get(0).unwrap().as_str();
        let number = match number_src.parse::<f32>() {
            Ok(v) => v,
            Err(_) => return number_src.to_string(),
        };
        let r_number = number.round();
        let res = match (number != r_number, Num2Words::new(r_number as i64).to_words()) {
            (true, Ok(number_in_words)) => format!("about {number_in_words}"),
            (false, Ok(number_in_words)) => number_in_words,
            (_, Err(_)) => number.to_string(),
        };

        tracing::trace!("Converted number \"{}\" -> \"{}\"", number_src, res);

        res
    })
    .to_string()
}
