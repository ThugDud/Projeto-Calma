use chrono::{Datelike, NaiveDate, Weekday};

/// Verifica se um ano é bissexto.
pub fn is_leap_year(year: i32) -> bool {
    NaiveDate::from_ymd_opt(year, 2, 29).is_some()
}

/// Retorna a quantidade de dias em um mês dado.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(year, month + 1, 1)
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or(31)
}

/// Retorna o primeiro dia da semana de uma data.
pub fn first_day_of_month(year: i32, month: u32) -> Weekday {
    NaiveDate::from_ymd_opt(year, month, 1)
        .map(|d| d.weekday())
        .unwrap_or(Weekday::Mon)
}

/// Constrói a matriz do calendário mensal.
///
/// Cada linha é uma semana (começando em segunda-feira).
/// Dias fora do mês são representados como `None`.
pub fn month_matrix(year: i32, month: u32) -> Vec<Vec<Option<NaiveDate>>> {
    let first_weekday = first_day_of_month(year, month);
    let total_days = days_in_month(year, month);

    // Segunda-feira = 0, Domingo = 6 (num_days_from_monday)
    let offset = first_weekday.num_days_from_monday() as usize;

    let mut matrix: Vec<Vec<Option<NaiveDate>>> = Vec::new();
    let mut week: Vec<Option<NaiveDate>> = vec![None; offset];

    for day in 1..=total_days {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            week.push(Some(date));
        }
        if week.len() == 7 {
            matrix.push(week);
            week = Vec::new();
        }
    }

    if !week.is_empty() {
        week.resize(7, None);
        matrix.push(week);
    }

    matrix
}

/// Nomes dos dias da semana em português brasileiro, começando em segunda-feira.
pub const DIAS_SEMANA: [&str; 7] = ["Seg", "Ter", "Qua", "Qui", "Sex", "Sáb", "Dom"];

/// Nome completo do dia da semana em português brasileiro.
pub const DIAS_SEMANA_COMPLETO: [&str; 7] = [
    "Segunda-feira",
    "Terça-feira",
    "Quarta-feira",
    "Quinta-feira",
    "Sexta-feira",
    "Sábado",
    "Domingo",
];

/// Nomes dos meses em português brasileiro.
pub const MESES: [&str; 12] = [
    "Janeiro",
    "Fevereiro",
    "Março",
    "Abril",
    "Maio",
    "Junho",
    "Julho",
    "Agosto",
    "Setembro",
    "Outubro",
    "Novembro",
    "Dezembro",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2020));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2021));
        assert!(!is_leap_year(2023));
        assert!(is_leap_year(2024));
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2026, 1), 31);
        assert_eq!(days_in_month(2026, 2), 28);
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2026, 4), 30);
        assert_eq!(days_in_month(2026, 12), 31);
    }

    #[test]
    fn test_first_day_of_month() {
        assert_eq!(first_day_of_month(2026, 10), Weekday::Thu);
        assert_eq!(first_day_of_month(2026, 1), Weekday::Thu);
    }

    #[test]
    fn test_month_matrix_october_2026() {
        let matrix = month_matrix(2026, 10);
        let first_week = &matrix[0];
        assert_eq!(first_week.len(), 7);
        assert_eq!(first_week[0], None);
        assert_eq!(first_week[1], None);
        assert_eq!(first_week[2], None);
        assert_eq!(
            first_week[3],
            Some(NaiveDate::from_ymd_opt(2026, 10, 1).unwrap())
        );
        assert_eq!(
            first_week[4],
            Some(NaiveDate::from_ymd_opt(2026, 10, 2).unwrap())
        );
    }

    #[test]
    fn test_month_matrix_all_weeks_have_7_days() {
        let matrix = month_matrix(2026, 10);
        for week in &matrix {
            assert_eq!(week.len(), 7);
        }
    }

    #[test]
    fn test_month_matrix_february_leap() {
        let matrix = month_matrix(2024, 2);
        let total_days: u32 = matrix
            .iter()
            .flatten()
            .filter_map(|d| *d)
            .map(|d| d.day())
            .max()
            .unwrap();
        assert_eq!(total_days, 29);
    }

    #[test]
    fn test_month_matrix_february_not_leap() {
        let matrix = month_matrix(2023, 2);
        let total_days: u32 = matrix
            .iter()
            .flatten()
            .filter_map(|d| *d)
            .map(|d| d.day())
            .max()
            .unwrap();
        assert_eq!(total_days, 28);
    }

    #[test]
    fn test_dias_semana_count() {
        assert_eq!(DIAS_SEMANA.len(), 7);
        assert_eq!(DIAS_SEMANA[0], "Seg");
        assert_eq!(DIAS_SEMANA[6], "Dom");
    }

    #[test]
    fn test_meses_count() {
        assert_eq!(MESES.len(), 12);
        assert_eq!(MESES[0], "Janeiro");
        assert_eq!(MESES[9], "Outubro");
    }
}
