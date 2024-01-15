use std::cmp::max;

#[derive(Default, Debug, Clone)]
pub struct Table<'a> {
    row_len: Vec<usize>,
    rows: Vec<Vec<&'a str>>,
}

impl<'a> Table<'a> {
    pub fn push_row(&mut self, row: Vec<&'a str>) {
        if row.len() > self.row_len.len() {
            self.row_len.resize(row.len(), 0);
        }
        self.row_len
            .iter_mut()
            .zip(&row)
            .for_each(
                |(mut max_len, s)| *max_len = max(*max_len, s.chars().count())
            );
        self.rows.push(row);
    }

    pub fn to_string(&self, sep: &str) -> String {
        let pad = |s: &str, l: usize| {
            let cs = s.chars().count();
            if cs < l {
                let amnt = l - cs;
                format!("{}{}", s, &" ".repeat(amnt))
            } else {
                s.to_string()
            }
        };
        self.rows.iter().map(|row|
            row.iter()
                .zip(&self.row_len)
                .map(
                    |(s, l)|
                        format!("{}{sep}", pad(s, *l))
                ).collect::<Vec<_>>().join("")
        ).collect::<Vec<_>>().join("\n")
    }
}

