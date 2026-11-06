use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Weekday};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTaskInput {
    pub title: String,
    pub due_date: Option<NaiveDate>,
}

/// Tenta interpretar uma expressão de data em português brasileiro com relação a uma data base (`today`).
pub fn parse_pt_date(text: &str, today: NaiveDate) -> Option<NaiveDate> {
    let s = text.trim().to_lowercase();

    match s.as_str() {
        "hoje" => return Some(today),
        "amanhã" | "amanha" => return Some(today + Duration::days(1)),
        "depois de amanhã" | "depois de amanha" => return Some(today + Duration::days(2)),
        _ => {}
    }

    // Dias da semana em português
    let target_weekday = match s.as_str() {
        "segunda" | "segunda-feira" | "seg" => Some(Weekday::Mon),
        "terça" | "terca" | "terça-feira" | "terca-feira" | "ter" => Some(Weekday::Tue),
        "quarta" | "quarta-feira" | "qua" => Some(Weekday::Wed),
        "quinta" | "quinta-feira" | "qui" => Some(Weekday::Thu),
        "sexta" | "sexta-feira" | "sex" => Some(Weekday::Fri),
        "sábado" | "sabado" | "sab" => Some(Weekday::Sat),
        "domingo" | "dom" => Some(Weekday::Sun),
        _ => None,
    };

    if let Some(target) = target_weekday {
        let mut d = today + Duration::days(1);
        while d.weekday() != target {
            d += Duration::days(1);
        }
        return Some(d);
    }

    // Padrão: "dia X" (ex: "dia 5", "dia 25")
    if s.starts_with("dia ") {
        let num_str = s.trim_start_matches("dia ").trim();
        if let Ok(day_num) = num_str.parse::<u32>() {
            return next_day_of_month(today, day_num);
        }
    }

    // Padrão DD/MM/AAAA ou DD/MM
    if s.contains('/') {
        let parts: Vec<&str> = s.split('/').map(|p| p.trim()).collect();
        if parts.len() == 3 {
            if let (Ok(d), Ok(m), Ok(y)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<i32>(),
            ) {
                return NaiveDate::from_ymd_opt(y, m, d);
            }
        } else if parts.len() == 2 {
            if let (Ok(d), Ok(m)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                // Ano atual; se já passou, assume próximo ano
                let current_year = today.year();
                if let Some(candidate) = NaiveDate::from_ymd_opt(current_year, m, d) {
                    if candidate >= today {
                        return Some(candidate);
                    } else {
                        return NaiveDate::from_ymd_opt(current_year + 1, m, d);
                    }
                }
            }
        }
    }

    // Padrão AAAA-MM-DD
    if let Ok(d) = NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
        return Some(d);
    }

    None
}

/// Encontra o próximo dia do mês dado (este mês se ainda não passou, ou próximo mês).
fn next_day_of_month(today: NaiveDate, target_day: u32) -> Option<NaiveDate> {
    if !(1..=31).contains(&target_day) {
        return None;
    }
    // Tenta no mês atual
    if let Some(candidate) =
        NaiveDate::from_ymd_opt(today.year(), today.month(), target_day).filter(|&c| c >= today)
    {
        return Some(candidate);
    }

    // Tenta no próximo mês
    let (next_year, next_month) = if today.month() == 12 {
        (today.year() + 1, 1)
    } else {
        (today.year(), today.month() + 1)
    };

    NaiveDate::from_ymd_opt(next_year, next_month, target_day)
}

/// Interpreta uma entrada rápida em linguagem natural (ex: "Estudar C amanhã" -> "Estudar C", Amanhã).
pub fn parse_quick_task(input: &str, today: NaiveDate) -> ParsedTaskInput {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return ParsedTaskInput {
            title: String::new(),
            due_date: None,
        };
    }

    // Verifica sufixos conhecidos (ex: " amanhã", " hoje", " sexta", " dia 5", " 12/11/2026")
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    if words.len() > 1 {
        // Tenta os últimos 2 tokens (ex: "dia 5", "depois de amanhã")
        if words.len() >= 3 {
            let last_three = words[words.len() - 3..].join(" ");
            if let Some(date) = parse_pt_date(&last_three, today) {
                let title = words[..words.len() - 3].join(" ");
                return ParsedTaskInput {
                    title,
                    due_date: Some(date),
                };
            }
        }

        if words.len() >= 2 {
            let last_two = words[words.len() - 2..].join(" ");
            if let Some(date) = parse_pt_date(&last_two, today) {
                let title = words[..words.len() - 2].join(" ");
                return ParsedTaskInput {
                    title,
                    due_date: Some(date),
                };
            }
        }

        // Tenta o último token
        let last_word = words[words.len() - 1];
        if let Some(date) = parse_pt_date(last_word, today) {
            let title = words[..words.len() - 1].join(" ");
            return ParsedTaskInput {
                title,
                due_date: Some(date),
            };
        }
    }

    ParsedTaskInput {
        title: trimmed.to_string(),
        due_date: None,
    }
}

