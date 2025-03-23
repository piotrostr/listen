use super::*;

#[tokio::test]
async fn test_twitter_analyst() {
    let analyst = Analyst::from_env_with_locale("en".to_string()).unwrap();
    let result = analyst
        .analyze_twitter(
            "pwease",
            &std::fs::read_to_string("./debug/tweets_by_ids.json")
                .unwrap()
                .parse::<serde_json::Value>()
                .unwrap(),
            None,
        )
        .await
        .unwrap();
    println!("{:#?}", result);
}

#[tokio::test]
async fn test_chart_analyst() {
    use crate::data::Candlestick;

    // Create some sample candlestick data
    let candlesticks = vec![
        Candlestick {
            timestamp: 1625097600,
            open: 35000.0,
            high: 35500.0,
            low: 34800.0,
            close: 35200.0,
            volume: 1000.0,
        },
        Candlestick {
            timestamp: 1625184000,
            open: 35200.0,
            high: 36000.0,
            low: 35100.0,
            close: 35800.0,
            volume: 1200.0,
        },
    ];

    let analyst = Analyst::from_env_with_locale("en".to_string()).unwrap();
    let result = analyst
        .analyze_chart(&candlesticks, "1d", None)
        .await
        .unwrap();
    println!("{}", result);
}