/// Resultado da interpretação de um evento rápido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEventInput {
    pub title: String,
    pub date: NaiveDate,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
}

/// Interpreta uma entrada rápida de evento.
///
/// A data é opcional (padrão: hoje) e o horário pode ser informado no final
/// nos formatos `HH:MM`, `HHh` ou `HHhMM`.
pub fn parse_quick_event(input: &str, today: NaiveDate) -> ParsedEventInput {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return ParsedEventInput {
            title: String::new(),
            date: today,
            start_time: None,
            end_time: None,
        };
    }

    let mut words: Vec<&str> = trimmed.split_whitespace().collect();

    // Horário opcional no final: "14:30", "14h", "14:30-16:00", "14h-16h".
    let mut start_time = None;
    let mut end_time = None;
    if let Some(last) = words.last().copied() {
        if let Some((start, end)) = parse_time_range(last) {
            start_time = Some(start);
            end_time = Some(end);
            words.pop();
        } else if let Some(time) = parse_time(last) {
            start_time = Some(time);
            words.pop();
        }
    }

    let remaining = words.join(" ");
    let parsed = parse_quick_task(&remaining, today);

    // Se a entrada inteira for apenas uma data (ex.: "amanhã", "12/11/2026"),
    // não use a data como título do evento.
    let (title, date) = if let Some(due) = parsed.due_date {
        (parsed.title, due)
    } else {
        match parse_pt_date(&remaining, today) {
            Some(d) => (String::new(), d),
            None => (parsed.title, today),
        }
    };

    ParsedEventInput {
        title,
        date,
        start_time,
        end_time,
    }
}

/// Interpreta um intervalo de horário como `14:30-16:00` ou `14h-16h`.
pub fn parse_time_range(text: &str) -> Option<(NaiveTime, NaiveTime)> {
    let (start, end) = text.split_once('-')?;
    let start = parse_time(start)?;
    let end = parse_time(end)?;
    if end <= start {
        return None;
    }
    Some((start, end))
}

/// Interpreta um horário nos formatos `HH:MM`, `HH:MM:SS`, `HHh` ou `HHhMM`.
pub fn parse_time(text: &str) -> Option<NaiveTime> {
    let s = text.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }

    let (hour_str, minute_str) = if let Some((h, m)) = s.split_once('h') {
        (h, if m.is_empty() { "0" } else { m })
    } else {
        let parts: Vec<&str> = s.split(':').collect();
        match parts.as_slice() {
            [h, m] => (*h, *m),
            [h, m, _sec] => (*h, *m),
            _ => return None,
        }
    };

    let hour: u32 = hour_str.parse().ok()?;
    let minute: u32 = minute_str.parse().ok()?;
    if hour > 23 || minute > 59 {
        return None;
    }
    NaiveTime::from_hms_opt(hour, minute, 0)
}

/// Formata a descrição amigável do prazo (Seção 9).
pub fn format_due_label(
    due_date: Option<NaiveDate>,
    today: NaiveDate,
    is_completed: bool,
) -> (String, &'static str) {
    if is_completed {
        return ("Concluída".to_string(), "completed");
    }

    let Some(due) = due_date else {
        return ("Sem prazo".to_string(), "muted");
    };

    let days = due.signed_duration_since(today).num_days();

    if days < 0 {
        let abs_days = days.abs();
        if abs_days == 1 {
            ("Atrasada há 1 dia".to_string(), "overdue")
        } else {
            (format!("Atrasada há {} dias", abs_days), "overdue")
        }
    } else if days == 0 {
        ("Vence hoje".to_string(), "due_today")
    } else if days == 1 {
        ("Falta 1 dia".to_string(), "due_soon")
    } else {
        (format!("Faltam {} dias", days), "normal")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pt_date_relative() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 27).unwrap();

        assert_eq!(parse_pt_date("hoje", today), Some(today));
        assert_eq!(
            parse_pt_date("amanhã", today),
            Some(NaiveDate::from_ymd_opt(2026, 10, 28).unwrap())
        );
        assert_eq!(
            parse_pt_date("amanha", today),
            Some(NaiveDate::from_ymd_opt(2026, 10, 28).unwrap())
        );
        assert_eq!(
            parse_pt_date("depois de amanhã", today),
            Some(NaiveDate::from_ymd_opt(2026, 10, 29).unwrap())
        );
    }

    #[test]
    fn test_parse_pt_date_weekdays() {
        // 2026-10-27 é terça-feira
        let today = NaiveDate::from_ymd_opt(2026, 10, 27).unwrap();
        assert_eq!(today.weekday(), Weekday::Tue);

        // Próxima quarta é amanhã (28)
        assert_eq!(
            parse_pt_date("quarta", today),
            Some(NaiveDate::from_ymd_opt(2026, 10, 28).unwrap())
        );
        // Próxima sexta é dia 30
        assert_eq!(
            parse_pt_date("sexta", today),
            Some(NaiveDate::from_ymd_opt(2026, 10, 30).unwrap())
        );
    }

    #[test]
    fn test_parse_pt_date_explicit() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 27).unwrap();

        assert_eq!(
            parse_pt_date("12/11/2026", today),
            Some(NaiveDate::from_ymd_opt(2026, 11, 12).unwrap())
        );
        assert_eq!(
            parse_pt_date("dia 5", today),
            Some(NaiveDate::from_ymd_opt(2026, 11, 5).unwrap())
        );
        assert_eq!(
            parse_pt_date("dia 29", today),
            Some(NaiveDate::from_ymd_opt(2026, 10, 29).unwrap())
        );
    }

    #[test]
    fn test_parse_quick_task() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 27).unwrap();

        let t1 = parse_quick_task("Estudar C amanhã", today);
        assert_eq!(t1.title, "Estudar C");
        assert_eq!(
            t1.due_date,
            Some(NaiveDate::from_ymd_opt(2026, 10, 28).unwrap())
        );

        let t2 = parse_quick_task("Pagar internet dia 5", today);
        assert_eq!(t2.title, "Pagar internet");
        assert_eq!(
            t2.due_date,
            Some(NaiveDate::from_ymd_opt(2026, 11, 5).unwrap())
        );

        let t3 = parse_quick_task("Comprar comida", today);
        assert_eq!(t3.title, "Comprar comida");
        assert_eq!(t3.due_date, None);
    }

    #[test]
    fn test_format_due_label() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 27).unwrap();

        let (l1, c1) = format_due_label(Some(today), today, false);
        assert_eq!(l1, "Vence hoje");
        assert_eq!(c1, "due_today");

        let (l2, c2) = format_due_label(Some(today - Duration::days(3)), today, false);
        assert_eq!(l2, "Atrasada há 3 dias");
        assert_eq!(c2, "overdue");

        let (l3, c3) = format_due_label(Some(today - Duration::days(3)), today, true);
        assert_eq!(l3, "Concluída");
        assert_eq!(c3, "completed");

        let (l4, c4) = format_due_label(Some(today + Duration::days(2)), today, false);
        assert_eq!(l4, "Faltam 2 dias");
        assert_eq!(c4, "normal");
    }

    #[test]
    fn test_parse_time() {
        assert_eq!(parse_time("14:30"), NaiveTime::from_hms_opt(14, 30, 0));
        assert_eq!(parse_time("14h"), NaiveTime::from_hms_opt(14, 0, 0));
        assert_eq!(parse_time("14h30"), NaiveTime::from_hms_opt(14, 30, 0));
        assert_eq!(parse_time("9:05"), NaiveTime::from_hms_opt(9, 5, 0));
        assert_eq!(parse_time("25:00"), None);
        assert_eq!(parse_time("abc"), None);
        assert_eq!(parse_time("14"), None);
        assert_eq!(
            parse_time_range("14:30-16:00"),
            Some((
                NaiveTime::from_hms_opt(14, 30, 0).unwrap(),
                NaiveTime::from_hms_opt(16, 0, 0).unwrap()
            ))
        );
        assert_eq!(parse_time_range("16:00-14:00"), None);
    }

    #[test]
    fn test_parse_quick_event() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 27).unwrap();

        let e1 = parse_quick_event("Reunião amanhã 14:30", today);
        assert_eq!(e1.title, "Reunião");
        assert_eq!(e1.date, NaiveDate::from_ymd_opt(2026, 10, 28).unwrap());
        assert_eq!(e1.start_time, NaiveTime::from_hms_opt(14, 30, 0));

        let e2 = parse_quick_event("Consulta médica dia 5", today);
        assert_eq!(e2.title, "Consulta médica");
        assert_eq!(e2.date, NaiveDate::from_ymd_opt(2026, 11, 5).unwrap());
        assert_eq!(e2.start_time, None);

        let e3 = parse_quick_event("Academia 7h", today);
        assert_eq!(e3.title, "Academia");
        assert_eq!(e3.date, today);
        assert_eq!(e3.start_time, NaiveTime::from_hms_opt(7, 0, 0));

        // Entrada contendo apenas uma data não deve virar título.
        let e4 = parse_quick_event("amanhã", today);
        assert_eq!(e4.title, "");
        assert_eq!(e4.date, NaiveDate::from_ymd_opt(2026, 10, 28).unwrap());

        // Intervalo de horário.
        let e5 = parse_quick_event("Reunião amanhã 14:30-16:00", today);
        assert_eq!(e5.title, "Reunião");
        assert_eq!(e5.start_time, NaiveTime::from_hms_opt(14, 30, 0));
        assert_eq!(e5.end_time, NaiveTime::from_hms_opt(16, 0, 0));
    }
}
